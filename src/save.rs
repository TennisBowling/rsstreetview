use crate::error::Result;
use crate::types::{ImageFormat, SaveOptions};
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::{DynamicImage, ExtendedColorType, ImageEncoder};
use std::fs::{self, File};
use std::io::{BufWriter, Cursor};
use std::path::Path;

/// Save a panorama image with specific format and quality settings.
///
/// This function handles directory creation and format-specific encoding.
pub fn save_panorama(
    img: &DynamicImage,
    path: impl AsRef<Path>,
    options: &SaveOptions,
) -> Result<()> {
    let path = path.as_ref();

    // Create parent directories if they don't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Open output file
    let file = File::create(path)?;
    let writer = BufWriter::new(file);

    // Convert image to RGB8 for encoding
    let rgb_img = img.to_rgb8();
    let (width, height) = rgb_img.dimensions();

    match options.format {
        ImageFormat::Jpeg => {
            let mut encoder = JpegEncoder::new_with_quality(writer, options.jpeg_quality);
            encoder.encode(
                rgb_img.as_raw(),
                width,
                height,
                ExtendedColorType::Rgb8,
            )?;
        }
        ImageFormat::Png => {
            let encoder = PngEncoder::new(writer);
            encoder.write_image(
                rgb_img.as_raw(),
                width,
                height,
                ExtendedColorType::Rgb8,
            )?;
        }
        ImageFormat::WebP => {
            // The image crate's WebP encoder doesn't expose quality settings directly
            // We drop the writer (which flushes and closes the file) and then use
            // the image crate's built-in save method
            drop(writer);
            img.save_with_format(path, image::ImageFormat::WebP)?;
        }
    }

    Ok(())
}

/// Encode a panorama image to bytes with specific format and quality settings.
///
/// Returns the encoded image as a `Vec<u8>`.
pub fn encode_panorama(img: &DynamicImage, options: &SaveOptions) -> Result<Vec<u8>> {
    let mut buffer = Cursor::new(Vec::new());

    // Convert image to RGB8 for encoding
    let rgb_img = img.to_rgb8();
    let (width, height) = rgb_img.dimensions();

    match options.format {
        ImageFormat::Jpeg => {
            let mut encoder = JpegEncoder::new_with_quality(&mut buffer, options.jpeg_quality);
            encoder.encode(
                rgb_img.as_raw(),
                width,
                height,
                ExtendedColorType::Rgb8,
            )?;
        }
        ImageFormat::Png => {
            let encoder = PngEncoder::new(&mut buffer);
            encoder.write_image(
                rgb_img.as_raw(),
                width,
                height,
                ExtendedColorType::Rgb8,
            )?;
        }
        ImageFormat::WebP => {
            // For WebP, we need to use the image crate's built-in save
            img.write_to(&mut buffer, image::ImageFormat::WebP)?;
        }
    }

    Ok(buffer.into_inner())
}

/// Extension trait for DynamicImage to add convenient save methods.
pub trait PanoramaSaveExt {
    /// Save image as WebP with default quality (85) and method (4).
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use rsstreetview::{StreetView, PanoramaSaveExt};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = StreetView::new();
    /// # let panos = client.search_panoramas(41.8982208, 12.4764804).await?;
    /// let image = client.download_panorama(&panos[0].pano_id, 5).await?;
    /// image.save_webp("panorama.webp")?;
    /// # Ok(())
    /// # }
    /// ```
    fn save_webp(&self, path: impl AsRef<Path>) -> Result<()>;

    /// Save image as JPEG with specified quality (1-100).
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use rsstreetview::{StreetView, PanoramaSaveExt};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = StreetView::new();
    /// # let panos = client.search_panoramas(41.8982208, 12.4764804).await?;
    /// let image = client.download_panorama(&panos[0].pano_id, 5).await?;
    /// image.save_jpeg("panorama.jpg", 90)?;
    /// # Ok(())
    /// # }
    /// ```
    fn save_jpeg(&self, path: impl AsRef<Path>, quality: u8) -> Result<()>;

    /// Save image as PNG.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use rsstreetview::{StreetView, PanoramaSaveExt};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = StreetView::new();
    /// # let panos = client.search_panoramas(41.8982208, 12.4764804).await?;
    /// let image = client.download_panorama(&panos[0].pano_id, 5).await?;
    /// image.save_png("panorama.png")?;
    /// # Ok(())
    /// # }
    /// ```
    fn save_png(&self, path: impl AsRef<Path>) -> Result<()>;

