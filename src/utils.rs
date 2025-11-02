use image::{DynamicImage, GenericImageView};

const BLACK_LUMINANCE_THRESHOLD: u8 = 4;

/// Crop black borders from the bottom and right edges of a panorama.
///
/// Some panoramas have black padding on the edges that can be removed.
/// This function scans from the bottom and right edges inward to find
/// where the actual image content begins, then crops to that region.
///
/// # Arguments
///
/// * `img` - The panorama image to crop
///
/// # Returns
///
/// A new image with black borders removed, or the original image if
/// no significant black borders are detected.
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
///
/// // Crop black borders
/// let cropped = client.crop_black_borders(image);
/// cropped.save("panorama_cropped.jpg")?;
/// # Ok(())
/// # }
/// ```
pub fn crop_bottom_and_right_black_border(img: DynamicImage) -> DynamicImage {
    let (width, height) = img.dimensions();

    // Convert to luma (grayscale) for easier processing
    let luma_img = img.to_luma8();

    // Find the bottom crop point
    let mut bottom_crop = height;
    for y in (0..height).rev() {
        // Check if this row has any non-black pixels
        let mut has_content = false;
        for x in 0..width {
            let pixel = luma_img.get_pixel(x, y);
            if pixel[0] > BLACK_LUMINANCE_THRESHOLD {
                has_content = true;
                break;
            }
        }

        if has_content {
            bottom_crop = y + 1;
            break;
        }
    }

    // Validate that all pixels below bottom_crop are black
    let mut all_black_below = true;
    if bottom_crop < height {
        'outer: for y in bottom_crop..height {
            for x in 0..width {
                let pixel = luma_img.get_pixel(x, y);
                if pixel[0] > BLACK_LUMINANCE_THRESHOLD {
                    all_black_below = false;
                    break 'outer;
                }
            }
        }

        if !all_black_below {
            // False positive, don't crop
            bottom_crop = height;
        }
    }

    // Find the right crop point
    let mut right_crop = width;
    for x in (0..width).rev() {
        // Check if this column has any non-black pixels
        let mut has_content = false;
        for y in 0..bottom_crop {
            // Only check up to bottom_crop
            let pixel = luma_img.get_pixel(x, y);
            if pixel[0] > BLACK_LUMINANCE_THRESHOLD {
                has_content = true;
                break;
            }
        }

        if has_content {
            right_crop = x + 1;
            break;
        }
    }

    // Validate that all pixels to the right of right_crop are black
    let mut all_black_right = true;
    if right_crop < width {
        'outer2: for x in right_crop..width {
            for y in 0..bottom_crop {
                let pixel = luma_img.get_pixel(x, y);
                if pixel[0] > BLACK_LUMINANCE_THRESHOLD {
                    all_black_right = false;
                    break 'outer2;
                }
            }
        }

        if !all_black_right {
            // False positive, don't crop
            right_crop = width;
        }
    }

    // If no cropping needed, return original
    if bottom_crop == height && right_crop == width {
        return img;
    }

    // Crop the image
    img.crop_imm(0, 0, right_crop, bottom_crop)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgb, RgbImage};

    #[test]
    fn test_no_black_borders() {
        // Create image with no black borders
        let img = DynamicImage::ImageRgb8(RgbImage::from_pixel(100, 100, Rgb([255, 255, 255])));
        let cropped = crop_bottom_and_right_black_border(img.clone());

        assert_eq!(cropped.dimensions(), (100, 100));
    }

    #[test]
    fn test_with_black_bottom() {
        // Create image with black bottom border
        let mut img = RgbImage::from_pixel(100, 100, Rgb([255, 255, 255]));

        // Add black bottom rows
        for y in 90..100 {
            for x in 0..100 {
                img.put_pixel(x, y, Rgb([0, 0, 0]));
            }
        }

        let cropped = crop_bottom_and_right_black_border(DynamicImage::ImageRgb8(img));
        assert_eq!(cropped.dimensions(), (100, 90));
    }

    #[test]
    fn test_with_black_right() {
        // Create image with black right border
        let mut img = RgbImage::from_pixel(100, 100, Rgb([255, 255, 255]));

        // Add black right columns
        for x in 90..100 {
            for y in 0..100 {
                img.put_pixel(x, y, Rgb([0, 0, 0]));
            }
        }

        let cropped = crop_bottom_and_right_black_border(DynamicImage::ImageRgb8(img));
        assert_eq!(cropped.dimensions(), (90, 100));
    }

    #[test]
    fn test_with_both_borders() {
        // Create image with both black borders
        let mut img = RgbImage::from_pixel(100, 100, Rgb([255, 255, 255]));

        // Add black bottom rows
        for y in 90..100 {
            for x in 0..100 {
                img.put_pixel(x, y, Rgb([0, 0, 0]));
            }
        }

        // Add black right columns
        for x in 90..100 {
            for y in 0..90 {
                img.put_pixel(x, y, Rgb([0, 0, 0]));
            }
        }

        let cropped = crop_bottom_and_right_black_border(DynamicImage::ImageRgb8(img));
        assert_eq!(cropped.dimensions(), (90, 90));
    }
}
