use rsstreetview::{PanoramaSaveExt, StreetView};
use std::env;

/// Basic usage example demonstrating the core functionality of rsstreetview.
///
/// This example is ported from the Python Jupyter notebook and shows:
/// 1. Searching for panoramas at a GPS coordinate
/// 2. Getting metadata for a panorama (requires API key)
/// 3. Downloading a partial Street View image (requires API key)
/// 4. Downloading a full panorama image
///
/// Run with:
/// ```bash
/// GOOGLE_MAPS_API_KEY=your_key cargo run --example basic_usage
/// ```
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Coordinates for the Colosseum in Rome
    let lat = 41.8982208;
    let lon = 12.4764804;

    // Create client (no API key needed for search and panorama download)
    let client = StreetView::new();

    println!("Searching for panoramas at coordinates: {lat}, {lon}");

    // Search for panoramas
    let panos = client.search_panoramas(lat, lon).await?;

    println!("Found {} panoramas:", panos.len());
    for (i, pano) in panos.iter().take(5).enumerate() {
        println!(
            "  {}. ID: {}, Date: {:?}, Heading: {:.1}°",
            i + 1,
            pano.pano_id,
            pano.date,
            pano.heading
        );
    }

    if panos.is_empty() {
        println!("No panoramas found. Exiting.");
        return Ok(());
    }

    // Select the second panorama (index 1) to match the Python example
    let pano_id = &panos[1].pano_id;
    println!("\nUsing panorama: {pano_id}");

    // Get metadata (requires API key)
    if let Ok(api_key) = env::var("GOOGLE_MAPS_API_KEY") {
        println!("\nFetching metadata with API key...");
        let client_with_key = StreetView::with_api_key(api_key.clone());

        match client_with_key.get_panorama_meta(pano_id).await {
            Ok(meta) => {
                println!("Metadata:");
                println!("  Date: {}", meta.date);
                println!("  Location: {}, {}", meta.location.lat, meta.location.lng);
                println!("  Copyright: {}", meta.copyright);
            }
            Err(e) => println!("Failed to get metadata: {e}"),
        }

        // Get partial Street View image (official API)
        println!("\nDownloading partial Street View image (640x640)...");
        match client_with_key
            .get_streetview(pano_id, 640, 640, 0, 120, 0)
            .await
        {
            Ok(image) => {
                image.save_jpeg("streetview_partial.jpg", 90)?;
                println!("Saved to: streetview_partial.jpg");
            }
            Err(e) => println!("Failed to get Street View image: {e}"),
        }
    } else {
        println!("\nSkipping metadata and partial image (no API key set)");
        println!("Set GOOGLE_MAPS_API_KEY environment variable to use these features");
    }

    // Download full panorama (no API key needed)
    println!("\nDownloading full panorama at zoom level 1 (1024x512 pixels)...");
    let img_panorama = client.download_panorama(pano_id, 1).await?;

    println!(
        "Downloaded panorama: {}x{} pixels",
        img_panorama.width(),
        img_panorama.height()
    );

    // Save as JPEG
    img_panorama.save_jpeg("panorama.jpg", 90)?;
    println!("Saved to: panorama.jpg");

    // Also save as WebP (smaller file size)
    img_panorama.save_webp("panorama.webp")?;
    println!("Saved to: panorama.webp");

    println!("\nDone!");

    Ok(())
}
