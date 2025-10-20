use crate::errors::Error;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Metar {
    pub raw: String,
    pub flight_rules: String,
    pub wind_dir: serde_json::Value,
    pub wind_speed: serde_json::Value,
    pub visibility: serde_json::Value,
    pub temperature: serde_json::Value,
    pub dewpoint: serde_json::Value,
    pub altimeter: serde_json::Value,
    pub clouds: Vec<serde_json::Value>,
    pub summary: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Taf {
    pub raw: String,
    pub start_time: String,
    pub end_time: String,
    pub forecasts: Vec<serde_json::Value>,
    pub summary: String,
}

/// Fetches METAR data for a given airport ICAO code.
///
/// # Arguments
///
/// * `icao` - The ICAO code of the airport.
/// * `base_url` - The base URL of the AVWX API (for testing).
///
/// # Returns
///
/// A `Result` containing the `Metar` data on success, or an `Error` on failure.
pub async fn get_metar(icao: &str, base_url: &str, api_key: &str) -> Result<Metar, Error> {
    let url = format!("{}/api/metar/{}", base_url, icao);

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await?;

    if response.status().is_success() {
        let metar = response.json::<Metar>().await?;
        Ok(metar)
    } else {
        Err(Error::Other(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!(
                "Failed to fetch METAR data: {}",
                response.status()
            ),
        )))
    }
}
