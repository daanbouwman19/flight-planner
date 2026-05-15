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
    /// Offset to pass on the next airport browse page request.
    airports_browse_offset: usize,
    /// Whether another page of browse airports is available from the server.
    airports_browse_has_more: bool,
    route_items: Vec<ListItemRoute>,
    history_items: Vec<ListItemHistory>,
    /// Offset for the next history page to fetch from the server.
    history_next_offset: usize,
    /// Whether more history pages are available.
    history_has_more: bool,
    settings: HashMap<String, String>,
    cached_statistics: Option<FlightStatistics>,
    toast_sender: Option<mpsc::Sender<(String, ToastKind)>>,
}

impl WebAppService {
    pub fn new(
        aircraft_raw: Vec<Aircraft>,
        history_page: crate::models::HistoryPageResponse,
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

        let page_size: usize = history_page.items.len();
        let history_has_more = history_page.has_more;
        let history_items: Vec<ListItemHistory> = history_page
            .items
            .into_iter()
            .map(history_item_response_to_list_item)
            .collect();

        Self {
            api_client,
            aircraft,
            aircraft_by_id,
            aircraft_items,
            airport_cache,
            airport_search_generation: 0,
            airports_browse_offset: 0,
            airports_browse_has_more: false,
            route_items,
            history_items,
            history_next_offset: page_size,
            history_has_more,
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

    pub fn history_has_more(&self) -> bool {
        self.history_has_more
    }

    pub fn airports_browse_has_more(&self) -> bool {
        self.airports_browse_has_more
    }

    /// True when more airports can be fetched for the departure dropdown.
    /// Covers both the initial load (random seed, offset=0) and subsequent pages.
    pub fn can_load_more_for_dropdown(&self) -> bool {
        self.airports_browse_offset == 0 || self.airports_browse_has_more
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
    /// Resets browse pagination state (this is a fresh search, not a page append).
    pub fn apply_airport_search_result(&mut self, generation: u64, airports: Vec<Airport>) {
        if generation < self.airport_search_generation {
            return;
        }
        self.airport_cache = airports.into_iter().map(Arc::new).collect();
        self.airports_browse_offset = 0;
        self.airports_browse_has_more = false;
    }

    /// Applies a browse page result, resetting the cache to page 1.
    pub fn apply_airport_browse_page(&mut self, airports: Vec<Airport>, has_more: bool) {
        let count = airports.len();
        self.airport_cache = airports.into_iter().map(Arc::new).collect();
        self.airports_browse_has_more = has_more;
        self.airports_browse_offset = count;
    }

    /// Appends the next browse page to the existing cache.
    pub fn append_airport_browse_page(&mut self, airports: Vec<Airport>, has_more: bool) {
        let count = airports.len();
        self.airport_cache
            .extend(airports.into_iter().map(Arc::new));
        self.airports_browse_has_more = has_more;
        self.airports_browse_offset += count;
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

    /// Spawns an async fetch of the first page of browse airports from `/api/airports`.
    pub fn spawn_airport_browse_page<F>(&self, page_size: usize, on_complete: F)
    where
        F: FnOnce(Vec<Airport>, bool) + 'static,
    {
        let client = self.api_client.clone();
        let toast = self.toast_sender.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match client.fetch_airports_page(0, page_size).await {
                Ok((airports, has_more)) => on_complete(airports, has_more),
                Err(e) => {
                    log::error!("airport browse failed: {e}");
                    if let Some(s) = toast {
                        let _ = s.send((format!("Airport load failed: {e}"), ToastKind::Error));
                    }
                }
            }
        });
    }

    /// Spawns an async fetch of the next browse page.
    /// When `airports_browse_offset == 0` this is the first load (replaces the random
    /// seed cache); subsequent calls append. The `is_append` flag in the callback reflects
    /// this so the receiver can call the right apply method.
    pub fn spawn_load_more_airports<F>(&self, on_complete: F)
    where
        F: FnOnce(Vec<Airport>, bool, bool) + 'static,
    {
        let offset = self.airports_browse_offset;
        let is_append = offset > 0;
        let client = self.api_client.clone();
        let toast = self.toast_sender.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match client.fetch_airports_page(offset, 200).await {
                Ok((airports, has_more)) => on_complete(airports, has_more, is_append),
                Err(e) => {
                    log::error!("airport load-more failed: {e}");
                    if let Some(s) = toast {
                        let _ = s.send((format!("Airport load failed: {e}"), ToastKind::Error));
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

    /// Extends the in-memory history list with a new page from the server.
    pub fn extend_history_items(&mut self, page: crate::models::HistoryPageResponse) {
        self.history_next_offset += page.items.len();
        self.history_has_more = page.has_more;
        self.history_items.extend(
            page.items
                .into_iter()
                .map(history_item_response_to_list_item),
        );
    }

    /// Spawns an async fetch of the next history page (50 items per page).
    pub fn spawn_load_more_history<F>(&self, on_complete: F)
    where
        F: FnOnce(crate::models::HistoryPageResponse) + 'static,
    {
        let offset = self.history_next_offset;
        let client = self.api_client.clone();
        let toast = self.toast_sender.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match client.fetch_history(50, offset).await {
                Ok(page) => on_complete(page),
                Err(e) => {
                    log::error!("history load-more failed: {e}");
                    if let Some(s) = toast {
                        let _ = s.send((format!("History load failed: {e}"), ToastKind::Error));
                    }
                }
            }
        });
    }

    pub fn toggle_aircraft_flown_status(&mut self, aircraft_id: i32) -> Result<(), Box<dyn Error>> {
        if let Some(a) = self.aircraft.iter_mut().find(|a| a.id == aircraft_id) {
            let mut updated = (**a).clone();
            updated.flown = if updated.flown == 0 { 1 } else { 0 };
            *a = Arc::new(updated);
        }

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
        for a in self.aircraft.iter_mut() {
            if a.flown != 0 {
                let mut updated = (**a).clone();
                updated.flown = 0;
                *a = Arc::new(updated);
            }
        }

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

fn history_item_response_to_list_item(h: HistoryItemResponse) -> ListItemHistory {
    ListItemHistory {
        id: h.id.to_string(),
        departure_info: format!("{} ({})", h.departure_name, h.departure_icao),
        departure_icao: h.departure_icao,
        arrival_info: format!("{} ({})", h.arrival_name, h.arrival_icao),
        arrival_icao: h.arrival_icao,
        aircraft_name: h.aircraft_name,
        aircraft_id: h.aircraft_id,
        date: h.date,
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
