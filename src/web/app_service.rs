use crate::gui::components::toast::ToastKind;
use crate::gui::data::{ListItemAircraft, ListItemAirport, ListItemHistory, ListItemRoute};
use crate::gui::services;
use crate::gui::services::popup_service::DisplayMode;
use crate::models::FlightStatistics;
use crate::models::{Aircraft, Airport, HistoryItemResponse, RouteResponse};
use crate::web::api_client::ApiClient;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use std::sync::mpsc;

/// WASM-compatible application service.
///
/// Routing, statistics, history enrichment, history-to-route conversion, and
/// airport search are all delegated to the backend. The frontend no longer
/// downloads the global airport list; instead, `airport_cache` is populated by
/// `/api/airports/random` and `/api/airports/search` calls driven by user
/// interaction (dropdown typing, view selection).
#[derive(Clone)]
pub struct WebAppService {
    api_client: ApiClient,
    aircraft: Vec<Arc<Aircraft>>,
    aircraft_by_id: HashMap<i32, Arc<Aircraft>>,
    aircraft_items: Vec<ListItemAircraft>,
    /// Latest set of airports relevant to the current UI context (dropdown
    /// search result, random sample for an "Airports" view, etc.).
    airport_cache: Vec<Arc<Airport>>,
    /// Monotonic counter to discard out-of-order search responses.
    airport_search_generation: u64,
    route_items: Vec<ListItemRoute>,
    history_items: Vec<ListItemHistory>,
    settings: HashMap<String, String>,
    cached_statistics: Option<FlightStatistics>,
    toast_sender: Option<mpsc::Sender<(String, ToastKind)>>,
}

impl WebAppService {
    pub fn new(
        aircraft_raw: Vec<Aircraft>,
        history_raw: Vec<HistoryItemResponse>,
        settings: HashMap<String, String>,
        api_client: ApiClient,
        initial_routes: Vec<RouteResponse>,
        initial_statistics: Option<FlightStatistics>,
        initial_airports: Vec<Airport>,
    ) -> Self {
        let aircraft: Vec<Arc<Aircraft>> = aircraft_raw.into_iter().map(Arc::new).collect();

        let aircraft_by_id: HashMap<i32, Arc<Aircraft>> =
            aircraft.iter().map(|a| (a.id, a.clone())).collect();

        let aircraft_items = services::aircraft_service::transform_to_list_items(&aircraft);

        let route_items = initial_routes
            .into_iter()
            .filter_map(|r| route_response_to_list_item(r, &aircraft_by_id))
            .collect();

        let airport_cache: Vec<Arc<Airport>> = initial_airports.into_iter().map(Arc::new).collect();

        let history_items: Vec<ListItemHistory> = history_raw
            .into_iter()
            .map(|h| ListItemHistory {
                id: h.id.to_string(),
                departure_info: format!("{} ({})", h.departure_name, h.departure_icao),
                departure_icao: h.departure_icao,
                arrival_info: format!("{} ({})", h.arrival_name, h.arrival_icao),
                arrival_icao: h.arrival_icao,
                aircraft_name: h.aircraft_name,
                aircraft_id: h.aircraft_id,
                date: h.date,
            })
            .collect();

        Self {
            api_client,
            aircraft,
            aircraft_by_id,
            aircraft_items,
            airport_cache,
            airport_search_generation: 0,
            route_items,
            history_items,
            settings,
            cached_statistics: initial_statistics,
            toast_sender: None,
        }
    }

    pub fn set_toast_sender(&mut self, sender: mpsc::Sender<(String, ToastKind)>) {
        self.toast_sender = Some(sender);
    }

    // --- Data Access ---

    pub fn airports(&self) -> &[Arc<Airport>] {
        &self.airport_cache
    }

    pub fn aircraft(&self) -> &[Arc<Aircraft>] {
        &self.aircraft
    }

    pub fn route_items(&self) -> &[ListItemRoute] {
        &self.route_items
    }

    pub fn clear_route_items(&mut self) {
        self.route_items.clear();
    }

    pub fn set_route_items(&mut self, mut routes: Vec<ListItemRoute>) {
        let now = web_time::Instant::now();
        for route in routes.iter_mut() {
            route.created_at = now;
        }
        self.route_items = routes;
    }

    pub fn append_route_items(&mut self, mut new_routes: Vec<ListItemRoute>) {
        let now = web_time::Instant::now();
        for route in new_routes.iter_mut() {
            route.created_at = now;
        }
        self.route_items.extend(new_routes);
    }

    pub fn history_items(&self) -> &[ListItemHistory] {
        &self.history_items
    }

