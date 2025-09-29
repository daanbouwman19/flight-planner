use crate::database::DatabasePool;
use crate::gui::data::{ListItemAircraft, ListItemAirport, ListItemHistory, ListItemRoute};
use crate::gui::services;
use crate::gui::services::popup_service::DisplayMode;
use crate::models::{Aircraft, Airport, Runway};
use crate::modules::data_operations::{DataOperations, FlightStatistics};
use crate::modules::routes::RouteGenerator;
use crate::traits::{AircraftOperations, AirportOperations};
use std::error::Error;
use std::sync::Arc;

/// Core application service handling business logic and data operations.
/// This is a **Model** in MVVM - it contains business logic, not UI state.
#[derive(Clone)]
pub struct AppService {
    /// Database connection pool
    database_pool: DatabasePool,

    /// Route generator for creating routes, shared with background threads.
    route_generator: Arc<RouteGenerator>,

    /// All loaded aircraft
    aircraft: Vec<Arc<Aircraft>>,

    /// All loaded airports
    airports: Vec<Arc<Airport>>,

    /// Currently loaded aircraft items for the UI
    aircraft_items: Vec<ListItemAircraft>,

    /// Currently loaded airport items for the UI
    airport_items: Vec<ListItemAirport>,

    /// Currently loaded route items for the UI
    route_items: Vec<ListItemRoute>,

    /// Currently loaded history items for the UI
    history_items: Vec<ListItemHistory>,

    /// Cached flight statistics
    cached_statistics: Option<FlightStatistics>,

    /// Flag to indicate if statistics need to be recalculated
    statistics_dirty: bool,
}

