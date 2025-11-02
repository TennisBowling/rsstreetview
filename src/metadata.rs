use crate::error::Result;
use crate::types::{Location, MetaData};
use image::DynamicImage;
use reqwest::Client;
use serde::Deserialize;

const METADATA_ENDPOINT: &str = "https://maps.googleapis.com/maps/api/streetview/metadata";
const STREETVIEW_ENDPOINT: &str = "https://maps.googleapis.com/maps/api/streetview";

/// Internal structure for parsing metadata response
#[derive(Debug, Deserialize)]
struct MetaDataResponse {
    date: String,
    location: LocationResponse,
    pano_id: String,
    copyright: String,
}

#[derive(Debug, Deserialize)]
struct LocationResponse {
    lat: f64,
    lng: f64,
}

/// Get official metadata for a panorama using the Google Maps API.
///
/// This function requires an API key but does not consume quota.
///
/// # Arguments
///
/// * `client` - HTTP client to use
/// * `pano_id` - The panorama ID
/// * `api_key` - Google Maps API key
///
/// # Example
///
/// ```no_run
/// # use rsstreetview::StreetView;
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let client = StreetView::with_api_key("YOUR_API_KEY");
/// let panos = client.search_panoramas(41.8982208, 12.4764804).await?;
/// let meta = client.get_panorama_meta(&panos[0].pano_id).await?;
/// println!("Date: {}, Copyright: {}", meta.date, meta.copyright);
/// # Ok(())
/// # }
/// ```
pub async fn get_panorama_meta(
    client: &Client,
    pano_id: &str,
    api_key: &str,
) -> Result<MetaData> {
    let url = format!("{METADATA_ENDPOINT}?pano={pano_id}&key={api_key}");

    let response = client.get(&url).send().await?;
    let data: MetaDataResponse = response.json().await?;

    Ok(MetaData {
        date: data.date,
        location: Location {
            lat: data.location.lat,
            lng: data.location.lng,
        },
        pano_id: data.pano_id,
        copyright: data.copyright,
    })
}

/// Get a partial Street View image using the official Google Maps API.
///
/// This returns a rendered view of the panorama from a specific angle,
/// not the full 360-degree panorama.
///
/// # Arguments
///
/// * `client` - HTTP client to use
/// * `pano_id` - The panorama ID
/// * `api_key` - Google Maps API key
/// * `width` - Image width in pixels (max 640 for free tier)
/// * `height` - Image height in pixels (max 640 for free tier)
/// * `heading` - Camera heading in degrees (0-360)
/// * `fov` - Field of view in degrees (default 120, max 120)
/// * `pitch` - Camera pitch in degrees (-90 to 90)
///
/// # Example
///
/// ```no_run
/// # use rsstreetview::StreetView;
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let client = StreetView::with_api_key("YOUR_API_KEY");
/// let panos = client.search_panoramas(41.8982208, 12.4764804).await?;
///
/// // Get a view looking north (heading=0) at normal pitch
/// let image = client.get_streetview(&panos[0].pano_id, 640, 640, 0, 120, 0).await?;
/// image.save("view.jpg")?;
/// # Ok(())
/// # }
/// ```
pub async fn get_streetview(
    client: &Client,
    pano_id: &str,
    api_key: &str,
    width: u32,
    height: u32,
    heading: u16,
    fov: u16,
    pitch: i16,
) -> Result<DynamicImage> {
    let url = format!(
        "{STREETVIEW_ENDPOINT}?size={width}x{height}&fov={fov}&pitch={pitch}&heading={heading}&pano={pano_id}&key={api_key}"
    );

    let response = client.get(&url).send().await?;
    let bytes = response.bytes().await?;

    let img = image::load_from_memory(&bytes)?;
    Ok(img)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_url_construction() {
        let url = format!("{}?pano={}&key={}", METADATA_ENDPOINT, "test_pano", "test_key");
        assert!(url.contains("pano=test_pano"));
        assert!(url.contains("key=test_key"));
    }

    #[test]
    fn test_streetview_url_construction() {
        let url = format!(
            "{}?size={}x{}&fov={}&pitch={}&heading={}&pano={}&key={}",
            STREETVIEW_ENDPOINT, 640, 640, 120, 0, 90, "test_pano", "test_key"
        );
        assert!(url.contains("size=640x640"));
        assert!(url.contains("fov=120"));
        assert!(url.contains("heading=90"));
    }
}
