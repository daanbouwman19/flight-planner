use crate::models::weather::{Metar, WeatherError};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const CACHE_DURATION: Duration = Duration::from_secs(60 * 15); // 15 minutes

type CacheEntry = Arc<Mutex<Option<(Metar, Instant)>>>;

#[derive(Clone)]
pub struct WeatherService {
    api_key: String,
    // Cache stores an Arc<Mutex<...>> for each station to allow per-station locking
    // This prevents the "thundering herd" problem where multiple threads fetch the same station
    cache: Arc<Mutex<HashMap<String, CacheEntry>>>,
}

impl WeatherService {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn fetch_metar(&self, station: &str) -> Result<Metar, WeatherError> {
        // Get or create the lock for this specific station
        let station_lock = {
            let mut cache = self.cache.lock().map_err(|_| WeatherError::Request("Cache lock failed".to_string()))?;
            cache
                .entry(station.to_string())
                .or_insert_with(|| Arc::new(Mutex::new(None)))
                .clone()
        };

        // Lock the specific station entry
        let mut entry = station_lock.lock().map_err(|_| WeatherError::Request("Station lock failed".to_string()))?;

        // Check if we have valid cached data
        #[allow(clippy::collapsible_if)]
        if let Some((metar, timestamp)) = &*entry {
            if timestamp.elapsed() < CACHE_DURATION {
                return Ok(metar.clone());
            }
        }

        // Fetch from API (holding the station lock, so others wait)
        let url = format!("https://avwx.rest/api/metar/{}", station);
        let client = reqwest::blocking::Client::new();
        let response = client
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

        let body = response.text().map_err(|e| WeatherError::Parse(e.to_string()))?;
        if body.trim().is_empty() {
            return Err(WeatherError::NoData);
        }

        let metar: Metar = serde_json::from_str(&body)
            .map_err(|e| WeatherError::Parse(format!("Failed to parse METAR JSON: {}. Body: {}", e, body)))?;

        // Update cache
        *entry = Some((metar.clone(), Instant::now()));

        Ok(metar)
    }
}