impl AppService {
    /// Creates a new AppService with loaded data.
    pub fn new(mut database_pool: DatabasePool) -> Result<Self, Box<dyn Error>> {
        // Load base data
        let aircraft_raw = database_pool.get_all_aircraft()?;
        let airports_raw = database_pool.get_airports()?;

        // Wrap in Arc for sharing
        let aircraft: Vec<Arc<Aircraft>> = aircraft_raw.into_iter().map(Arc::new).collect();
        let airports: Vec<Arc<Airport>> = airports_raw.into_iter().map(Arc::new).collect();

        // Create route generator
        let runways = database_pool.get_runways()?;

        // Create spatial index for airports
        let spatial_airports = rstar::RTree::bulk_load(
            airports
                .iter()
                .map(|airport| crate::models::airport::SpatialAirport {
                    airport: Arc::clone(airport),
                })
                .collect(),
        );

        // Create runways hashmap
        let all_runways: std::collections::HashMap<i32, Arc<Vec<Runway>>> = runways
            .into_iter()
            .fold(
                std::collections::HashMap::<i32, Vec<Runway>>::new(),
                |mut acc, runway| {
                    acc.entry(runway.AirportID).or_default().push(runway);
                    acc
                },
            )
            .into_iter()
            .map(|(k, v)| (k, Arc::new(v)))
            .collect();

        let route_generator = Arc::new(RouteGenerator::new(
            airports.clone(),
            all_runways,
            spatial_airports,
        ));

        // Generate UI items using services
        let aircraft_items = services::aircraft_service::transform_to_list_items(&aircraft);
        let airport_items = services::airport_service::transform_to_list_items_with_runways(
            &airports,
            &route_generator.all_runways,
        );
        let route_items = DataOperations::generate_random_routes(&route_generator, &aircraft, None);
        let history_items = DataOperations::load_history_data(&mut database_pool, &aircraft)?;

        Ok(Self {
            database_pool,
            route_generator,
            aircraft,
            airports,
            aircraft_items,
            airport_items,
            route_items,
            history_items,
            cached_statistics: None,
            statistics_dirty: true,
        })
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

    pub fn set_route_items(&mut self, routes: Vec<ListItemRoute>) {
        self.route_items = routes;
    }

    /// Appends new routes to the existing route_items vector in place.
    /// This is more efficient than cloning the entire vector when adding new routes.
    pub fn append_route_items(&mut self, new_routes: Vec<ListItemRoute>) {
        self.route_items.extend(new_routes);
    }

    pub fn history_items(&self) -> &[ListItemHistory] {
        &self.history_items
    }

    pub fn airport_items(&self) -> &[ListItemAirport] {
        &self.airport_items
    }

    pub fn aircraft_items(&self) -> &[ListItemAircraft] {
        &self.aircraft_items
    }

    /// Gets the database pool (mutable reference for operations)
    pub fn database_pool(&mut self) -> &mut DatabasePool {
        &mut self.database_pool
    }

    /// Gets the route generator
    pub fn route_generator(&self) -> &Arc<RouteGenerator> {
        &self.route_generator
    }

    // --- Business Logic Methods ---

    pub fn get_random_airports(&self, count: usize) -> Vec<Arc<Airport>> {
        DataOperations::generate_random_airports(&self.airports, count)
    }

    pub fn get_runways_for_airport(&self, airport: &Airport) -> Vec<Arc<Runway>> {
        self.route_generator
            .all_runways
            .get(&airport.ID)
            .map(|runways| runways.iter().map(|r| Arc::new(r.clone())).collect())
            .unwrap_or_default()
    }

    pub fn regenerate_routes_for_aircraft(
        &mut self,
        aircraft: &Arc<Aircraft>,
        departure_icao: Option<&str>,
    ) {
        self.route_items = DataOperations::generate_routes_for_aircraft(
            &self.route_generator,
            aircraft,
            departure_icao,
        );
    }

    pub fn regenerate_random_routes(&mut self, departure_icao: Option<&str>) {
        self.route_items = DataOperations::generate_random_routes(
            &self.route_generator,
            &self.aircraft,
            departure_icao,
        );
    }

    pub fn regenerate_not_flown_routes(&mut self, departure_icao: Option<&str>) {
        self.route_items = DataOperations::generate_not_flown_routes(
            &self.route_generator,
            &self.aircraft,
            departure_icao,
        );
    }

    pub fn append_routes_for_aircraft(
        &mut self,
        aircraft: &Arc<Aircraft>,
        departure_icao: Option<&str>,
    ) {
        let additional_routes = DataOperations::generate_routes_for_aircraft(
            &self.route_generator,
            aircraft,
            departure_icao,
        );
        self.route_items.extend(additional_routes);
    }

    pub fn append_random_routes(&mut self, departure_icao: Option<&str>) {
        let additional_routes = DataOperations::generate_random_routes(
            &self.route_generator,
            &self.aircraft,
            departure_icao,
        );
        self.route_items.extend(additional_routes);
    }

    pub fn append_not_flown_routes(&mut self, departure_icao: Option<&str>) {
        let additional_routes = DataOperations::generate_not_flown_routes(
            &self.route_generator,
            &self.aircraft,
            departure_icao,
        );
        self.route_items.extend(additional_routes);
    }

    pub fn toggle_aircraft_flown_status(&mut self, aircraft_id: i32) -> Result<(), Box<dyn Error>> {
        // Toggle the aircraft flown status using database operations
        DataOperations::toggle_aircraft_flown_status(&mut self.database_pool, aircraft_id)?;

        // Refresh aircraft data
        self.aircraft = self
            .database_pool
            .get_all_aircraft()?
            .into_iter()
            .map(Arc::new)
            .collect();
        self.aircraft_items = DataOperations::load_aircraft_data(&self.aircraft)?;

        Ok(())
    }

    pub fn add_history_entry(
        &mut self,
        aircraft: &Arc<Aircraft>,
        departure: &Arc<Airport>,
        destination: &Arc<Airport>,
    ) -> Result<(), Box<dyn Error>> {
        // Add the history entry using high-level operation
        DataOperations::add_history_entry(
            &mut self.database_pool,
            aircraft,
            departure,
            destination,
        )?;

        // Refresh history items
        self.history_items =
            DataOperations::load_history_data(&mut self.database_pool, &self.aircraft)?;

        // Invalidate statistics cache since a new flight was added
        self.invalidate_statistics_cache();

        Ok(())
    }

    pub fn mark_route_as_flown(&mut self, route: &ListItemRoute) -> Result<(), Box<dyn Error>> {
        // Mark the route as flown using high-level operation
        DataOperations::mark_route_as_flown(&mut self.database_pool, route)?;

        // Refresh history items
        self.history_items =
            DataOperations::load_history_data(&mut self.database_pool, &self.aircraft)?;

        // Invalidate statistics cache since a new flight was added
        self.invalidate_statistics_cache();

        Ok(())
    }

    pub fn get_flight_statistics(
        &mut self,
    ) -> Result<FlightStatistics, Box<dyn Error + Send + Sync>> {
        if self.statistics_dirty || self.cached_statistics.is_none() {
            let statistics =
                DataOperations::calculate_statistics(&mut self.database_pool, &self.aircraft)?;
            self.cached_statistics = Some(statistics);
            self.statistics_dirty = false;
        }
        // The cache is now guaranteed to be populated.
        // We clone from the cache to return an owned value.
        Ok(self.cached_statistics.as_ref().unwrap().clone())
    }

    /// Marks the statistics cache as dirty and clears it
    /// Call this when flights are added or removed
    pub fn invalidate_statistics_cache(&mut self) {
        self.statistics_dirty = true;
        self.cached_statistics = None;
    }

    // --- Filtering and Sorting ---

    /// Filters aircraft items based on search text
    pub fn filter_aircraft_items(&self, search_text: &str) -> Vec<ListItemAircraft> {
        services::aircraft_service::filter_items(&self.aircraft_items, search_text)
    }

    /// Filters airport items based on search text
    pub fn filter_airport_items(&self, search_text: &str) -> Vec<ListItemAirport> {
        services::airport_service::filter_items(&self.airport_items, search_text)
    }

    /// Filters route items based on search text
    pub fn filter_route_items(&self, search_text: &str) -> Vec<ListItemRoute> {
        services::route_service::filter_items(&self.route_items, search_text)
    }

    /// Filters history items based on search text
    pub fn filter_history_items(&self, search_text: &str) -> Vec<ListItemHistory> {
        services::history_service::filter_items(&self.history_items, search_text)
    }

    /// Sorts route items by the given column and direction
    pub fn sort_route_items(&mut self, column: &str, ascending: bool) {
        services::route_service::sort_items(&mut self.route_items, column, ascending);
    }

    /// Sorts history items by the given column and direction
    pub fn sort_history_items(&mut self, column: &str, ascending: bool) {
        services::history_service::sort_items(&mut self.history_items, column, ascending);
    }

    /// Gets the display name for an aircraft by its ID
    pub fn get_aircraft_display_name(&self, aircraft_id: i32) -> String {
        services::aircraft_service::get_display_name(&self.aircraft, aircraft_id)
    }

    /// Gets the display name for an airport by its ICAO
    pub fn get_airport_display_name(&self, icao: &str) -> String {
        services::airport_service::get_display_name(&self.airports, icao)
    }

    pub fn get_selected_airport_icao(
        &self,
        selected_airport: &Option<Arc<Airport>>,
    ) -> Option<String> {
        selected_airport.as_ref().map(|a| a.ICAO.clone())
    }

    pub fn create_list_item_for_airport(&self, airport: &Arc<Airport>) -> ListItemAirport {
        let runway_length = self
            .route_generator
            .all_runways
            .get(&airport.ID)
            .and_then(|runways| runways.iter().max_by_key(|r| r.Length))
            .map_or("No runways".to_string(), |r| format!("{}ft", r.Length));
        ListItemAirport::new(airport.Name.clone(), airport.ICAO.clone(), runway_length)
    }

    pub fn generate_routes(
        &self,
        display_mode: &DisplayMode,
        selected_aircraft: &Option<Arc<Aircraft>>,
        departure_icao: Option<&str>,
    ) -> Vec<ListItemRoute> {
        match (display_mode, selected_aircraft) {
            (DisplayMode::RandomRoutes | DisplayMode::SpecificAircraftRoutes, Some(aircraft)) => {
                DataOperations::generate_routes_for_aircraft(
                    &self.route_generator,
                    aircraft,
                    departure_icao,
                )
            }
            (DisplayMode::RandomRoutes | DisplayMode::SpecificAircraftRoutes, None) => {
                DataOperations::generate_random_routes(
                    &self.route_generator,
                    &self.aircraft,
                    departure_icao,
                )
            }
            (DisplayMode::NotFlownRoutes, _) => DataOperations::generate_not_flown_routes(
                &self.route_generator,
                &self.aircraft,
                departure_icao,
            ),
            _ => Vec::new(),
        }
    }

    pub fn spawn_route_generation_thread<F>(
        &self,
        display_mode: DisplayMode,
        selected_aircraft: Option<Arc<Aircraft>>,
        departure_icao: Option<String>,
        on_complete: F,
    ) where
        F: FnOnce(Vec<ListItemRoute>) + Send + 'static,
    {
        let app_service = self.clone();
        std::thread::spawn(move || {
            let routes = app_service.generate_routes(
                &display_mode,
                &selected_aircraft,
                departure_icao.as_deref(),
            );
            on_complete(routes);
        });
    }
}
