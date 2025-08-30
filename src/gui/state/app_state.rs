use std::sync::Arc;

use crate::database::DatabasePool;
use crate::gui::data::{ListItemAircraft, ListItemAirport, ListItemHistory, ListItemRoute};
use crate::gui::services::{aircraft_service, airport_service, history_service, route_service};
use crate::models::{Aircraft, Airport, Runway};
use crate::modules::data_operations::{DataOperations, FlightStatistics};
use crate::modules::routes::RouteGenerator;
use crate::traits::{AircraftOperations, AirportOperations};

/// Central application state
pub struct AppState {
    /// Database connection pool
    database_pool: DatabasePool,

    /// Route generator for creating routes
    route_generator: RouteGenerator,

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

impl AppState {
    /// Creates a new AppState with loaded data.
    ///
    /// # Arguments
    ///
    /// * `database_pool` - The database pool
    ///
    /// # Returns
    ///
    /// Returns a Result with the new AppState or an error.
    pub fn new(mut database_pool: DatabasePool) -> Result<Self, Box<dyn std::error::Error>> {
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
                .map(|airport| crate::gui::ui::SpatialAirport {
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

        let route_generator = RouteGenerator {
            all_airports: airports.clone(),
            all_runways,
            spatial_airports,
        };

        // Generate UI items using services
        let aircraft_items = aircraft_service::transform_to_list_items(&aircraft);
        let airport_items = airport_service::transform_to_list_items_with_runways(
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

    /// Gets the database pool (mutable reference for operations)
    pub fn database_pool(&mut self) -> &mut DatabasePool {
        &mut self.database_pool
    }

    /// Gets the route generator
    pub fn route_generator(&self) -> &RouteGenerator {
        &self.route_generator
    }

    /// Gets all aircraft
    pub fn aircraft(&self) -> &[Arc<Aircraft>] {
        &self.aircraft
    }

    /// Gets all airports
    pub fn airports(&self) -> &[Arc<Airport>] {
        &self.airports
    }

    /// Gets aircraft items for the UI
    pub fn aircraft_items(&self) -> &[ListItemAircraft] {
        &self.aircraft_items
    }

    /// Gets airport items for the UI
    pub fn airport_items(&self) -> &[ListItemAirport] {
        &self.airport_items
    }

    /// Gets route items for the UI
    pub fn route_items(&self) -> &[ListItemRoute] {
        &self.route_items
    }

    /// Gets history items for the UI
    pub fn history_items(&self) -> &[ListItemHistory] {
        &self.history_items
    }

    /// Regenerates random routes
    pub fn regenerate_random_routes(&mut self, departure_icao: Option<&str>) {
        self.route_items = DataOperations::generate_random_routes(
            &self.route_generator,
            &self.aircraft,
            departure_icao,
        );
    }

    /// Regenerates routes for not flown aircraft
    pub fn regenerate_not_flown_routes(&mut self, departure_icao: Option<&str>) {
        self.route_items = DataOperations::generate_not_flown_routes(
            &self.route_generator,
            &self.aircraft,
            departure_icao,
        );
    }

    /// Appends additional random routes to the existing list for infinite scrolling
    pub fn append_random_routes(&mut self, departure_icao: Option<&str>) {
        let additional_routes = DataOperations::generate_random_routes(
            &self.route_generator,
            &self.aircraft,
            departure_icao,
        );
        self.route_items.extend(additional_routes);
    }

    /// Appends additional not flown routes to the existing list for infinite scrolling
    pub fn append_not_flown_routes(&mut self, departure_icao: Option<&str>) {
        let additional_routes = DataOperations::generate_not_flown_routes(
            &self.route_generator,
            &self.aircraft,
            departure_icao,
        );
        self.route_items.extend(additional_routes);
    }

    /// Appends additional routes for a specific aircraft to the existing list for infinite scrolling
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

    /// Regenerates routes for a specific aircraft
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

    /// Marks a route as flown and refreshes history
    pub fn mark_route_as_flown(
        &mut self,
        route: &ListItemRoute,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Mark the route as flown using high-level operation
        DataOperations::mark_route_as_flown(&mut self.database_pool, route)?;

        // Refresh history items
        self.history_items =
            DataOperations::load_history_data(&mut self.database_pool, &self.aircraft)?;

        // Invalidate statistics cache since a new flight was added
        self.invalidate_statistics_cache();

        Ok(())
    }

    /// Toggles the flown status of an aircraft and refreshes aircraft data
    pub fn toggle_aircraft_flown_status(
        &mut self,
        aircraft_id: i32,
    ) -> Result<(), Box<dyn std::error::Error>> {
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

    /// Gets random airports for display
    pub fn get_random_airports(&self, amount: usize) -> Vec<Arc<Airport>> {
        DataOperations::generate_random_airports(&self.airports, amount)
    }

    /// Filters aircraft items based on search text
    pub fn filter_aircraft_items(&self, search_text: &str) -> Vec<ListItemAircraft> {
        aircraft_service::filter_items(&self.aircraft_items, search_text)
    }

    /// Filters airport items based on search text
    pub fn filter_airport_items(&self, search_text: &str) -> Vec<ListItemAirport> {
        airport_service::filter_items(&self.airport_items, search_text)
    }

    /// Filters route items based on search text
    pub fn filter_route_items(&self, search_text: &str) -> Vec<ListItemRoute> {
        route_service::filter_items(&self.route_items, search_text)
    }

    /// Filters history items based on search text
    pub fn filter_history_items(&self, search_text: &str) -> Vec<ListItemHistory> {
        history_service::filter_items(&self.history_items, search_text)
    }

    /// Sorts route items by the given column and direction
    pub fn sort_route_items(&mut self, column: &str, ascending: bool) {
        route_service::sort_items(&mut self.route_items, column, ascending);
    }

    /// Sorts history items by the given column and direction
    pub fn sort_history_items(&mut self, column: &str, ascending: bool) {
        history_service::sort_items(&mut self.history_items, column, ascending);
    }

    /// Gets the display name for an aircraft by its ID
    pub fn get_aircraft_display_name(&self, aircraft_id: i32) -> String {
        aircraft_service::get_display_name(&self.aircraft, aircraft_id)
    }

    /// Gets the display name for an airport by its ICAO
    pub fn get_airport_display_name(&self, icao: &str) -> String {
        airport_service::get_display_name(&self.airports, icao)
    }

    /// Gets runway data for an airport
    pub fn get_runways_for_airport(&self, airport: &Airport) -> Vec<Arc<Runway>> {
        self.route_generator
            .all_runways
            .get(&airport.ID)
            .map(|runways| runways.iter().map(|r| Arc::new(r.clone())).collect())
            .unwrap_or_default()
    }

    /// Gets flight statistics with caching for performance
    pub fn get_flight_statistics(
        &mut self,
    ) -> Result<FlightStatistics, Box<dyn std::error::Error>> {
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
}
