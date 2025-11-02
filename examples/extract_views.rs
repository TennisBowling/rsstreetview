use rsstreetview::{Direction, PanoramaSaveExt, StreetView, ViewConfig};

/// Example demonstrating view extraction from Street View panoramas.
///
/// This shows how to extract specific directional views (front, left, right)
/// without downloading the entire high-resolution panorama. This is much more
/// efficient when you only need specific views.
///
/// All views are saved as WebP for optimal file size and quality.
///
/// Run with:
/// ```bash
/// cargo run --example extract_views
/// ```
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Coordinates for the Colosseum in Rome
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
    let pano_id = &panos[0].pano_id;
    println!("Using panorama: {pano_id}");
    println!("Date: {:?}\n", panos[0].date);


    let configs = vec![
        ViewConfig::from_direction(Direction::Front).zoom(3),
        ViewConfig::from_direction(Direction::Left).zoom(3),
        ViewConfig::from_direction(Direction::Right).zoom(3),
    ];

    let views = client.extract_multiple_views(pano_id, &configs).await?;

    let directions = [Direction::Front, Direction::Left, Direction::Right];

    for (dir, view) in directions.iter().zip(views.iter()) {
        let filename = format!("{}.webp", dir.name());
        view.save_webp(&filename)?;
        println!("Saved: {filename}");
    }

    Ok(())
}
