use crate::error::Result;
use crate::download::{download_panorama};
use image::{DynamicImage, GenericImageView};
use reqwest::Client;

/// Cardinal direction for view extraction.
#[derive(Debug, Clone, Copy)]
pub enum Direction {
    /// Front view (heading 0°)
    Front,
    /// Right view (heading 90°)
    Right,
    /// Back view (heading 180°)
    Back,
    /// Left view (heading 270°)
    Left,
}

impl Direction {
    /// Get the heading in degrees for this direction.
    pub fn heading(&self) -> u16 {
        match self {
            Direction::Front => 0,
            Direction::Right => 90,
            Direction::Back => 180,
            Direction::Left => 270,
        }
    }

    /// Get the name of this direction as a string.
    pub fn name(&self) -> &'static str {
        match self {
            Direction::Front => "front",
            Direction::Right => "right",
            Direction::Back => "back",
            Direction::Left => "left",
        }
    }
}

/// Configuration for extracting a view from a panorama.
#[derive(Debug, Clone)]
pub struct ViewConfig {
    /// Heading in degrees (0-360, where 0 is front/north)
    pub heading: u16,
    /// Field of view in degrees (default: 90, typical range: 60-120)
    pub fov: u16,
    /// Vertical pitch in degrees (-90 to 90, where 0 is horizontal)
    pub pitch: i16,
    /// Output dimensions (optional - defaults to native panorama resolution)
    /// If None, uses the full resolution from the zoom level
    pub size: Option<(u32, u32)>,
    /// Zoom level for panorama download (1-7, default: 3)
    /// Higher zoom = more detail but slower download
    /// - Zoom 1: 1024×512
    /// - Zoom 2: 2048×1024
    /// - Zoom 3: 4096×2048 (default)
    /// - Zoom 4: 8192×4096
    /// - Zoom 5: 16384×8192
    /// - Zoom 6: 32768×16384
    /// - Zoom 7: 65536×32768
    pub zoom: u8,
}

impl ViewConfig {
    /// Create a new view configuration with default values.
    ///
    /// By default, uses native resolution from the panorama (no resizing).
    pub fn new(heading: u16) -> Self {
        Self {
            heading,
            fov: 90,
            pitch: 0,
            size: None,  // Native resolution by default
            zoom: 3,
        }
    }

    /// Create a view for a cardinal direction.
    pub fn from_direction(direction: Direction) -> Self {
        Self::new(direction.heading())
    }

    /// Set the field of view.
    pub fn fov(mut self, fov: u16) -> Self {
        self.fov = fov.min(180);
        self
    }

    /// Set the pitch.
    pub fn pitch(mut self, pitch: i16) -> Self {
        self.pitch = pitch.clamp(-90, 90);
        self
    }

    /// Set custom output dimensions.
    ///
    /// If not called, the view will use the native resolution from the panorama
    /// (determined by the zoom level).
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.size = Some((width, height));
        self
    }

    /// Set the zoom level for panorama download (1-7).
    ///
    /// Higher zoom = better quality but slower download:
    /// - Zoom 1: 1024×512 (4 tiles)
    /// - Zoom 2: 2048×1024 (8 tiles)
    /// - Zoom 3: 4096×2048 (32 tiles) - **default**
    /// - Zoom 4: 8192×4096 (128 tiles)
    /// - Zoom 5: 16384×8192 (512 tiles)
    /// - Zoom 6: 32768×16384 (2048 tiles)
    /// - Zoom 7: 65536×32768 (8192 tiles)
    pub fn zoom(mut self, zoom: u8) -> Self {
        self.zoom = zoom.clamp(1, 7);
        self
    }
}

impl Default for ViewConfig {
    fn default() -> Self {
        Self::new(0)
    }
}

/// Extract a specific view from a panorama.
///
/// This function downloads the full panorama at a lower zoom level and then
/// extracts the requested view. For maximum efficiency with API key access,
/// consider using `StreetView::get_streetview()` which uses the official API.
///
/// # Arguments
///
/// * `client` - HTTP client to use
/// * `pano_id` - The panorama ID
/// * `config` - View configuration (heading, FOV, pitch, size)
///
/// # Example
///
/// ```no_run
/// # use rsstreetview::{StreetView, views::{extract_view, ViewConfig, Direction}};
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let client = StreetView::new();
/// let panos = client.search_panoramas(41.8982208, 12.4764804).await?;
///
/// // Extract front view
/// let config = ViewConfig::from_direction(Direction::Front).size(1024, 1024);
/// let front_view = client.extract_view(&panos[0].pano_id, &config).await?;
/// front_view.save("front.jpg")?;
/// # Ok(())
/// # }
/// ```
pub async fn extract_view(
    client: &Client,
    pano_id: &str,
    config: &ViewConfig,
) -> Result<DynamicImage> {
    // Download panorama at the configured zoom level
    let panorama = download_panorama(client, pano_id, config.zoom).await?;

    // Extract the view from the panorama
    extract_view_from_panorama(&panorama, config)
}

