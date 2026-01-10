use crate::database::DatabasePool;
use crate::models::weather::{FlightRules, Metar, WeatherError};
use crate::schema::metar_cache;
use diesel::prelude::*;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

const CACHE_DURATION: Duration = Duration::from_secs(60 * 15); // 15 minutes
const DB_CACHE_FETCH_TIME_OFFSET: Duration = Duration::from_secs(3600);

struct CachedFlightRules {
    rules: Option<String>,
    valid_until: Instant,
    fetched_at: Instant,
}

/// Memory cache: station -> CachedFlightRules
type FlightRulesCache = HashMap<String, CachedFlightRules>;

#[derive(Queryable, Insertable)]
#[diesel(table_name = metar_cache)]
struct MetarCacheEntry {
    station: String,
    raw: String,
    flight_rules: Option<String>,
    observation_time: Option<String>,
    observation_dt: Option<String>,
    fetched_at: String,
}

#[derive(Clone)]
pub struct WeatherService {
    api_key: String,
    base_url: String,
    client: reqwest::blocking::Client,
    pool: DatabasePool,
    memory_cache: Arc<RwLock<FlightRulesCache>>,
}

impl WeatherService {
    pub fn new(api_key: String, pool: DatabasePool) -> Self {
        Self {
            api_key,
            base_url: "https://avwx.rest".to_string(),
            client: reqwest::blocking::Client::new(),
            pool,
            memory_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }

    pub fn update_api_key(&mut self, api_key: String) {
        self.api_key = api_key;
    }

    pub fn fetch_metar(&self, station: &str) -> Result<Metar, WeatherError> {
        self.fetch_metar_internal(station)
    }

    fn fetch_metar_internal(&self, station_id: &str) -> Result<Metar, WeatherError> {
        // 1. Check DB cache
        if let Ok(mut conn) = self.pool.airport_pool.get() {
            use crate::schema::metar_cache::dsl::{metar_cache, station};

            if let Ok(entry) = metar_cache
                .filter(station.eq(station_id))
                .first::<MetarCacheEntry>(&mut conn)
                && let Ok(fetched_time) = chrono::DateTime::parse_from_rfc3339(&entry.fetched_at)
            {
                let now = chrono::Utc::now();
                let age = now.signed_duration_since(fetched_time).num_seconds();
                if age < CACHE_DURATION.as_secs() as i64 {
                    // Update memory cache on successful DB hit
                    if let Ok(mut cache) = self.memory_cache.write() {
                        let remaining_ttl = CACHE_DURATION.as_secs() - age as u64;
                        cache.insert(
                            station_id.to_string(),
                            CachedFlightRules {
                                rules: entry.flight_rules.clone(),
                                valid_until: Instant::now() + Duration::from_secs(remaining_ttl),
                                fetched_at: Instant::now()
                                    .checked_sub(DB_CACHE_FETCH_TIME_OFFSET)
                                    .or_else(|| {
                                        Instant::now().checked_sub(Duration::from_secs(600))
                                    })
                                    .or_else(|| Instant::now().checked_sub(Duration::from_secs(60)))
                                    .or_else(|| {
                                        Instant::now().checked_sub(Duration::from_millis(600))
                                    })
                                    .unwrap_or_else(Instant::now),
                            },
                        );
                    }

                    return Ok(Metar {
                        raw: Some(entry.raw),
                        flight_rules: entry.flight_rules,
                        san: Some(entry.station),
                        time: Some(crate::models::weather::MetarTime {
                            repr: entry.observation_time,
                            dt: entry.observation_dt,
                        }),
                    });
                }
            }
        }

        // 2. Fetch from API
        let metar = self.call_avwx_api(station_id)?;

        // 3. Save to DB
        if let Ok(mut conn) = self.pool.airport_pool.get() {
            use crate::schema::metar_cache::dsl::metar_cache;

            let new_entry = MetarCacheEntry {
                station: station_id.to_string(),
                raw: metar.raw.clone().unwrap_or_default(),
                flight_rules: metar.flight_rules.clone(),
                observation_time: metar.time.as_ref().and_then(|t| t.repr.clone()),
                observation_dt: metar.time.as_ref().and_then(|t| t.dt.clone()),
                fetched_at: chrono::Utc::now().to_rfc3339(),
            };

            diesel::replace_into(metar_cache)
                .values(&new_entry)
                .execute(&mut conn)
                .map_err(|e| {
                    log::error!("Failed to save METAR to cache for {}: {}", station_id, e);
                    e
                })
                .ok();

            // Update memory cache
            if let Ok(mut cache) = self.memory_cache.write() {
                cache.insert(
                    station_id.to_string(),
                    CachedFlightRules {
                        rules: metar.flight_rules.clone(),
                        valid_until: Instant::now() + CACHE_DURATION,
                        fetched_at: Instant::now(), // Fresh from API
                    },
                );
            }
        }

        Ok(metar)
    }

    #[cfg(not(tarpaulin_include))]
    fn call_avwx_api(&self, station_id: &str) -> Result<Metar, WeatherError> {
        let url = format!("{}/api/metar/{}", self.base_url, station_id);
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
        Ok(metar)
    }

    pub fn get_cached_flight_rules(&self, station_id: &str) -> Option<(FlightRules, Instant)> {
        // 1. Check Memory Cache
        if let Ok(cache) = self.memory_cache.read()
            && let Some(entry) = cache.get(station_id)
            && Instant::now() < entry.valid_until
        {
            return entry
                .rules
                .as_deref()
                .map(FlightRules::from)
                .map(|rules| (rules, entry.fetched_at));
        }

        // 2. Check DB Cache
        if let Ok(mut conn) = self.pool.airport_pool.get() {
            use crate::schema::metar_cache::dsl::{metar_cache, station};

            if let Ok(entry) = metar_cache
                .filter(station.eq(station_id))
                .first::<MetarCacheEntry>(&mut conn)
                && let Ok(fetched_time) = chrono::DateTime::parse_from_rfc3339(&entry.fetched_at)
            {
                let now = chrono::Utc::now();
                let age = now.signed_duration_since(fetched_time).num_seconds();
                if age < CACHE_DURATION.as_secs() as i64 {
                    let flight_rules = entry.flight_rules.clone();
                    // Use checked_sub to avoid panic if system uptime < DB_CACHE_FETCH_TIME_OFFSET
                    // Try progressively smaller offsets to get a "past" time if uptime is short
                    let fetched_at = Instant::now()
                        .checked_sub(DB_CACHE_FETCH_TIME_OFFSET)
                        .or_else(|| Instant::now().checked_sub(Duration::from_secs(600)))
                        .or_else(|| Instant::now().checked_sub(Duration::from_secs(60)))
                        .or_else(|| Instant::now().checked_sub(Duration::from_millis(600)))
                        .unwrap_or_else(Instant::now);

                    // Populate memory cache
                    if let Ok(mut cache) = self.memory_cache.write() {
                        let remaining_ttl = CACHE_DURATION.as_secs() - age as u64;
                        cache.insert(
                            station_id.to_string(),
                            CachedFlightRules {
                                rules: flight_rules.clone(),
                                valid_until: Instant::now() + Duration::from_secs(remaining_ttl),
                                fetched_at,
                            },
                        );
                    }

                    return flight_rules
                        .as_deref()
                        .map(FlightRules::from)
                        .map(|rules| (rules, fetched_at));
                }
            }
        }
        None
    }
}
