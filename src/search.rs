use crate::error::{Result, StreetViewError};
use crate::types::Panorama;
use regex::Regex;
use reqwest::Client;
use serde_json::Value;

const SEARCH_ENDPOINT: &str = "https://maps.googleapis.com/maps/api/js/GeoPhotoService.SingleImageSearch";

/// Build the search URL for a given GPS coordinate.
fn make_search_url(lat: f64, lon: f64) -> String {
    // This constructs the undocumented Google endpoint URL
    format!(
        "{SEARCH_ENDPOINT}?pb=!1m5!1sapiv3!5sUS!11m2!1m1!1b0!2m4!1m2!3d{lat}!4d{lon}!2d50!3m18!2m2!1sen!2sUS!9m1!1e2!11m12!1m3!1e2!2b1!3e2!1m3!1e3!2b1!3e2!1m3!1e10!2b1!3e2!4m6!1e1!1e2!1e3!1e4!1e8!1e6&callback=callbackfunc"
    )
}

/// Extract panoramas from Google's JavaScript callback response.
fn extract_panoramas(text: &str) -> Result<Vec<Panorama>> {
    // Check if the search returned no images
    if text.contains("Search returned no images") {
        return Ok(Vec::new());
    }

    // Extract JSON from the JavaScript callback: callbackfunc(JSON_DATA)
    let re = Regex::new(r"callbackfunc\((.*)\)").unwrap();
    let json_str = re
        .captures(text)
        .and_then(|cap| cap.get(1))
        .ok_or_else(|| StreetViewError::ParseError("Could not extract JSON from response".to_string()))?
        .as_str();

    let data: Value = serde_json::from_str(json_str)
        .map_err(|e| StreetViewError::ParseError(format!("JSON parse error: {e}")))?;

    // Navigate to the panorama data: data[1][5][0][3][0]
    let pano_array = data
        .get(1)
        .and_then(|v| v.get(5))
        .and_then(|v| v.get(0))
        .and_then(|v| v.get(3))
        .and_then(|v| v.get(0))
        .and_then(|v| v.as_array())
        .ok_or_else(|| StreetViewError::InvalidResponse("Panorama data not found".to_string()))?;

    // Get dates: data[1][5][0][8]
    // Each date is structured as: [[something], [year, month]]
    // Format as "YYYY-MM" like Python does
    let dates = data
        .get(1)
        .and_then(|v| v.get(5))
        .and_then(|v| v.get(0))
        .and_then(|v| v.get(8))
        .and_then(|v| v.as_array())
        .map(|arr| {
            // Dates need to be reversed to align with panoramas
            let mut dates: Vec<Option<String>> = arr
                .iter()
                .filter_map(|d| {
                    // Each date is an array like [[...], [year, month]]
                    let date_arr = d.as_array()?;
                    let date_info = date_arr.get(1)?.as_array()?;
                    let year = date_info.first()?.as_i64()?;
                    let month = date_info.get(1)?.as_i64()?;
                    Some(format!("{year}-{month:02}"))
                })
                .map(Some)
                .collect();
            dates.reverse();
            dates
        })
        .unwrap_or_default();

    // Reverse panoramas to match Python behavior
    // Google returns them in reverse chronological order for some locations,
    // so we flip to make the 0th panorama align with 0th date
    let pano_array: Vec<&Value> = pano_array.iter().rev().collect();

    let mut panoramas = Vec::new();

    for (idx, pano_data) in pano_array.iter().enumerate() {
        let pano_arr = pano_data
            .as_array()
            .ok_or_else(|| StreetViewError::ParseError("Invalid panorama format".to_string()))?;

        // Extract fields from the array
        let pano_id = pano_arr.first()
            .and_then(|v| v.get(1))
            .and_then(|v| v.as_str())
            .ok_or_else(|| StreetViewError::ParseError("Missing pano_id".to_string()))?
            .to_string();

        // GPS coordinates are in pano_arr[2][0]
        let coords = pano_arr
            .get(2)
            .and_then(|v| v.get(0))
            .and_then(|v| v.as_array())
            .ok_or_else(|| StreetViewError::ParseError("Missing coordinates".to_string()))?;

        let lat = coords
            .get(2)
            .and_then(|v| v.as_f64())
            .ok_or_else(|| StreetViewError::ParseError("Missing latitude".to_string()))?;

        let lon = coords
            .get(3)
            .and_then(|v| v.as_f64())
            .ok_or_else(|| StreetViewError::ParseError("Missing longitude".to_string()))?;

        // Camera orientation is in pano_arr[2][2] (not pano_arr[2][0]!)
        let orientation = pano_arr
            .get(2)
            .and_then(|v| v.get(2))
            .and_then(|v| v.as_array());

        let heading = orientation
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let pitch = orientation
            .and_then(|arr| arr.get(1))
            .and_then(|v| v.as_f64());

        let roll = orientation
            .and_then(|arr| arr.get(2))
            .and_then(|v| v.as_f64());

        // Elevation is in pano_arr[3][0]
        let elevation = pano_arr
            .get(3)
            .and_then(|v| v.get(0))
            .and_then(|v| v.as_f64());

        // Get date for this panorama
        let date = dates.get(idx).and_then(|d| d.clone());

        panoramas.push(Panorama {
            pano_id,
            lat,
            lon,
            heading,
            pitch,
            roll,
            date,
            elevation,
        });
    }

    Ok(panoramas)
}

