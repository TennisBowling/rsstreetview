use thiserror::Error;

/// Result type alias for StreetView operations.
pub type Result<T> = std::result::Result<T, StreetViewError>;

/// Errors that can occur when using the StreetView library.
#[derive(Error, Debug)]
pub enum StreetViewError {
    /// HTTP request failed
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    /// Failed to parse response
    #[error("Failed to parse response: {0}")]
    ParseError(String),

    /// Invalid response from Google
    #[error("Invalid response from Google: {0}")]
    InvalidResponse(String),

    /// Image processing error
    #[error("Image error: {0}")]
    ImageError(#[from] image::ImageError),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Missing API key
    #[error("API key required for this operation. Use StreetView::with_api_key() to set one.")]
    MissingApiKey,

    /// No panoramas found
    #[error("No panoramas found at the specified location")]
    NoPanoramasFound,

    /// Invalid URL format
    #[error("Invalid Google Maps URL format")]
    InvalidUrl,

    /// Tile download failed after retries
    #[error("Failed to download tile after {0} retries")]
    TileDownloadFailed(u32),
}
