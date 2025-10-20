use crate::modules::weather::{get_metar, Metar};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const CACHE_DURATION: Duration = Duration::from_secs(900); // 15 minutes

#[derive(Clone)]
pub struct WeatherService {
    cache: Arc<Mutex<HashMap<String, (Metar, Instant)>>>,
    api_key: String,
}

impl WeatherService {
    pub fn new(api_key: String) -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            api_key,
        }
    }

    pub async fn fetch_metar(&self, icao: &str) -> Option<Metar> {
        // Scope the cache lock to release it before the await
        {
            let cache = self.cache.lock().unwrap();
            if let Some((metar, timestamp)) = cache.get(icao) {
                if timestamp.elapsed() < CACHE_DURATION {
                    return Some(metar.clone());
                }
            }
        }

        // Fetch outside the lock
        if let Ok(metar) = get_metar(icao, "https://avwx.rest", &self.api_key).await {
            // Re-lock to insert the new value
            let mut cache = self.cache.lock().unwrap();
            cache.insert(icao.to_string(), (metar.clone(), Instant::now()));
            Some(metar)
        } else {
            None
        }
    }
}