    pub fn generate_airport_items(&self) -> Vec<ListItemAirport> {
        services::airport_service::transform_to_list_items(&self.airport_cache)
    }

    pub fn aircraft_items(&self) -> &[ListItemAircraft] {
        &self.aircraft_items
    }

    // --- Airport cache management ---

    /// Reserves a new search generation. Spawned requests should pass this
    /// value back so stale responses can be discarded by `apply_airport_search_result`.
    pub fn begin_airport_search(&mut self) -> u64 {
        self.airport_search_generation += 1;
        self.airport_search_generation
    }

    /// Applies a search result, ignoring responses older than the latest request.
    pub fn apply_airport_search_result(&mut self, generation: u64, airports: Vec<Airport>) {
        if generation < self.airport_search_generation {
            return;
        }
        self.airport_cache = airports.into_iter().map(Arc::new).collect();
    }

    /// Spawns an async airport search. Empty query returns a random sample.
    pub fn spawn_airport_search<F>(
        &self,
        generation: u64,
        query: String,
        limit: usize,
        on_complete: F,
    ) where
        F: FnOnce(u64, Vec<Airport>) + 'static,
    {
        let client = self.api_client.clone();
        let toast = self.toast_sender.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let result = if query.trim().is_empty() {
                client.random_airports(limit).await
            } else {
                client.search_airports(&query, limit).await
            };
            match result {
                Ok(airports) => on_complete(generation, airports),
                Err(e) => {
                    log::error!("airport search failed: {e}");
                    if let Some(s) = toast {
                        let _ = s.send((format!("Airport search failed: {e}"), ToastKind::Error));
                    }
                }
            }
        });
    }

    // --- Business Logic ---

    pub fn get_random_airports(&self, count: usize) -> Vec<Arc<Airport>> {
        crate::modules::data_operations::DataOperations::generate_random_airports(
            &self.airport_cache,
            count,
        )
    }

    pub fn create_list_item_for_airport(&self, airport: &Arc<Airport>) -> ListItemAirport {
        ListItemAirport::new(
            airport.Name.clone(),
            airport.ICAO.clone(),
            "N/A".to_string(),
        )
    }

    pub fn get_selected_airport_icao(
        &self,
        selected_airport: &Option<Arc<Airport>>,
    ) -> Option<String> {
        selected_airport.as_ref().map(|a| a.ICAO.clone())
    }

    /// Asks the backend to generate routes and delivers them via `on_complete`.
    pub fn spawn_route_generation_thread<F>(
        &self,
        display_mode: DisplayMode,
        selected_aircraft: Option<Arc<Aircraft>>,
        departure_icao: Option<String>,
        on_complete: F,
    ) where
        F: FnOnce(Vec<ListItemRoute>) + 'static,
    {
        let (mode, aircraft_id) = match (&display_mode, selected_aircraft.as_ref()) {
            (DisplayMode::NotFlownRoutes, _) => ("not_flown", None),
            (_, Some(a)) => ("specific", Some(a.id)),
            _ => ("random", None),
        };

        let client = self.api_client.clone();
        let aircraft_by_id = self.aircraft_by_id.clone();
        let mode = mode.to_string();
        let toast = self.toast_sender.clone();

        wasm_bindgen_futures::spawn_local(async move {
            match client
                .generate_routes(&mode, aircraft_id, departure_icao.as_deref())
                .await
            {
                Ok(responses) => {
                    let routes = responses
                        .into_iter()
                        .filter_map(|r| route_response_to_list_item(r, &aircraft_by_id))
                        .collect();
                    on_complete(routes);
                }
                Err(e) => {
                    log::error!("Route generation API call failed: {e}");
                    if let Some(s) = toast {
                        let _ = s.send((format!("Route generation failed: {e}"), ToastKind::Error));
                    }
                    on_complete(Vec::new());
                }
            }
        });
    }

    /// Asks the backend to convert a history record into a full route.
    pub fn spawn_route_from_history<F>(&self, history: &ListItemHistory, on_complete: F)
    where
        F: FnOnce(Option<ListItemRoute>) + 'static,
    {
        let client = self.api_client.clone();
        let aircraft_by_id = self.aircraft_by_id.clone();
        let toast = self.toast_sender.clone();
        let aircraft_id = history.aircraft_id;
        let dep = history.departure_icao.clone();
        let arr = history.arrival_icao.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match client.route_from_history(aircraft_id, &dep, &arr).await {
                Ok(resp) => on_complete(route_response_to_list_item(resp, &aircraft_by_id)),
                Err(e) => {
                    log::error!("route_from_history API call failed: {e}");
                    if let Some(s) = toast {
                        let _ = s.send((format!("Could not open route: {e}"), ToastKind::Error));
                    }
                    on_complete(None);
                }
            }
        });
    }

    pub fn toggle_aircraft_flown_status(&mut self, aircraft_id: i32) -> Result<(), Box<dyn Error>> {
        self.aircraft = self
            .aircraft
            .iter()
            .map(|a| {
                if a.id == aircraft_id {
                    let mut updated = a.as_ref().clone();
                    updated.flown = if updated.flown == 0 { 1 } else { 0 };
                    Arc::new(updated)
                } else {
                    a.clone()
                }
            })
            .collect();

        self.aircraft_by_id = self.aircraft.iter().map(|a| (a.id, a.clone())).collect();
        self.aircraft_items = services::aircraft_service::transform_to_list_items(&self.aircraft);

        let client = self.api_client.clone();
        let toast = self.toast_sender.clone();
        wasm_bindgen_futures::spawn_local(async move {
            if let Err(e) = client.toggle_flown(aircraft_id).await {
                log::error!("toggle_flown API call failed: {}", e);
                if let Some(s) = toast {
                    let _ = s.send((
                        format!("Failed to update flown status: {e}"),
                        ToastKind::Error,
                    ));
                }
            }
        });

        Ok(())
    }

    pub fn mark_all_aircraft_as_not_flown(&mut self) -> Result<(), Box<dyn Error>> {
        self.aircraft = self
            .aircraft
            .iter()
            .map(|a| {
                if a.flown != 0 {
                    let mut updated = a.as_ref().clone();
                    updated.flown = 0;
                    Arc::new(updated)
                } else {
                    a.clone()
                }
            })
            .collect();

        self.aircraft_by_id = self.aircraft.iter().map(|a| (a.id, a.clone())).collect();
        self.aircraft_items = services::aircraft_service::transform_to_list_items(&self.aircraft);

        let client = self.api_client.clone();
        let toast = self.toast_sender.clone();
        wasm_bindgen_futures::spawn_local(async move {
            if let Err(e) = client.reset_flown().await {
                log::error!("reset_flown API call failed: {}", e);
                if let Some(s) = toast {
                    let _ = s.send((format!("Failed to reset fleet: {e}"), ToastKind::Error));
                }
            }
        });

        Ok(())
    }

    pub fn add_history_entry(
        &mut self,
        aircraft: &Arc<Aircraft>,
        departure: &Arc<Airport>,
        destination: &Arc<Airport>,
    ) -> Result<(), Box<dyn Error>> {
        let item = ListItemHistory {
            id: format!("local-{}", self.history_items.len()),
            departure_info: format!("{} ({})", departure.Name, departure.ICAO),
            departure_icao: departure.ICAO.clone(),
            arrival_info: format!("{} ({})", destination.Name, destination.ICAO),
            arrival_icao: destination.ICAO.clone(),
            aircraft_name: format!("{} {}", aircraft.manufacturer, aircraft.variant),
            aircraft_id: aircraft.id,
            date: crate::date_utils::get_current_date_utc(),
        };
        self.history_items.push(item);
        self.cached_statistics = None;

        let aircraft_id = aircraft.id;
        let dep = departure.ICAO.clone();
        let arr = destination.ICAO.clone();
        let client = self.api_client.clone();
        let toast = self.toast_sender.clone();
        wasm_bindgen_futures::spawn_local(async move {
            if let Err(e) = client.add_history(aircraft_id, &dep, &arr).await {
                log::error!("add_history API call failed: {}", e);
                if let Some(s) = toast {
                    let _ = s.send((format!("Failed to record history: {e}"), ToastKind::Error));
                }
            }
        });

        Ok(())
    }

    pub fn mark_route_as_flown(&mut self, route: &ListItemRoute) -> Result<(), Box<dyn Error>> {
        self.add_history_entry(&route.aircraft, &route.departure, &route.destination)?;
        self.toggle_aircraft_flown_status(route.aircraft.id)?;
        Ok(())
    }

    /// Returns the most recently cached statistics, if any.
    pub fn cached_statistics(&self) -> Option<&FlightStatistics> {
        self.cached_statistics.as_ref()
    }

    pub fn set_cached_statistics(&mut self, stats: FlightStatistics) {
        self.cached_statistics = Some(stats);
    }

    /// Spawns an async fetch of flight statistics from the server.
    pub fn refresh_statistics<F>(&self, on_complete: F)
    where
        F: FnOnce(Result<FlightStatistics, String>) + 'static,
    {
        let client = self.api_client.clone();
        wasm_bindgen_futures::spawn_local(async move {
            on_complete(client.fetch_statistics().await);
        });
    }

    pub fn get_flight_statistics(
        &mut self,
    ) -> Result<FlightStatistics, Box<dyn Error + Send + Sync>> {
        Ok(self.cached_statistics.clone().unwrap_or_default())
    }

    pub fn invalidate_statistics_cache(&mut self) {
        self.cached_statistics = None;
    }

    pub fn get_setting(&mut self, key_str: &str) -> Result<Option<String>, Box<dyn Error>> {
        Ok(self.settings.get(key_str).cloned())
    }

    pub fn set_setting(&mut self, key_str: &str, value_str: &str) -> Result<(), Box<dyn Error>> {
        self.settings
            .insert(key_str.to_string(), value_str.to_string());
        let key = key_str.to_string();
        let value = value_str.to_string();
        let client = self.api_client.clone();
        let toast = self.toast_sender.clone();
        wasm_bindgen_futures::spawn_local(async move {
            if let Err(e) = client.save_setting(&key, &value).await {
                log::error!("save_setting API call failed: {}", e);
                if let Some(s) = toast {
                    let _ = s.send((format!("Failed to save setting: {e}"), ToastKind::Error));
                }
            }
        });
        Ok(())
    }

    pub fn get_api_key(&mut self) -> Result<Option<String>, Box<dyn Error>> {
        self.get_setting("api_key")
    }

    pub fn set_api_key(&mut self, api_key: &str) -> Result<(), Box<dyn Error>> {
        self.set_setting("api_key", api_key)
    }

    // --- Filtering and Sorting ---

    pub fn filter_aircraft_items(&self, search_text: &str) -> Vec<ListItemAircraft> {
        services::aircraft_service::filter_items(&self.aircraft_items, search_text)
    }

    pub fn filter_airport_items(
        items: &[ListItemAirport],
        search_text: &str,
    ) -> Vec<ListItemAirport> {
        services::airport_service::filter_items(items, search_text)
    }

    pub fn filter_route_items(&self, search_text: &str) -> Vec<ListItemRoute> {
        services::route_service::filter_items(&self.route_items, search_text)
    }

    pub fn filter_history_items(&self, search_text: &str) -> Vec<ListItemHistory> {
        services::history_service::filter_items(&self.history_items, search_text)
    }

    pub fn sort_route_items(&mut self, column: &str, ascending: bool) {
        services::route_service::sort_items(&mut self.route_items, column, ascending);
    }

    pub fn sort_history_items(&mut self, column: &str, ascending: bool) {
        services::history_service::sort_items(&mut self.history_items, column, ascending);
    }

    pub fn get_aircraft_display_name(&self, aircraft_id: i32) -> String {
        services::aircraft_service::get_display_name(&self.aircraft, aircraft_id)
    }

    pub fn get_airport_display_name(&self, icao: &str) -> String {
        services::airport_service::get_display_name(&self.airport_cache, icao)
    }
}

