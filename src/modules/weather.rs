//! This module handles fetching and parsing weather data from the AVWX API.

use serde::Deserialize;
use reqwest::Client;
use crate::errors::Error;

pub const AVWX_API_URL: &str = "https://avwx.rest";
const AVWX_API_PATH: &str = "/api/metar/";

#[derive(Deserialize, Debug, Clone)]
pub struct Wind {
    pub speed_kts: u32,
    pub direction: Option<u32>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Visibility {
    pub miles: f32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Metar {
    pub wind: Option<Wind>,
    pub visibility: Option<Visibility>,
    pub flight_rules: String,
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
        .header("Authorization", api_key)
        .send()
        .await?
        .json::<Metar>()
        .await?;
    Ok(response)
}