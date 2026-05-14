use crate::models::weather::{FlightRules, Metar, WeatherError};
use crate::web::api_client::ApiClient;
use std::collections::HashMap;
use std::time::Duration;
use web_time::Instant;

const CACHE_DURATION: Duration = Duration::from_secs(60 * 15); // 15 minutes

struct CachedEntry {
    rules: Option<String>,
    valid_until: Instant,
    fetched_at: Instant,
}

/// WASM-compatible weather service that fetches METAR data from the backend API.
#[derive(Clone)]
pub struct WebWeatherService {
    api_client: ApiClient,
    memory_cache: std::rc::Rc<std::cell::RefCell<HashMap<String, CachedEntry>>>,
}

impl WebWeatherService {
    pub fn new(api_client: ApiClient) -> Self {
        Self {
            api_client,
            memory_cache: std::rc::Rc::new(std::cell::RefCell::new(HashMap::new())),
        }
    }

    /// Returns cached flight rules for a station without making a network request.
    pub fn get_cached_flight_rules(&self, station_id: &str) -> Option<(FlightRules, Instant)> {
        let cache = self.memory_cache.borrow();
        let entry = cache.get(station_id)?;
        if Instant::now() < entry.valid_until {
            entry
                .rules
                .as_deref()
                .map(FlightRules::from)
                .map(|r| (r, entry.fetched_at))
        } else {
            None
        }
    }

    /// Spawns an async fetch for METAR data, storing results in the cache when done.
    ///
    /// The result is delivered via the provided callback on completion. The UI should
    /// call `request_repaint()` from inside the callback if needed.
    pub fn fetch_metar_async<F>(&self, station: &str, on_complete: F)
    where
        F: FnOnce(Result<Metar, WeatherError>) + 'static,
    {
        let client = self.api_client.clone();
        let cache = self.memory_cache.clone();
        let station_owned = station.to_string();

        wasm_bindgen_futures::spawn_local(async move {
            let result = client
                .fetch_metar(&station_owned)
                .await
                .map_err(|e| WeatherError::Request(e));

            if let Ok(ref metar) = result {
                let mut c = cache.borrow_mut();
                c.insert(
                    station_owned,
                    CachedEntry {
                        rules: metar.flight_rules.clone(),
                        valid_until: Instant::now() + CACHE_DURATION,
                        fetched_at: Instant::now(),
                    },
                );
            }

            on_complete(result);
        });
    }

    pub fn api_key(&self) -> &str {
        "" // no local api key storage in WASM — managed by the backend
    }

    pub fn update_api_key(&mut self, _key: String) {
        // no-op: api key is stored server-side for the web version
    }
}
