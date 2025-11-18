use crate::models::weather::{Metar, WeatherError};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const CACHE_DURATION: Duration = Duration::from_secs(60 * 15); // 15 minutes

type CacheEntry = Arc<Mutex<Option<(Metar, Instant)>>>;

#[derive(Clone)]
pub struct WeatherService {
    api_key: String,
    base_url: String,
    client: reqwest::blocking::Client,
    cache: Arc<Mutex<HashMap<String, CacheEntry>>>,
}

impl WeatherService {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://avwx.rest".to_string(),
            client: reqwest::blocking::Client::new(),
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    #[cfg(test)]
    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }

    pub fn fetch_metar(&self, station: &str) -> Result<Metar, WeatherError> {
        let station_lock = {
            let mut cache = self
                .cache
                .lock()
                .map_err(|_| WeatherError::Request("Cache lock failed".to_string()))?;
            cache
                .entry(station.to_string())
                .or_insert_with(|| Arc::new(Mutex::new(None)))
                .clone()
        };

        let mut entry = station_lock
            .lock()
            .map_err(|_| WeatherError::Request("Station lock failed".to_string()))?;

        if let Some((metar, timestamp)) = &*entry
            && timestamp.elapsed() < CACHE_DURATION
        {
            return Ok(metar.clone());
        }

        let url = format!("{}/api/metar/{}", self.base_url, station);
        let response = self
            .client
            .get(&url)
            .header("Authorization", &self.api_key)
            .send()
            .map_err(|e| WeatherError::Request(e.to_string()))?;

        if response.status() == reqwest::StatusCode::NO_CONTENT {
            return Err(WeatherError::NoData);
        }

        if !response.status().is_success() {
            if response.status() == reqwest::StatusCode::BAD_REQUEST {
                return Err(WeatherError::StationNotFound);
            }
            return Err(WeatherError::Api(response.status().to_string()));
        }

        let body = response
            .text()
            .map_err(|e| WeatherError::Parse(e.to_string()))?;
        if body.trim().is_empty() {
            return Err(WeatherError::NoData);
        }

        let metar: Metar = serde_json::from_str(&body).map_err(|e| {
            WeatherError::Parse(format!("Failed to parse METAR JSON: {}. Body: {}", e, body))
        })?;

        *entry = Some((metar.clone(), Instant::now()));

        Ok(metar)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::prelude::*;
    use serde_json::json;

    #[test]
    fn test_fetch_metar_success() {
        let server = MockServer::start();
        let metar_mock = server.mock(|when, then| {
            when.method(GET).path("/api/metar/KMCO");
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!({
                    "raw": "KMCO 181253Z 12006KT 10SM FEW030 SCT045 BKN060 22/17 A3012 RMK AO2 SLP198 T02220167",
                    "san": "KMCO",
                    "flight_rules": "VFR",
                }));
        });

        let weather_service =
            WeatherService::new("test_api_key".to_string()).with_base_url(server.base_url());
        let result = weather_service.fetch_metar("KMCO");

        metar_mock.assert();
        assert!(result.is_ok());
        let metar = result.unwrap();
        assert_eq!(metar.san, Some("KMCO".to_string()));
        assert_eq!(metar.flight_rules, Some("VFR".to_string()));
    }

    #[test]
    fn test_fetch_metar_caching() {
        let server = MockServer::start();
        let metar_mock = server.mock(|when, then| {
            when.method(GET).path("/api/metar/KLAX");
            then.status(200)
                .header("content-type", "application/json")
                .json_body(json!({"san": "KLAX", "raw": "KLAX raw metar"}));
        });

        let weather_service =
            WeatherService::new("test_api_key".to_string()).with_base_url(server.base_url());

        let result1 = weather_service.fetch_metar("KLAX");
        metar_mock.assert();
        assert!(result1.is_ok());

        let result2 = weather_service.fetch_metar("KLAX");
        metar_mock.assert_calls(1);
        assert!(result2.is_ok());
        assert_eq!(result1.unwrap().raw, result2.unwrap().raw);
    }

    #[test]
    fn test_fetch_metar_station_not_found() {
        let server = MockServer::start();
        let error_mock = server.mock(|when, then| {
            when.method(GET).path("/api/metar/INVALID");
            then.status(400);
        });

        let weather_service =
            WeatherService::new("test_api_key".to_string()).with_base_url(server.base_url());
        let result = weather_service.fetch_metar("INVALID");

        error_mock.assert();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), WeatherError::StationNotFound));
    }

    #[test]
    fn test_fetch_metar_no_data() {
        let server = MockServer::start();
        let no_content_mock = server.mock(|when, then| {
            when.method(GET).path("/api/metar/NODATA");
            then.status(204);
        });

        let weather_service =
            WeatherService::new("test_api_key".to_string()).with_base_url(server.base_url());
        let result = weather_service.fetch_metar("NODATA");

        no_content_mock.assert();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), WeatherError::NoData));
    }

    #[test]
    fn test_fetch_metar_api_error() {
        let server = MockServer::start();
        let error_mock = server.mock(|when, then| {
            when.method(GET).path("/api/metar/ERROR");
            then.status(500);
        });

        let weather_service =
            WeatherService::new("test_api_key".to_string()).with_base_url(server.base_url());
        let result = weather_service.fetch_metar("ERROR");

        error_mock.assert();
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), WeatherError::Api(_)));
    }
}
