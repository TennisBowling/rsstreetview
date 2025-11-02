use rsstreetview::StreetView;

/// Detailed comparison example to match Python output.
///
/// Run with:
/// ```bash
/// cargo run --example compare_outputs
/// ```
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Same coordinates as Python example (Colosseum)
    let lat = 41.8982208;
    let lon = 12.4764804;

    let client = StreetView::new();

    println!("Searching for panoramas at the Colosseum...");

    let panos = client.search_panoramas(lat, lon).await?;

    if panos.is_empty() {
        println!("No panoramas found.");
        return Ok(());
    }

    println!("Found {} panoramas", panos.len());

    // Show first 5 panoramas for comparison with Python
    println!("\nFirst 5 panoramas found:");
    for (i, pano) in panos.iter().take(5).enumerate() {
        println!(
            "  {}. ID: {}, Date: {:?}, Heading: {:.1}°",
            i + 1,
            pano.pano_id,
            pano.date.as_deref().unwrap_or("None"),
            pano.heading
        );
    }

    let pano_id = &panos[0].pano_id;
    println!("\nUsing panorama: {pano_id}");
    println!("Date: {:?}", panos[0].date.as_deref().unwrap_or("None"));
    println!("Location: {}, {}", panos[0].lat, panos[0].lon);
    println!("Heading: {}", panos[0].heading);

    // Download at zoom 3 (same as Python)
    println!("\nDownloading panorama at zoom level 3 (4096x2048)...");
    let img = client.download_panorama(pano_id, 3).await?;
    println!("Downloaded: {}x{} pixels", img.width(), img.height());

    // Save as JPEG and WebP for comparison with Python
    use rsstreetview::PanoramaSaveExt;

    println!("\nSaving full panorama...");
    img.save_jpeg("rust_panorama_zoom3.jpg", 90)?;
    println!("✓ Saved: rust_panorama_zoom3.jpg");

    img.save_webp("rust_panorama_zoom3.webp")?;
    println!("✓ Saved: rust_panorama_zoom3.webp");

    println!("\n--- Comparison Info ---");
    println!("Panorama dimensions: {}x{}", img.width(), img.height());
    println!("First panorama ID: {pano_id}");

    Ok(())
}
