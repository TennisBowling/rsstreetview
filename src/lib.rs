//! # rsstreetview
//!
//! An async Rust library for downloading Google Street View panoramas.
//!
//! This library provides:
//! - Search for panorama IDs using GPS coordinates
//! - Retrieve historical Street View photos
//! - Download full panoramic images (360-degree)
//! - Save images in multiple formats (JPEG, PNG, WebP)
//!
//! ## Example
//!
//! ```no_run
//! use rsstreetview::StreetView;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = StreetView::new();
//!
//!     // Search for panoramas
//!     let panos = client.search_panoramas(41.8982208, 12.4764804).await?;
//!
//!     // Download a panorama
//!     let image = client.download_panorama(&panos[0].pano_id, 5).await?;
//!
//!     // Save as WebP
//!     image.save("panorama.webp")?;
//!
//!     Ok(())
//! }
//! ```

mod error;
mod types;
mod search;
mod download;
mod metadata;
mod save;
mod utils;
pub mod views;

pub use error::{Result, StreetViewError};
pub use types::{ImageFormat, Location, MetaData, Panorama, SaveOptions};
pub use save::PanoramaSaveExt;
pub use views::{Direction, ViewConfig};

use reqwest::Client;

/// Main client for interacting with Google Street View.
///
/// This client maintains a reusable HTTP client for efficient connection pooling.
#[derive(Clone)]
pub struct StreetView {
    client: Client,
    api_key: Option<String>,
}

