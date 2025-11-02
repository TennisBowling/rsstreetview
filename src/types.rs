use serde::{Deserialize, Serialize};

/// A Street View panorama with location and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Panorama {
    /// Unique panorama identifier
    pub pano_id: String,
    /// Latitude coordinate
    pub lat: f64,
    /// Longitude coordinate
    pub lon: f64,
    /// Camera heading in degrees (0-360)
    pub heading: f64,
    /// Camera pitch in degrees (optional)
    pub pitch: Option<f64>,
    /// Camera roll in degrees (optional)
    pub roll: Option<f64>,
    /// Date in YYYY-MM format (optional)
    pub date: Option<String>,
    /// Elevation/altitude data (optional)
    pub elevation: Option<f64>,
}

/// GPS location with latitude and longitude.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    /// Latitude coordinate
    pub lat: f64,
    /// Longitude coordinate
    pub lng: f64,
}

/// Official metadata from Google Maps API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaData {
    /// Date of panorama capture
    pub date: String,
    /// GPS location
    pub location: Location,
    /// Panorama ID
    pub pano_id: String,
    /// Copyright information
    pub copyright: String,
}

/// Image output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    /// JPEG format
    Jpeg,
    /// PNG format
    Png,
    /// WebP format (recommended for best compression)
    WebP,
}

impl From<ImageFormat> for image::ImageFormat {
    fn from(format: ImageFormat) -> Self {
        match format {
            ImageFormat::Jpeg => image::ImageFormat::Jpeg,
            ImageFormat::Png => image::ImageFormat::Png,
            ImageFormat::WebP => image::ImageFormat::WebP,
        }
    }
}

/// Options for saving panorama images.
#[derive(Debug, Clone)]
pub struct SaveOptions {
    /// Image format
    pub format: ImageFormat,
    /// JPEG quality (1-100, default 90)
    pub jpeg_quality: u8,
    /// WebP quality (1-100, default 85)
    pub webp_quality: u8,
    /// WebP compression method (0-6, default 4)
    /// Higher values = slower but better compression
    pub webp_method: u8,
}

impl SaveOptions {
    /// Create default save options with WebP format.
    pub fn new() -> Self {
        Self {
            format: ImageFormat::WebP,
            jpeg_quality: 90,
            webp_quality: 85,
            webp_method: 4,
        }
    }

    /// Set the output format.
    pub fn format(mut self, format: ImageFormat) -> Self {
        self.format = format;
        self
    }

    /// Set JPEG quality (1-100).
    pub fn jpeg_quality(mut self, quality: u8) -> Self {
        self.jpeg_quality = quality.clamp(1, 100);
        self
    }

    /// Set WebP quality (1-100).
    pub fn webp_quality(mut self, quality: u8) -> Self {
        self.webp_quality = quality.clamp(1, 100);
        self
    }

    /// Set WebP compression method (0-6).
    ///
    /// Higher values produce smaller files but take longer to encode.
    /// Default is 4, which provides a good balance.
    pub fn webp_method(mut self, method: u8) -> Self {
        self.webp_method = method.min(6);
        self
    }

    /// Save an image with these options.
    pub fn save(&self, img: &image::DynamicImage, path: impl AsRef<std::path::Path>) -> crate::error::Result<()> {
        crate::save::save_panorama(img, path, self)
    }
}

impl Default for SaveOptions {
    fn default() -> Self {
        Self::new()
    }
}

/// Internal: Information about a single tile to download.
#[derive(Debug, Clone)]
pub(crate) struct TileInfo {
    pub x: u32,
    pub y: u32,
    pub url: String,
}

/// Internal: A downloaded tile with its position.
#[derive(Debug)]
pub(crate) struct Tile {
    pub x: u32,
    pub y: u32,
    pub image: image::DynamicImage,
}