/// Extract a view from an already-downloaded panorama.
///
/// This is useful if you've already downloaded a full panorama and want to
/// extract multiple views from it without re-downloading.
///
/// # Arguments
///
/// * `panorama` - The full panorama image (equirectangular projection)
/// * `config` - View configuration (heading, FOV, pitch, size)
pub fn extract_view_from_panorama(
    panorama: &DynamicImage,
    config: &ViewConfig,
) -> Result<DynamicImage> {
    let (pano_width, pano_height) = panorama.dimensions();

    // Calculate the horizontal span based on FOV
    // For equirectangular projection: pixels per degree = width / 360
    let pixels_per_degree_h = pano_width as f64 / 360.0;
    let pixels_per_degree_v = pano_height as f64 / 180.0;

    // Calculate the center point based on heading and pitch
    // Heading: 0° = center of image (x = width/2), wraps around
    // Pitch: 0° = center of image (y = height/2), -90° = top, +90° = bottom
    let center_x = ((config.heading as f64 / 360.0) * pano_width as f64) as u32;
    let center_y = (((90.0 - config.pitch as f64) / 180.0) * pano_height as f64) as u32;

    // Calculate the crop region based on FOV
    // Use a square aspect ratio if no custom size specified
    let aspect_ratio = if let Some((w, h)) = config.size {
        w as f64 / h as f64
    } else {
        1.0  // Square by default
    };

    let half_fov_h = config.fov as f64 / 2.0;
    let half_fov_v = half_fov_h / aspect_ratio;

    let crop_width = (half_fov_h * 2.0 * pixels_per_degree_h) as u32;
    let crop_height = (half_fov_v * 2.0 * pixels_per_degree_v) as u32;

    // Calculate crop boundaries
    let half_width = crop_width / 2;
    let half_height = crop_height / 2;

    // Handle wrapping for horizontal dimension
    let x_start = center_x.saturating_sub(half_width);

    let y_start = center_y.saturating_sub(half_height);

    let x_end = (x_start + crop_width).min(pano_width);
    let y_end = (y_start + crop_height).min(pano_height);

    // Simple crop (doesn't handle wrapping around the edges yet)
    let cropped = panorama.crop_imm(x_start, y_start, x_end - x_start, y_end - y_start);

    // Resize if custom size specified, otherwise use native resolution
    if let Some((width, height)) = config.size {
        // Resize to custom dimensions
        let resized = image::imageops::resize(
            &cropped,
            width,
            height,
            image::imageops::FilterType::Lanczos3,
        );

        // Convert to RGB (resize returns RGBA)
        let rgb_image = DynamicImage::ImageRgba8(resized).to_rgb8();
        Ok(DynamicImage::ImageRgb8(rgb_image))
    } else {
        // Use native resolution (no resize)
        Ok(cropped)
    }
}

/// Extract multiple views from a panorama in one call.
///
/// This is more efficient than calling `extract_view` multiple times because
/// it only downloads the panorama once.
///
/// # Example
///
/// ```no_run
/// # use rsstreetview::{StreetView, views::{extract_multiple_views, ViewConfig, Direction}};
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let client = StreetView::new();
/// let panos = client.search_panoramas(41.8982208, 12.4764804).await?;
///
/// // Extract all cardinal direction views
/// let configs = vec![
///     ViewConfig::from_direction(Direction::Front),
///     ViewConfig::from_direction(Direction::Right),
///     ViewConfig::from_direction(Direction::Back),
///     ViewConfig::from_direction(Direction::Left),
/// ];
///
/// let views = client.extract_multiple_views(&panos[0].pano_id, &configs).await?;
/// for (i, view) in views.iter().enumerate() {
///     view.save(format!("view_{}.jpg", i))?;
/// }
/// # Ok(())
/// # }
/// ```
pub async fn extract_multiple_views(
    client: &Client,
    pano_id: &str,
    configs: &[ViewConfig],
) -> Result<Vec<DynamicImage>> {
    if configs.is_empty() {
        return Ok(Vec::new());
    }

    // Download panorama once at the zoom level from the first config
    // (all configs should use the same zoom for efficiency)
    let zoom = configs[0].zoom;
    let panorama = download_panorama(client, pano_id, zoom).await?;

    // Extract all views from the same panorama
    let mut views = Vec::new();
    for config in configs {
        let view = extract_view_from_panorama(&panorama, config)?;
        views.push(view);
    }

    Ok(views)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_headings() {
        assert_eq!(Direction::Front.heading(), 0);
        assert_eq!(Direction::Right.heading(), 90);
        assert_eq!(Direction::Back.heading(), 180);
        assert_eq!(Direction::Left.heading(), 270);
    }

    #[test]
    fn test_view_config_builder() {
        let config = ViewConfig::new(45)
            .fov(120)
            .pitch(10)
            .size(800, 600);

        assert_eq!(config.heading, 45);
        assert_eq!(config.fov, 120);
        assert_eq!(config.pitch, 10);
    }

    #[test]
    fn test_direction_names() {
        assert_eq!(Direction::Front.name(), "front");
        assert_eq!(Direction::Right.name(), "right");
        assert_eq!(Direction::Back.name(), "back");
        assert_eq!(Direction::Left.name(), "left");
    }
}
