use crate::gui::data::{ListItemAircraft, ListItemAirport, ListItemHistory, ListItemRoute};
use crate::gui::services;
use crate::gui::services::popup_service::DisplayMode;
use crate::models::FlightStatistics;
use crate::models::{Aircraft, Airport, History, RouteResponse};
use crate::web::api_client::ApiClient;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

/// WASM-compatible application service that mirrors AppService.
///
/// Holds aircraft and airport data fetched once from the backend at startup.
/// Route generation is delegated entirely to the backend via `POST /api/routes`,
/// so the WASM client never needs to load the runway database or build a spatial
/// index locally.
#[derive(Clone)]
pub struct WebAppService {
    api_client: ApiClient,
    aircraft: Vec<Arc<Aircraft>>,
    airports: Vec<Arc<Airport>>,
    airport_by_icao: HashMap<String, Arc<Airport>>,
    aircraft_by_id: HashMap<i32, Arc<Aircraft>>,
    aircraft_items: Vec<ListItemAircraft>,
    route_items: Vec<ListItemRoute>,
    history_items: Vec<ListItemHistory>,
    settings: HashMap<String, String>,
    cached_statistics: Option<FlightStatistics>,
    statistics_dirty: bool,
}

impl WebAppService {
    pub fn new(
        aircraft_raw: Vec<Aircraft>,
        airports_raw: Vec<Airport>,
        history_raw: Vec<History>,
        settings: HashMap<String, String>,
        api_client: ApiClient,
        initial_routes: Vec<RouteResponse>,
    ) -> Self {
        let aircraft: Vec<Arc<Aircraft>> = aircraft_raw.into_iter().map(Arc::new).collect();
        let airports: Vec<Arc<Airport>> = airports_raw.into_iter().map(Arc::new).collect();

        let airport_by_icao: HashMap<String, Arc<Airport>> = airports
            .iter()
            .map(|a| (a.ICAO.clone(), a.clone()))
            .collect();
        let aircraft_by_id: HashMap<i32, Arc<Aircraft>> =
            aircraft.iter().map(|a| (a.id, a.clone())).collect();

        let aircraft_items = services::aircraft_service::transform_to_list_items(&aircraft);

        let route_items = initial_routes
            .into_iter()
            .filter_map(|r| route_response_to_list_item(r, &aircraft_by_id))
            .collect();

        let airport_map: HashMap<&str, &Arc<Airport>> =
            airports.iter().map(|a| (a.ICAO.as_str(), a)).collect();
        let aircraft_map: HashMap<i32, &Arc<Aircraft>> =
            aircraft.iter().map(|a| (a.id, a)).collect();

        let history_items = history_raw
            .into_iter()
            .map(|h| {
                let aircraft_name = aircraft_map.get(&h.aircraft).map_or_else(
                    || format!("Unknown Aircraft (ID: {})", h.aircraft),
                    |a| format!("{} {}", a.manufacturer, a.variant),
                );
                let dep_name = airport_map
                    .get(h.departure_icao.as_str())
                    .map_or("Unknown Airport", |a| a.Name.as_str());
                let arr_name = airport_map
                    .get(h.arrival_icao.as_str())
                    .map_or("Unknown Airport", |a| a.Name.as_str());
                ListItemHistory {
                    id: h.id.to_string(),
                    departure_info: format!("{} ({})", dep_name, h.departure_icao),
                    departure_icao: h.departure_icao,
                    arrival_info: format!("{} ({})", arr_name, h.arrival_icao),
                    arrival_icao: h.arrival_icao,
                    aircraft_name,
                    aircraft_id: h.aircraft,
                    date: h.date,
                }
            })
            .collect();

        Self {
            api_client,
            aircraft,
            airports,
            airport_by_icao,
            aircraft_by_id,
            aircraft_items,
            route_items,
            history_items,
            settings,
            cached_statistics: None,
            statistics_dirty: true,
        }
    }

    // --- Data Access ---

    pub fn airports(&self) -> &[Arc<Airport>] {
        &self.airports
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
        services::airport_service::transform_to_list_items(&self.airports)
    }

    pub fn aircraft_items(&self) -> &[ListItemAircraft] {
        &self.aircraft_items
    }

    // --- Business Logic ---