/// Search for panoramas at a given GPS coordinate.
pub async fn search_panoramas(client: &Client, lat: f64, lon: f64) -> Result<Vec<Panorama>> {
    let url = make_search_url(lat, lon);
    let response = client.get(&url).send().await?;
    let text = response.text().await?;
    extract_panoramas(&text)
}

/// Parse a Google Maps URL to extract GPS coordinates and panorama ID.
pub fn parse_url(url: &str) -> Result<(f64, f64, Option<String>)> {
    // Google Maps URLs can have various formats:
    // - https://www.google.com/maps/@LAT,LON,zoom
    // - https://www.google.com/maps/@LAT,LON,zoom!data=...!1sPANO_ID...
    //  https://www.google.com/maps/...!8m2!3dLAT!4dLON...

    // Try to extract lat/lon using regex
    let lat_lon_re = Regex::new(r"@(-?\d+\.?\d*),(-?\d+\.?\d*)").unwrap();
    let coords = lat_lon_re
        .captures(url)
        .and_then(|cap| {
            let lat = cap.get(1)?.as_str().parse::<f64>().ok()?;
            let lon = cap.get(2)?.as_str().parse::<f64>().ok()?;
            Some((lat, lon))
        })
        .or_else(|| {
            // Try alternative format: !3dLAT!4dLON
            let alt_re = Regex::new(r"!3d(-?\d+\.?\d*)!4d(-?\d+\.?\d*)").unwrap();
            alt_re.captures(url).and_then(|cap| {
                let lat = cap.get(1)?.as_str().parse::<f64>().ok()?;
                let lon = cap.get(2)?.as_str().parse::<f64>().ok()?;
                Some((lat, lon))
            })
        })
        .ok_or_else(|| StreetViewError::InvalidUrl)?;

    // Try to extract panorama ID
    let pano_id_re = Regex::new(r"!1s([a-zA-Z0-9_-]+)").unwrap();
    let pano_id = pano_id_re
        .captures(url)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().to_string());

    Ok((coords.0, coords.1, pano_id))
}

/// Search for panoramas from a Google Maps URL.
pub async fn search_panoramas_url(client: &Client, url: &str) -> Result<Vec<Panorama>> {
    let (lat, lon, _) = parse_url(url)?;
    search_panoramas(client, lat, lon).await
}

/// Find the exact panorama shown in a Google Maps URL.
pub async fn search_panoramas_url_exact(
    client: &Client,
    url: &str,
) -> Result<Option<Panorama>> {
    let (lat, lon, pano_id) = parse_url(url)?;

    if let Some(target_id) = pano_id {
        let panos = search_panoramas(client, lat, lon).await?;
        Ok(panos.into_iter().find(|p| p.pano_id == target_id))
    } else {
        // No panorama ID in URL, return the first result
        let panos = search_panoramas(client, lat, lon).await?;
        Ok(panos.into_iter().next())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_url() {
        let url = "https://www.google.com/maps/@41.8982208,12.4764804,17z";
        let (lat, lon, _) = parse_url(url).unwrap();
        assert!((lat - 41.8982208).abs() < 0.0001);
        assert!((lon - 12.4764804).abs() < 0.0001);
    }

    #[test]
    fn test_parse_url_with_pano() {
        let url = "https://www.google.com/maps/@41.8982208,12.4764804,3a,75y,90t/data=!3m6!1e1!3m4!1sAF1QipNRA!2e0!7i16384!8i8192!1sABCD123";
        let (lat, lon, pano_id) = parse_url(url).unwrap();
        assert!((lat - 41.8982208).abs() < 0.0001);
        assert!((lon - 12.4764804).abs() < 0.0001);
        assert!(pano_id.is_some());
    }
}