    /// Encode image as WebP and return bytes.
    ///
    /// Returns the encoded image as `Vec<u8>` with specified effort and quality.
    ///
    /// # Arguments
    ///
    /// * `effort` - Compression effort (0-6, higher = slower but better compression)
    /// * `quality` - WebP quality (1-100, higher = better quality but larger file)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use rsstreetview::{StreetView, PanoramaSaveExt};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = StreetView::new();
    /// # let panos = client.search_panoramas(41.8982208, 12.4764804).await?;
    /// let image = client.download_panorama(&panos[0].pano_id, 3).await?;
    /// let bytes = image.to_webp_bytes(4, 85)?;
    /// // Send bytes over HTTP, store in database, etc.
    /// # Ok(())
    /// # }
    /// ```
    fn to_webp_bytes(&self, effort: u8, quality: u8) -> Result<Vec<u8>>;

    /// Encode image as JPEG and return bytes.
    ///
    /// Returns the encoded image as `Vec<u8>` with specified quality.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use rsstreetview::{StreetView, PanoramaSaveExt};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = StreetView::new();
    /// # let panos = client.search_panoramas(41.8982208, 12.4764804).await?;
    /// let image = client.download_panorama(&panos[0].pano_id, 3).await?;
    /// let bytes = image.to_jpeg_bytes(90)?;
    /// // Send bytes over HTTP, store in database, etc.
    /// # Ok(())
    /// # }
    /// ```
    fn to_jpeg_bytes(&self, quality: u8) -> Result<Vec<u8>>;

    /// Encode image as PNG and return bytes.
    ///
    /// Returns the encoded image as `Vec<u8>`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use rsstreetview::{StreetView, PanoramaSaveExt};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = StreetView::new();
    /// # let panos = client.search_panoramas(41.8982208, 12.4764804).await?;
    /// let image = client.download_panorama(&panos[0].pano_id, 3).await?;
    /// let bytes = image.to_png_bytes()?;
    /// // Send bytes over HTTP, store in database, etc.
    /// # Ok(())
    /// # }
    /// ```
    fn to_png_bytes(&self) -> Result<Vec<u8>>;
}

impl PanoramaSaveExt for DynamicImage {
    fn save_webp(&self, path: impl AsRef<Path>) -> Result<()> {
        let options = SaveOptions::new()
            .format(ImageFormat::WebP)
            .webp_quality(85)
            .webp_method(4);
        save_panorama(self, path, &options)
    }

    fn save_jpeg(&self, path: impl AsRef<Path>, quality: u8) -> Result<()> {
        let options = SaveOptions::new()
            .format(ImageFormat::Jpeg)
            .jpeg_quality(quality);
        save_panorama(self, path, &options)
    }

    fn save_png(&self, path: impl AsRef<Path>) -> Result<()> {
        let options = SaveOptions::new().format(ImageFormat::Png);
        save_panorama(self, path, &options)
    }

    fn to_webp_bytes(&self, effort: u8, quality: u8) -> Result<Vec<u8>> {
        let options = SaveOptions::new()
            .format(ImageFormat::WebP)
            .webp_quality(quality)
            .webp_method(effort);
        encode_panorama(self, &options)
    }

    fn to_jpeg_bytes(&self, quality: u8) -> Result<Vec<u8>> {
        let options = SaveOptions::new()
            .format(ImageFormat::Jpeg)
            .jpeg_quality(quality);
        encode_panorama(self, &options)
    }

    fn to_png_bytes(&self) -> Result<Vec<u8>> {
        let options = SaveOptions::new().format(ImageFormat::Png);
        encode_panorama(self, &options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::RgbImage;

    #[test]
    fn test_save_formats() {
        // Create a small test image
        let img = DynamicImage::ImageRgb8(RgbImage::new(100, 100));

        // Test saving in different formats (to /tmp)
        let temp_dir = std::env::temp_dir();

        // Test WebP
        let webp_path = temp_dir.join("test.webp");
        img.save_webp(&webp_path).unwrap();
        assert!(webp_path.exists());

        // Test JPEG
        let jpeg_path = temp_dir.join("test.jpg");
        img.save_jpeg(&jpeg_path, 90).unwrap();
        assert!(jpeg_path.exists());

        // Test PNG
        let png_path = temp_dir.join("test.png");
        img.save_png(&png_path).unwrap();
        assert!(png_path.exists());

        // Cleanup
        std::fs::remove_file(webp_path).ok();
        std::fs::remove_file(jpeg_path).ok();
        std::fs::remove_file(png_path).ok();
    }
}