/// Converts a server-generated `RouteResponse` into a `ListItemRoute` for the UI.
fn route_response_to_list_item(
    resp: RouteResponse,
    aircraft_by_id: &HashMap<i32, Arc<Aircraft>>,
) -> Option<ListItemRoute> {
    let aircraft = aircraft_by_id.get(&resp.aircraft_id)?.clone();
    let departure = Arc::new(resp.departure);
    let destination = Arc::new(resp.destination);
    Some(ListItemRoute {
        aircraft_info: Arc::new(format!("{} {}", aircraft.manufacturer, aircraft.variant)),
        departure_info: Arc::new(format!("{} ({})", departure.Name, departure.ICAO)),
        destination_info: Arc::new(format!("{} ({})", destination.Name, destination.ICAO)),
        departure_runway_length: resp.departure_runway_ft,
        departure_runway_length_str: if resp.departure_runway_ft > 0 {
            format!("{}ft", resp.departure_runway_ft)
        } else {
            "N/A".to_string()
        },
        destination_runway_length: resp.destination_runway_ft,
        destination_runway_length_str: if resp.destination_runway_ft > 0 {
            format!("{}ft", resp.destination_runway_ft)
        } else {
            "N/A".to_string()
        },
        route_length: resp.distance_nm,
        distance_str: format!("{:.0} NM", resp.distance_nm),
        created_at: web_time::Instant::now(),
        departure,
        destination,
        aircraft,
    })
}
