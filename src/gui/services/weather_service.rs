use crate::models::weather::Metar;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const CACHE_DURATION: Duration = Duration::from_secs(60 * 15); // 15 minutes

#[derive(Clone)]
pub struct WeatherService {
    api_key: String,
    cache: Arc<Mutex<HashMap<String, (Metar, Instant)>>>,
}

impl WeatherService {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn fetch_metar(&self, station: &str) -> Result<Metar, String> {
        // Check cache first
        {
            let mut cache = self.cache.lock().map_err(|e| e.to_string())?;
            if let Some((metar, timestamp)) = cache.get(station) {
                if timestamp.elapsed() < CACHE_DURATION {
                    return Ok(metar.clone());
                } else {
                    cache.remove(station);
                }
            }
        }

        // Fetch from API
        let url = format!("https://avwx.rest/api/metar/{}", station);
        let client = reqwest::blocking::Client::new();
        let response = client
            .get(&url)
            .header("Authorization", &self.api_key)
            .send()
            .map_err(|e| e.to_string())?;

        if response.status() == reqwest::StatusCode::NO_CONTENT {
            return Err("No METAR data available".to_string());
        }

        if !response.status().is_success() {
            if response.status() == reqwest::StatusCode::BAD_REQUEST {
                return Err("Station not found".to_string());
            }
            return Err(format!("API Error: {}", response.status()));
        }

        let body = response.text().map_err(|e| e.to_string())?;
        if body.trim().is_empty() {
            return Err("No METAR data available".to_string());
        }

        let metar: Metar = serde_json::from_str(&body)
            .map_err(|e| format!("Failed to parse METAR JSON: {}. Body: {}", e, body))?;

        // Update cache
        {
            let mut cache = self.cache.lock().map_err(|e| e.to_string())?;
            cache.insert(station.to_string(), (metar.clone(), Instant::now()));
        }

        Ok(metar)
    }
}
