use rsstreetview::{ImageFormat, PanoramaSaveExt, SaveOptions, StreetView};

/// Example demonstrating WebP saving with different quality settings.
///
/// This shows how to save panoramas in WebP format, which provides superior
/// compression compared to JPEG while maintaining good image quality.
///
/// Run with:
/// ```bash
/// cargo run --example save_webp
/// ```
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Coordinates for the Eiffel Tower in Paris
    let lat = 48.8584;
    let lon = 2.2945;

    let client = StreetView::new();

    println!("Searching for panoramas at the Eiffel Tower...");

    let panos = client.search_panoramas(lat, lon).await?;

    if panos.is_empty() {
        println!("No panoramas found.");
        return Ok(());
    }

    println!("Found {} panoramas", panos.len());
    let pano_id = &panos[0].pano_id;
    println!("Using panorama: {pano_id}");

    // Download panorama at zoom level 2 (2048x1024 pixels)
    println!("\nDownloading panorama at zoom level 2...");
    let img = client.download_panorama(pano_id, 2).await?;
    println!("Downloaded: {}x{} pixels", img.width(), img.height());

    // Method 1: Save with default WebP settings (quality=85, method=4)
    println!("\n--- Saving with different formats ---");

    println!("1. Saving as WebP with default settings (quality=85, method=4)...");
    img.save_webp("output_default.webp")?;
    let webp_size = std::fs::metadata("output_default.webp")?.len();
    println!("   Saved: output_default.webp ({webp_size} bytes)");

    // Method 2: Save as JPEG for comparison
    println!("2. Saving as JPEG (quality=90) for comparison...");
    img.save_jpeg("output.jpg", 90)?;
    let jpeg_size = std::fs::metadata("output.jpg")?.len();
    println!("   Saved: output.jpg ({jpeg_size} bytes)");

    // Method 3: Save as PNG for comparison
    println!("3. Saving as PNG for comparison...");
    img.save_png("output.png")?;
    let png_size = std::fs::metadata("output.png")?.len();
    println!("   Saved: output.png ({png_size} bytes)");

    // Method 4: Using SaveOptions builder for custom settings
    println!("4. Saving as WebP with custom quality (95)...");
    SaveOptions::new()
        .format(ImageFormat::WebP)
        .webp_quality(95)
        .webp_method(4)
        .save(&img, "output_high_quality.webp")?;
    let webp_hq_size = std::fs::metadata("output_high_quality.webp")?.len();
    println!(
        "   Saved: output_high_quality.webp ({webp_hq_size} bytes)"
    );

    // Method 5: Lower quality WebP for smaller file size
    println!("5. Saving as WebP with lower quality (70) for smaller file...");
    SaveOptions::new()
        .format(ImageFormat::WebP)
        .webp_quality(70)
        .webp_method(4)
        .save(&img, "output_low_quality.webp")?;
    let webp_lq_size = std::fs::metadata("output_low_quality.webp")?.len();
    println!(
        "   Saved: output_low_quality.webp ({webp_lq_size} bytes)"
    );

    // Print comparison
    println!("\n--- File Size Comparison ---");
    println!("JPEG (q=90):          {jpeg_size:>10} bytes (100%)");
    println!(
        "PNG:                  {:>10} bytes ({:.1}%)",
        png_size,
        (png_size as f64 / jpeg_size as f64) * 100.0
    );
    println!(
        "WebP (q=85, default): {:>10} bytes ({:.1}%)",
        webp_size,
        (webp_size as f64 / jpeg_size as f64) * 100.0
    );
    println!(
        "WebP (q=95):          {:>10} bytes ({:.1}%)",
        webp_hq_size,
        (webp_hq_size as f64 / jpeg_size as f64) * 100.0
    );
    println!(
        "WebP (q=70):          {:>10} bytes ({:.1}%)",
        webp_lq_size,
        (webp_lq_size as f64 / jpeg_size as f64) * 100.0
    );

    println!("\nWebP typically provides 25-35% better compression than JPEG at similar quality!");
    println!("Default settings (quality=85, method=4) provide a good balance.");

    Ok(())
}
