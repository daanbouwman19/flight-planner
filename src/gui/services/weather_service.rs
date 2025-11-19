use crate::models::weather::{Metar, WeatherError};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

const CACHE_FILE_NAME: &str = "metar_cache.json";

const CACHE_DURATION: Duration = Duration::from_secs(60 * 15); // 15 minutes

type CacheEntry = Arc<Mutex<Option<(Metar, SystemTime)>>>;

#[derive(serde::Serialize, serde::Deserialize)]
struct CachedMetar {
    metar: Metar,
    timestamp: SystemTime,
}

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
            cache: Arc::new(Mutex::new(Self::load_cache())),
        }
    }

    fn get_cache_path() -> PathBuf {
        let mut path = dirs::cache_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("Flight Planner");
        fs::create_dir_all(&path).ok();
        path.push(CACHE_FILE_NAME);
        path
    }

    fn load_cache() -> HashMap<String, CacheEntry> {
        let path = Self::get_cache_path();
        if let Ok(file) = File::open(path) {
            let reader = BufReader::new(file);
            if let Ok(cached_data) = serde_json::from_reader::<_, HashMap<String, CachedMetar>>(reader) {
                return cached_data
                    .into_iter()
                    .map(|(k, v)| (k, Arc::new(Mutex::new(Some((v.metar, v.timestamp))))))
                    .collect();
            }
        }
        HashMap::new()
    }

    pub fn save_cache(&self) {
        let path = Self::get_cache_path();
        if let Ok(file) = File::create(path) {
            let writer = BufWriter::new(file);
            let cache_lock = self.cache.lock().unwrap();
            let data_to_save: HashMap<String, CachedMetar> = cache_lock
                .iter()
                .filter_map(|(k, v)| {
                    let entry_lock = v.lock().unwrap();
                    entry_lock.as_ref().map(|(metar, timestamp)| {
                        (
                            k.clone(),
                            CachedMetar {
                                metar: metar.clone(),
                                timestamp: *timestamp,
                            },
                        )
                    })
                })
                .collect();
            serde_json::to_writer(writer, &data_to_save).ok();
        }
    }

    #[cfg(test)]
    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }

    pub fn fetch_metar(&self, station: &str) -> Result<Metar, WeatherError> {
        self.fetch_metar_internal(station, true)
    }

    pub fn fetch_metar_no_save(&self, station: &str) -> Result<Metar, WeatherError> {
        self.fetch_metar_internal(station, false)
    }

    fn fetch_metar_internal(&self, station: &str, save: bool) -> Result<Metar, WeatherError> {
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

        // Check cache first
        {
            let entry = station_lock
                .lock()
                .map_err(|_| WeatherError::Request("Station lock failed".to_string()))?;

            if let Some((metar, timestamp)) = &*entry
                && timestamp.elapsed().unwrap_or(Duration::MAX) < CACHE_DURATION
            {
                return Ok(metar.clone());
            }
        } // Lock is dropped here

        // Perform network request without holding the lock
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

        // Re-acquire lock to update cache
        {
            let mut entry = station_lock
                .lock()
                .map_err(|_| WeatherError::Request("Station lock failed".to_string()))?;
            *entry = Some((metar.clone(), SystemTime::now()));
        }
        
        if save {
            // Save cache in a separate thread to avoid blocking
            let service_clone = self.clone();
            std::thread::spawn(move || {
                service_clone.save_cache();
            });
        }

        Ok(metar)
    }
    pub fn get_cached_flight_rules(&self, station: &str) -> Option<String> {
        let station_lock = self.cache.lock().ok()?;
        let entry = station_lock.get(station)?;
        let entry_lock = entry.lock().ok()?;

        if let Some((metar, timestamp)) = &*entry_lock {
            if timestamp.elapsed().unwrap_or(Duration::MAX) < CACHE_DURATION {
                return metar.flight_rules.clone();
            }
        }
        None
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
