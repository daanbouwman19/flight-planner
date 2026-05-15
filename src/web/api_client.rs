use crate::models::weather::Metar;
use crate::models::{
    Aircraft, Airport, FlightStatistics, HistoryPageResponse, RouteResponse, Runway,
};
use std::collections::HashMap;

/// Async HTTP client for communicating with the backend REST API.
#[derive(Clone)]
pub struct ApiClient {
    base_url: String,
    client: reqwest::Client,
}

impl ApiClient {
    pub fn new() -> Self {
        // reqwest on WASM requires absolute URLs; derive origin from window.location
        let base_url = web_sys::window()
            .and_then(|w| w.location().origin().ok())
            .unwrap_or_default();
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }

    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    fn url(&self, path: &str) -> String {
        if self.base_url.is_empty() {
            format!("/api{path}")
        } else {
            format!("{}/api{path}", self.base_url)
        }
    }

    pub async fn fetch_aircraft(&self) -> Result<Vec<Aircraft>, String> {
        self.client
            .get(self.url("/aircraft"))
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<Vec<Aircraft>>()
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn fetch_airports(&self) -> Result<Vec<Airport>, String> {
        self.client
            .get(self.url("/airports"))
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<Vec<Airport>>()
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn search_airports(&self, query: &str, limit: usize) -> Result<Vec<Airport>, String> {
        let url = self.url(&format!(
            "/airports/search?q={}&limit={}",
            urlencoding::encode(query),
            limit
        ));
        self.client
            .get(url)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<Vec<Airport>>()
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn random_airports(&self, n: usize) -> Result<Vec<Airport>, String> {
        self.client
            .get(self.url(&format!("/airports/random?n={n}")))
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<Vec<Airport>>()
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn airport_by_icao(&self, icao: &str) -> Result<Airport, String> {
        let resp = self
            .client
            .get(self.url(&format!("/airports/by-icao/{icao}")))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("API error: {}", resp.status()));
        }
        resp.json::<Airport>().await.map_err(|e| e.to_string())
    }

    pub async fn fetch_runways(&self) -> Result<HashMap<i32, Vec<Runway>>, String> {
        let raw: HashMap<String, Vec<Runway>> = self
            .client
            .get(self.url("/runways"))
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;
        Ok(raw
            .into_iter()
            .filter_map(|(k, v)| k.parse::<i32>().ok().map(|id| (id, v)))
            .collect())
    }

    pub async fn fetch_history(
        &self,
        limit: usize,
        offset: usize,
    ) -> Result<HistoryPageResponse, String> {
        self.client
            .get(self.url(&format!("/history?limit={limit}&offset={offset}")))
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<HistoryPageResponse>()
            .await
            .map_err(|e| e.to_string())
    }

    /// Fetches a page of airports for browsing (`GET /api/airports?offset=N&limit=N`).
    ///
    /// Requests `limit + 1` items; if the response contains more than `limit`
    /// entries the caller knows another page exists (`has_more = true`).
    pub async fn fetch_airports_page(
        &self,
        offset: usize,
        limit: usize,
    ) -> Result<(Vec<Airport>, bool), String> {
        let mut airports: Vec<Airport> = self
            .client
            .get(self.url(&format!("/airports?offset={offset}&limit={}", limit + 1)))
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<Vec<Airport>>()
            .await
            .map_err(|e| e.to_string())?;
        let has_more = airports.len() > limit;
        airports.truncate(limit);
        Ok((airports, has_more))
    }

    pub async fn fetch_statistics(&self) -> Result<FlightStatistics, String> {
        self.client
            .get(self.url("/statistics"))
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<FlightStatistics>()
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn fetch_metar(&self, icao: &str) -> Result<Metar, String> {
        self.client
            .get(self.url(&format!("/weather/{icao}")))
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<Metar>()
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn toggle_flown(&self, aircraft_id: i32) -> Result<(), String> {
        let resp = self
            .client
            .put(self.url(&format!("/aircraft/{aircraft_id}/flown")))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(format!("API error: {}", resp.status()))
        }
    }

    pub async fn reset_flown(&self) -> Result<(), String> {
        let resp = self
            .client
            .post(self.url("/aircraft/reset"))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(format!("API error: {}", resp.status()))
        }
    }

    pub async fn add_history(
        &self,
        aircraft_id: i32,
        departure_icao: &str,
        arrival_icao: &str,
    ) -> Result<(), String> {
        let body = serde_json::json!({
            "aircraft_id": aircraft_id,
            "departure_icao": departure_icao,
            "arrival_icao": arrival_icao
        });
        let resp = self
            .client
            .post(self.url("/history"))
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(format!("API error: {}", resp.status()))
        }
    }

    pub async fn generate_routes(
        &self,
        mode: &str,
        aircraft_id: Option<i32>,
        departure_icao: Option<&str>,
    ) -> Result<Vec<RouteResponse>, String> {
        let body = serde_json::json!({
            "mode": mode,
            "aircraft_id": aircraft_id,
            "departure_icao": departure_icao,
        });
        self.client
            .post(self.url("/routes"))
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<Vec<RouteResponse>>()
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn route_from_history(
        &self,
        aircraft_id: i32,
        departure_icao: &str,
        arrival_icao: &str,
    ) -> Result<RouteResponse, String> {
        let body = serde_json::json!({
            "aircraft_id": aircraft_id,
            "departure_icao": departure_icao,
            "arrival_icao": arrival_icao,
        });
        let resp = self
            .client
            .post(self.url("/routes/from-history"))
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("API error: {}", resp.status()));
        }
        resp.json::<RouteResponse>()
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn fetch_settings(&self) -> Result<HashMap<String, String>, String> {
        self.client
            .get(self.url("/settings"))
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json::<HashMap<String, String>>()
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn save_setting(&self, key: &str, value: &str) -> Result<(), String> {
        let body = serde_json::json!({ "key": key, "value": value });
        let resp = self
            .client
            .post(self.url("/settings"))
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(format!("API error: {}", resp.status()))
        }
    }
}

impl Default for ApiClient {
    fn default() -> Self {
        Self::new()
    }
}