impl StreetView {
    /// Creates a new StreetView client without an API key.
    ///
    /// This is sufficient for searching and downloading panoramas using
    /// undocumented endpoints. An API key is only needed for official
    /// Google Maps API functions.
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            api_key: None,
        }
    }

    /// Creates a new StreetView client with a Google Maps API key.
    ///
    /// The API key is required for:
    /// - `get_panorama_meta()` - Get official metadata
    /// - `get_streetview()` - Get partial Street View images
    pub fn with_api_key(api_key: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            api_key: Some(api_key.into()),
        }
    }

    /// Creates a new StreetView client with a custom reqwest Client.
    ///
    /// This allows you to configure the HTTP client with custom settings
    /// such as proxies, timeouts, or custom headers.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use rsstreetview::StreetView;
    /// # use reqwest::Client;
    /// # use std::time::Duration;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let custom_client = Client::builder()
    ///     .timeout(Duration::from_secs(60))
    ///     .build()?;
    /// let client = StreetView::with_client(custom_client);
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_client(client: Client) -> Self {
        Self {
            client,
            api_key: None,
        }
    }

    /// Search for panoramas at a given GPS coordinate.
    ///
    /// Returns a list of panoramas ordered by relevance, including historical
    /// panoramas if available.
    ///
    /// # Arguments
    ///
    /// * `lat` - Latitude coordinate
    /// * `lon` - Longitude coordinate
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use rsstreetview::StreetView;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = StreetView::new();
    /// let panos = client.search_panoramas(41.8982208, 12.4764804).await?;
    /// println!("Found {} panoramas", panos.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn search_panoramas(&self, lat: f64, lon: f64) -> Result<Vec<Panorama>> {
        search::search_panoramas(&self.client, lat, lon).await
    }

    /// Search for panoramas from a Google Maps URL.
    ///
    /// Extracts the GPS coordinates from the URL and searches for panoramas.
    pub async fn search_panoramas_url(&self, url: &str) -> Result<Vec<Panorama>> {
        search::search_panoramas_url(&self.client, url).await
    }

    /// Find the exact panorama shown in a Google Maps URL.
    ///
    /// Returns the specific panorama if it can be identified from the URL.
    pub async fn search_panoramas_url_exact(&self, url: &str) -> Result<Option<Panorama>> {
        search::search_panoramas_url_exact(&self.client, url).await
    }

    /// Download a full panorama image.
    ///
    /// # Arguments
    ///
    /// * `pano_id` - The panorama ID
    /// * `zoom` - Zoom level (1-7, default 5)
    ///   - Zoom 5: 16384x8192 pixels (default)
    ///   - Zoom 4: 8192x4096 pixels
    ///   - Higher zoom = larger images
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use rsstreetview::StreetView;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = StreetView::new();
    /// let panos = client.search_panoramas(41.8982208, 12.4764804).await?;
    /// let image = client.download_panorama(&panos[0].pano_id, 5).await?;
    /// image.save("panorama.jpg")?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn download_panorama(
        &self,
        pano_id: &str,
        zoom: u8,
    ) -> Result<image::DynamicImage> {
        download::download_panorama(&self.client, pano_id, zoom).await
    }

    /// Get official metadata for a panorama.
    ///
    /// Requires an API key. Use `StreetView::with_api_key()` to set one.
    ///
    /// # Errors
    ///
    /// Returns an error if no API key is set.
    pub async fn get_panorama_meta(&self, pano_id: &str) -> Result<MetaData> {
        let api_key = self.api_key.as_ref()
            .ok_or_else(|| StreetViewError::MissingApiKey)?;
        metadata::get_panorama_meta(&self.client, pano_id, api_key).await
    }

    /// Get a partial Street View image using the official API.
    ///
    /// Requires an API key. Use `StreetView::with_api_key()` to set one.
    ///
    /// # Arguments
    ///
    /// * `pano_id` - The panorama ID
    /// * `width` - Image width (default 640)
    /// * `height` - Image height (default 640)
    /// * `heading` - Camera heading in degrees (0-360)
    /// * `fov` - Field of view (default 120)
    /// * `pitch` - Camera pitch in degrees
    pub async fn get_streetview(
        &self,
        pano_id: &str,
        width: u32,
        height: u32,
        heading: u16,
        fov: u16,
        pitch: i16,
    ) -> Result<image::DynamicImage> {
        let api_key = self.api_key.as_ref()
            .ok_or_else(|| StreetViewError::MissingApiKey)?;
        metadata::get_streetview(&self.client, pano_id, api_key, width, height, heading, fov, pitch).await
    }

    /// Extract a specific view from a panorama.
    ///
    /// This downloads the panorama at a moderate zoom level and extracts the
    /// requested view based on heading, FOV, and pitch. This is more efficient
    /// than downloading a full high-resolution panorama if you only need specific views.
    ///
    /// # Arguments
    ///
    /// * `pano_id` - The panorama ID
    /// * `config` - View configuration (heading, FOV, pitch, size)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use rsstreetview::{StreetView, ViewConfig, Direction};
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
        &self,
        pano_id: &str,
        config: &ViewConfig,
    ) -> Result<image::DynamicImage> {
        views::extract_view(&self.client, pano_id, config).await
    }

    /// Extract multiple views from a panorama in one call.
    ///
    /// This is more efficient than calling `extract_view` multiple times because
    /// it only downloads the panorama once.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use rsstreetview::{StreetView, ViewConfig, Direction};
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
    /// for (dir, view) in [Direction::Front, Direction::Right, Direction::Back, Direction::Left].iter().zip(views.iter()) {
    ///     view.save(format!("{}.jpg", dir.name()))?;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn extract_multiple_views(
        &self,
        pano_id: &str,
        configs: &[ViewConfig],
    ) -> Result<Vec<image::DynamicImage>> {
        views::extract_multiple_views(&self.client, pano_id, configs).await
    }

    /// Crop black borders from the bottom and right edges of a panorama.
    ///
    /// Some panoramas have black padding that can be removed.
    pub fn crop_black_borders(&self, img: image::DynamicImage) -> image::DynamicImage {
        utils::crop_bottom_and_right_black_border(img)
    }
}

impl Default for StreetView {
    fn default() -> Self {
        Self::new()
    }
}
