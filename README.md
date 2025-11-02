# rsstreetview

An async Rust library for downloading Google Street View panoramas, with WebP support and efficient view extraction.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
rsstreetview = "0.1.0"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

```rust
use rsstreetview::{StreetView, PanoramaSaveExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = StreetView::new();

    // Search for panoramas
    let panos = client.search_panoramas(41.8982208, 12.4764804).await?;

    // Download a panorama (zoom 5 = 16384x8192 pixels)
    let image = client.download_panorama(&panos[0].pano_id, 5).await?;

    // Save as WebP (default quality 85, method 4)
    image.save_webp("panorama.webp")?;

    Ok(())
}
```

## Examples

### Search for Panoramas

```rust
let client = StreetView::new();

// By GPS coordinates
let panos = client.search_panoramas(41.8982208, 12.4764804).await?;

// From Google Maps URL
let panos = client.search_panoramas_url(
    "https://www.google.com/maps/@41.8982208,12.4764804,3a,75y,90h,90t/data=..."
).await?;

// Find exact panorama from URL
let pano = client.search_panoramas_url_exact(url).await?;
```

### Download Panoramas

```rust
// Zoom levels:
// 1: 1024x512      (0.5 MP)
// 2: 2048x1024     (2 MP)
// 3: 4096x2048     (8 MP)
// 4: 8192x4096     (33 MP)
// 5: 16384x8192    (134 MP) - default
// 6: 32768x16384   (536 MP)
// 7: 65536x32768   (2147 MP)

let image = client.download_panorama(&pano_id, 3).await?;
```

### Save in Different Formats

```rust
use rsstreetview::{PanoramaSaveExt, SaveOptions, ImageFormat};

// WebP (smallest file size)
image.save_webp("output.webp")?;

// JPEG with quality
image.save_jpeg("output.jpg", 90)?;

// PNG
image.save_png("output.png")?;

// Using SaveOptions builder
SaveOptions::new()
    .format(ImageFormat::WebP)
    .webp_quality(95)
    .save(&image, "high_quality.webp")?;
```

### Extract Specific Views (Efficient!)

Instead of downloading the entire panorama, extract only the views you need:

```rust
use rsstreetview::{Direction, ViewConfig};

// Extract a single direction view
let config = ViewConfig::from_direction(Direction::Front).size(1024, 1024);
let front_view = client.extract_view(&pano_id, &config).await?;
front_view.save_webp("front.webp")?;

// Extract all cardinal directions at once (more efficient!)
let configs = vec![
    ViewConfig::from_direction(Direction::Front),
    ViewConfig::from_direction(Direction::Right),
    ViewConfig::from_direction(Direction::Back),
    ViewConfig::from_direction(Direction::Left),
];

let views = client.extract_multiple_views(&pano_id, &configs).await?;

for (dir, view) in [Direction::Front, Direction::Right, Direction::Back, Direction::Left]
    .iter().zip(views.iter())
{
    view.save_webp(format!("{}.webp", dir.name()))?;
}
```

### Custom Views

```rust
// Custom heading and field of view
let config = ViewConfig::new(45)  // 45° heading
    .fov(120)                      // 120° field of view
    .pitch(30)                     // Look up 30°
    .size(1280, 720);              // 720p output

let view = client.extract_view(&pano_id, &config).await?;
```

### Official Google Maps API

```rust
// Requires API key
let client = StreetView::with_api_key("YOUR_API_KEY");

// Get metadata (no quota usage)
let meta = client.get_panorama_meta(&pano_id).await?;
println!("Date: {}, Copyright: {}", meta.date, meta.copyright);

// Get partial Street View image (official API)
let image = client.get_streetview(
    &pano_id,
    640,    // width
    640,    // height
    0,      // heading
    120,    // field of view
    0       // pitch
).await?;
```

### Utility Functions

```rust
// Crop black borders
let cropped = client.crop_black_borders(image);
```

## Running Examples

```bash
# Basic usage
cargo run --example basic_usage

# WebP format comparison
cargo run --example save_webp

# View extraction
cargo run --example extract_views

# With API key
GOOGLE_MAPS_API_KEY=your_key cargo run --example basic_usage
```

## Architecture

- **Async-only**: Built on `tokio` and `reqwest` for efficient I/O
- **Connection pooling**: Reusable HTTP client across all requests
- **Concurrent downloads**: Tiles downloaded in parallel with controlled concurrency
- **Error handling**: Typed errors with automatic retry logic
- **Zero-copy where possible**: Efficient memory usage

## Important Notes

⚠️ **Undocumented API**: This library uses Google's undocumented Street View tile endpoints, which may change without notice.

⚠️ **Rate limiting**: Google may rate-limit or temporarily block IPs making too many requests. The library includes:
- Automatic retry logic with exponential backoff
- Controlled concurrency (8 concurrent tile downloads)
- Consider using delays between panorama downloads

⚠️ **Terms of Service**: Ensure your use complies with Google's terms of service.

## Performance Tips

1. **Use view extraction** instead of full panoramas when possible
2. **Choose appropriate zoom levels** - zoom 3 (4096x2048) is good for most uses
3. **Use WebP** for 25-35% smaller file sizes
4. **Batch operations** - use `extract_multiple_views` instead of multiple `extract_view` calls
5. **Reuse client** - create one `StreetView` instance and reuse it

Generate local docs:
```bash
cargo doc --open
```

## Credits

Based on the Python [streetview](https://github.com/robolyst/streetview) library.