    pub fn get_random_airports(&self, count: usize) -> Vec<Arc<Airport>> {
        crate::modules::data_operations::DataOperations::generate_random_airports(
            &self.airports,
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
                    on_complete(Vec::new());
                }
            }
        });
    }

    pub fn get_route_from_history(&self, history: &ListItemHistory) -> Option<ListItemRoute> {
        let departure = self.airport_by_icao.get(&history.departure_icao)?.clone();
        let destination = self.airport_by_icao.get(&history.arrival_icao)?.clone();
        let aircraft = self.aircraft_by_id.get(&history.aircraft_id)?.clone();
        let distance =
            crate::util::calculate_haversine_distance_nm(&departure, &destination) as f64;

        Some(ListItemRoute {
            departure: departure.clone(),
            destination: destination.clone(),
            aircraft: aircraft.clone(),
            departure_runway_length: 0,
            departure_runway_length_str: "N/A".to_string(),
            destination_runway_length: 0,
            destination_runway_length_str: "N/A".to_string(),
            route_length: distance,
            aircraft_info: Arc::new(format!("{} {}", aircraft.manufacturer, aircraft.variant)),
            departure_info: Arc::new(format!("{} ({})", departure.Name, departure.ICAO)),
            destination_info: Arc::new(format!("{} ({})", destination.Name, destination.ICAO)),
            distance_str: format!("{:.0} NM", distance),
            created_at: web_time::Instant::now(),
        })
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
        wasm_bindgen_futures::spawn_local(async move {
            if let Err(e) = client.toggle_flown(aircraft_id).await {
                log::error!("toggle_flown API call failed: {}", e);
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
        wasm_bindgen_futures::spawn_local(async move {
            if let Err(e) = client.reset_flown().await {
                log::error!("reset_flown API call failed: {}", e);
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
        let distance = crate::util::calculate_haversine_distance_nm(departure, destination) as i32;
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
        self.invalidate_statistics_cache();

        let _ = distance; // stored in DB by server
        let aircraft_id = aircraft.id;
        let dep = departure.ICAO.clone();
        let arr = destination.ICAO.clone();
        let client = self.api_client.clone();
        wasm_bindgen_futures::spawn_local(async move {
            if let Err(e) = client.add_history(aircraft_id, &dep, &arr).await {
                log::error!("add_history API call failed: {}", e);
            }
        });

        Ok(())
    }

    pub fn mark_route_as_flown(&mut self, route: &ListItemRoute) -> Result<(), Box<dyn Error>> {
        self.add_history_entry(&route.aircraft, &route.departure, &route.destination)?;
        self.toggle_aircraft_flown_status(route.aircraft.id)?;
        Ok(())
    }

    pub fn get_flight_statistics(
        &mut self,
    ) -> Result<FlightStatistics, Box<dyn Error + Send + Sync>> {
        if self.statistics_dirty || self.cached_statistics.is_none() {
            let stats = self.compute_statistics();
            self.cached_statistics = Some(stats);
            self.statistics_dirty = false;
        }
        Ok(self.cached_statistics.as_ref().unwrap().clone())
    }

    pub fn invalidate_statistics_cache(&mut self) {
        self.statistics_dirty = true;
        self.cached_statistics = None;
    }

    fn compute_statistics(&self) -> FlightStatistics {
        let total_flights = self.history_items.len();
        let total_distance: i32 = self
            .history_items
            .iter()
            .filter_map(|h| self.lookup_distance(h))
            .sum();
        let average_flight_distance = if total_flights > 0 {
            total_distance as f64 / total_flights as f64
        } else {
            0.0
        };

        let mut aircraft_count: HashMap<i32, usize> = HashMap::new();
        let mut dep_count: HashMap<&str, usize> = HashMap::new();
        let mut arr_count: HashMap<&str, usize> = HashMap::new();

        for h in &self.history_items {
            *aircraft_count.entry(h.aircraft_id).or_insert(0) += 1;
            *dep_count.entry(h.departure_icao.as_str()).or_insert(0) += 1;
            *arr_count.entry(h.arrival_icao.as_str()).or_insert(0) += 1;
        }

        let most_flown_aircraft = aircraft_count
            .iter()
            .max_by_key(|(_, c)| *c)
            .and_then(|(id, _)| self.aircraft_by_id.get(id))
            .map(|a| format!("{} {}", a.manufacturer, a.variant));

        let most_visited_airport = arr_count
            .iter()
            .max_by_key(|(_, c)| *c)
            .map(|(icao, _)| self.format_airport_name(icao));

        let favorite_departure = dep_count
            .iter()
            .max_by_key(|(_, c)| *c)
            .map(|(icao, _)| self.format_airport_name(icao));

        let longest_flight = self
            .history_items
            .iter()
            .max_by_key(|h| self.lookup_distance(h).unwrap_or(0))
            .map(|h| {
                format!(
                    "{} → {} ({} NM)",
                    h.departure_icao,
                    h.arrival_icao,
                    self.lookup_distance(h).unwrap_or(0)
                )
            });

        let shortest_flight = self
            .history_items
            .iter()
            .filter(|h| self.lookup_distance(h).unwrap_or(0) > 0)
            .min_by_key(|h| self.lookup_distance(h).unwrap_or(i32::MAX))
            .map(|h| {
                format!(
                    "{} → {} ({} NM)",
                    h.departure_icao,
                    h.arrival_icao,
                    self.lookup_distance(h).unwrap_or(0)
                )
            });

        FlightStatistics {
            total_flights,
            total_distance,
            most_flown_aircraft,
            most_visited_airport: most_visited_airport.clone(),
            average_flight_distance,
            longest_flight,
            shortest_flight,
            favorite_departure_airport: favorite_departure,
            favorite_arrival_airport: most_visited_airport,
        }
    }

    fn lookup_distance(&self, h: &ListItemHistory) -> Option<i32> {
        let dep = self.airport_by_icao.get(&h.departure_icao)?;
        let arr = self.airport_by_icao.get(&h.arrival_icao)?;
        Some(crate::util::calculate_haversine_distance_nm(dep, arr) as i32)
    }

    fn format_airport_name(&self, icao: &str) -> String {
        self.airport_by_icao
            .get(icao)
            .map_or_else(|| icao.to_string(), |a| format!("{} ({})", a.Name, a.ICAO))
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
        wasm_bindgen_futures::spawn_local(async move {
            if let Err(e) = client.save_setting(&key, &value).await {
                log::error!("save_setting API call failed: {}", e);
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
        services::airport_service::get_display_name(&self.airports, icao)
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
