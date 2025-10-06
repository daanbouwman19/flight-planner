//! This module handles fetching and parsing weather data from the AVWX API.

use crate::errors::Error;
use reqwest::Client;
use serde::Deserialize;

pub const AVWX_API_URL: &str = "https://avwx.rest";
const AVWX_API_PATH: &str = "/api/metar/";

#[derive(Deserialize, Debug, Clone)]
pub struct ValueU32 {
    pub value: u32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ValueF32 {
    pub value: f32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Metar {
    pub wind_speed: Option<ValueU32>,
    pub visibility: Option<ValueF32>,
    pub flight_rules: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ApiResponse {
    pub sample: Option<Metar>,
}

/// Fetches and parses METAR data for a given airport ICAO code.
///
/// # Arguments
///
/// * `icao` - The ICAO code of the airport.
/// * `client` - The `reqwest::Client` to use for the request.
///
/// # Returns
///
/// A `Result` containing the `Metar` data on success, or an `Error` on failure.
pub async fn get_weather_data(
    base_url: &str,
    icao: &str,
    client: &Client,
    api_key: &str,
) -> Result<Metar, Error> {
    let url = format!("{}{}{}", base_url, AVWX_API_PATH, icao);
    let response = client
        .get(&url)
        .header("Authorization", format!("BEARER {}", api_key))
        .send()
        .await?;
    let raw_text = response.text().await?;
    log::info!("AVWX API Response for {}: {}", icao, raw_text);
    let api_response: ApiResponse = serde_json::from_str(&raw_text)?;
    if let Some(metar) = api_response.sample {
        Ok(metar)
    } else {
        Err(Error::Other(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No weather data in API response",
        )))
    }
}