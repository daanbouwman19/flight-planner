use serde::Deserialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Wind {
    pub speed_kts: f64,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Metar {
    pub raw: String,
    pub wind: Wind,
}

struct CacheEntry {
    metar: Metar,
    timestamp: Instant,
}

pub struct WeatherApi {
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    cache: Arc<Mutex<HashMap<String, CacheEntry>>>,
}

impl WeatherApi {
    pub fn new(api_key: String) -> Self {
        Self::new_with_url(api_key, "https://avwx.rest".to_string())
    }

    pub fn new_with_url(api_key: String, base_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            base_url,
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get_metar(&self, icao: &str) -> Result<Metar, reqwest::Error> {
        // Scope the lock to release it before the await point
        {
            let cache = self.cache.lock().unwrap();
            if let Some(entry) = cache.get(icao) {
                if entry.timestamp.elapsed() < Duration::from_secs(900) {
                    return Ok(entry.metar.clone());
                }
            }
        } // lock is dropped here

        let url = format!("{}/api/metar/{}", self.base_url, icao);
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        let metar: Metar = response.json().await?;

        // Lock again to insert
        {
            let mut cache = self.cache.lock().unwrap();
            cache.insert(
                icao.to_string(),
                CacheEntry {
                    metar: metar.clone(),
                    timestamp: Instant::now(),
                },
            );
        }
        Ok(metar)
    }
}