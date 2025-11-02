use crate::error::{Result, StreetViewError};
use crate::types::{Tile, TileInfo};
use futures::stream::{self, StreamExt};
use image::{DynamicImage, GenericImage};
use reqwest::Client;
use std::time::Duration;

const TILE_WIDTH: u32 = 512;
const TILE_HEIGHT: u32 = 512;
const DEFAULT_MAX_RETRIES: u32 = 6;
const RETRY_DELAY_SECS: u64 = 2;
const TILE_ENDPOINT: &str = "https://cbk0.google.com/cbk";
const CONCURRENT_DOWNLOADS: usize = 8;

/// Calculate the width and height of the panorama grid from zoom level.
///
/// Returns (width_in_tiles, height_in_tiles)
fn get_width_and_height_from_zoom(zoom: u8) -> (u32, u32) {
    let width = 2_u32.pow(zoom as u32);
    let height = 2_u32.pow((zoom - 1) as u32);
    (width, height)
}

/// Build the download URL for a single tile.
fn make_download_url(pano_id: &str, zoom: u8, x: u32, y: u32) -> String {
    format!(
        "{TILE_ENDPOINT}?output=tile&panoid={pano_id}&zoom={zoom}&x={x}&y={y}"
    )
}

/// Generate all tile info for a panorama.
fn iter_tile_info(pano_id: &str, zoom: u8) -> Vec<TileInfo> {
    let (width, height) = get_width_and_height_from_zoom(zoom);
    let mut tiles = Vec::new();

    for y in 0..height {
        for x in 0..width {
            tiles.push(TileInfo {
                x,
                y,
                url: make_download_url(pano_id, zoom, x, y),
            });
        }
    }

    tiles
}

/// Download a single tile with retry logic.
async fn fetch_tile_with_retry(
    client: &Client,
    tile_info: &TileInfo,
    max_retries: u32,
) -> Result<Tile> {
    let mut retries = 0;

    loop {
        match client.get(&tile_info.url).send().await {
            Ok(response) => {
                match response.bytes().await {
                    Ok(bytes) => {
                        // Try to load the image
                        match image::load_from_memory(&bytes) {
                            Ok(img) => {
                                return Ok(Tile {
                                    x: tile_info.x,
                                    y: tile_info.y,
                                    image: img,
                                });
                            }
                            Err(e) => {
                                if retries >= max_retries {
                                    return Err(StreetViewError::ImageError(e));
                                }
                            }
                        }
                    }
                    Err(e) => {
                        if retries >= max_retries {
                            return Err(StreetViewError::HttpError(e));
                        }
                    }
                }
            }
            Err(_e) => {
                if retries >= max_retries {
                    return Err(StreetViewError::TileDownloadFailed(max_retries));
                }
                // Connection error, retry
            }
        }

        retries += 1;
        tokio::time::sleep(Duration::from_secs(RETRY_DELAY_SECS)).await;
    }
}

/// Download all tiles for a panorama concurrently.
async fn download_tiles(client: &Client, pano_id: &str, zoom: u8) -> Result<Vec<Tile>> {
    let tile_infos = iter_tile_info(pano_id, zoom);

    // Download tiles concurrently with controlled concurrency
    let tiles: Vec<Result<Tile>> = stream::iter(tile_infos)
        .map(|tile_info| async move {
            fetch_tile_with_retry(client, &tile_info, DEFAULT_MAX_RETRIES).await
        })
        .buffer_unordered(CONCURRENT_DOWNLOADS)
        .collect()
        .await;

    // Collect results and return errors if any
    tiles.into_iter().collect()
}

/// Assemble tiles into a single panorama image.
fn assemble_tiles(tiles: Vec<Tile>, zoom: u8) -> Result<DynamicImage> {
    let (width_tiles, height_tiles) = get_width_and_height_from_zoom(zoom);
    let width_pixels = width_tiles * TILE_WIDTH;
    let height_pixels = height_tiles * TILE_HEIGHT;

    // Create a new image to hold the panorama
    let mut panorama = DynamicImage::new_rgb8(width_pixels, height_pixels);

    // Paste each tile into the panorama
    for tile in tiles {
        let x_offset = tile.x * TILE_WIDTH;
        let y_offset = tile.y * TILE_HEIGHT;

        // Copy the tile into the panorama
        panorama.copy_from(&tile.image, x_offset, y_offset)
            .map_err(StreetViewError::ImageError)?;
    }

    Ok(panorama)
}

/// Download a full panorama image.
///
/// # Arguments
///
/// * `client` - HTTP client to use for requests
/// * `pano_id` - The panorama ID
/// * `zoom` - Zoom level (1-7)
///   - Zoom 1: 1024x512 pixels
///   - Zoom 2: 2048x1024 pixels
///   - Zoom 3: 4096x2048 pixels
///   - Zoom 4: 8192x4096 pixels
///   - Zoom 5: 16384x8192 pixels (recommended default)
///   - Zoom 6: 32768x16384 pixels
///   - Zoom 7: 65536x32768 pixels
///
/// Higher zoom levels produce larger images with more detail but take longer to download.
pub async fn download_panorama(client: &Client, pano_id: &str, zoom: u8) -> Result<DynamicImage> {
    // Validate zoom level
    if !(1..=7).contains(&zoom) {
        return Err(StreetViewError::ParseError(
            "Zoom level must be between 1 and 7".to_string(),
        ));
    }

    // Download all tiles
    let tiles = download_tiles(client, pano_id, zoom).await?;

    // Assemble into final panorama
    assemble_tiles(tiles, zoom)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zoom_dimensions() {
        assert_eq!(get_width_and_height_from_zoom(1), (2, 1));
        assert_eq!(get_width_and_height_from_zoom(2), (4, 2));
        assert_eq!(get_width_and_height_from_zoom(3), (8, 4));
        assert_eq!(get_width_and_height_from_zoom(4), (16, 8));
        assert_eq!(get_width_and_height_from_zoom(5), (32, 16));
    }

    #[test]
    fn test_make_download_url() {
        let url = make_download_url("test_pano_id", 3, 5, 2);
        assert!(url.contains("panoid=test_pano_id"));
        assert!(url.contains("zoom=3"));
        assert!(url.contains("x=5"));
        assert!(url.contains("y=2"));
    }

    #[test]
    fn test_iter_tile_info() {
        let tiles = iter_tile_info("test", 2);
        assert_eq!(tiles.len(), 8); // 4x2 = 8 tiles

        // Check first and last tiles
        assert_eq!(tiles[0].x, 0);
        assert_eq!(tiles[0].y, 0);
        assert_eq!(tiles[7].x, 3);
        assert_eq!(tiles[7].y, 1);
    }
}
