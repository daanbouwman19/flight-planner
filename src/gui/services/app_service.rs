use crate::database::DatabasePool;
use crate::gui::data::{ListItemAircraft, ListItemAirport, ListItemHistory, ListItemRoute};
use crate::gui::services;
use crate::gui::services::popup_service::DisplayMode;
use crate::models::setting::Setting;
use crate::models::{Aircraft, Airport, Runway};
use crate::modules::data_operations::{DataOperations, FlightStatistics};
use crate::modules::routes::RouteGenerator;
use crate::schema::settings::dsl::*;
use crate::traits::{AircraftOperations, AirportOperations};
use diesel::prelude::*;
use std::error::Error;
use std::sync::Arc;
use std::thread;

/// The core application service that handles business logic and data operations.
///
/// `AppService` acts as the primary intermediary between the UI and the database.
/// It loads, caches, and provides access to all necessary application data,
/// such as aircraft, airports, and routes. It also encapsulates high-level
/// operations like generating routes and calculating statistics.
///
/// This service is designed to be cloneable and shareable, particularly for
/// use in background threads.
#[derive(Clone)]
pub struct AppService {
    /// The database connection pool.
    database_pool: DatabasePool,
    /// A shared `RouteGenerator` for creating flight routes.
    route_generator: Arc<RouteGenerator>,
    /// A cached vector of all aircraft, wrapped in `Arc` for efficient sharing.
    aircraft: Vec<Arc<Aircraft>>,
    /// A cached vector of all airports, wrapped in `Arc`.
    airports: Vec<Arc<Airport>>,
    /// A cached vector of aircraft formatted for UI display.
    aircraft_items: Vec<ListItemAircraft>,
    /// The currently loaded list of routes for display.
    route_items: Vec<ListItemRoute>,
    /// The currently loaded flight history for display.
    history_items: Vec<ListItemHistory>,
    /// An optional cache for flight statistics to avoid recalculation.
    cached_statistics: Option<FlightStatistics>,
    /// A flag indicating whether the statistics cache is stale and needs recalculation.
    statistics_dirty: bool,
}

impl AppService {
    /// Creates a new `AppService` instance by loading initial data from the database.
    ///
    /// This constructor is responsible for:
    /// - Loading all aircraft and airports from the database in parallel.
    /// - Initializing the `RouteGenerator` with all necessary data.
    /// - Pre-populating the lists of UI-formatted items.
    ///
    /// # Arguments
    ///
    /// * `database_pool` - A `DatabasePool` for accessing the application's databases.
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `AppService` instance on success, or an error
    /// if any database operation fails.
    pub fn new(mut database_pool: DatabasePool) -> Result<Self, Box<dyn Error>> {
        let start = std::time::Instant::now();

        // Parallel Data Loading
        let pool_clone_1 = database_pool.clone();
        let aircraft_handle = thread::spawn(
            move || -> Result<Vec<Aircraft>, Box<dyn Error + Send + Sync>> {
                let mut pool = pool_clone_1;
                Ok(pool.get_all_aircraft()?)
            },
        );

        let pool_clone_2 = database_pool.clone();
        let airports_handle = thread::spawn(
            move || -> Result<Vec<Airport>, Box<dyn Error + Send + Sync>> {
                let mut pool = pool_clone_2;
                Ok(pool.get_airports()?)
            },
        );

        let pool_clone_3 = database_pool.clone();
        let runways_handle = thread::spawn(
            move || -> Result<Vec<Runway>, Box<dyn Error + Send + Sync>> {
                let pool = pool_clone_3;
                Ok(pool.get_runways()?)
            },
        );

        // Wait for aircraft and airports first, as they are needed for other operations
        // We map error to string then to Box<dyn Error> to satisfy ? operator
        let aircraft_raw = aircraft_handle
            .join()
            .map_err(|_| Box::<dyn Error>::from("Aircraft thread panicked"))?
            .map_err(|e| Box::<dyn Error>::from(e.to_string()))?;

        let airports_raw = airports_handle
            .join()
            .map_err(|_| Box::<dyn Error>::from("Airports thread panicked"))?
            .map_err(|e| Box::<dyn Error>::from(e.to_string()))?;

        let runways = runways_handle
            .join()
            .map_err(|_| Box::<dyn Error>::from("Runways thread panicked"))?
            .map_err(|e| Box::<dyn Error>::from(e.to_string()))?;

        log::info!(
            "Parallel DB load finished in {}ms",
            start.elapsed().as_millis()
        );

        // Wrap in Arc for sharing
        let aircraft: Vec<Arc<Aircraft>> = aircraft_raw.into_iter().map(Arc::new).collect();
        let airports: Vec<Arc<Airport>> = airports_raw.into_iter().map(Arc::new).collect();

        // Create spatial index for airports (Fast in-memory)
        let spatial_airports = rstar::RTree::bulk_load(
            airports
                .iter()
                .map(|airport| crate::models::airport::SpatialAirport {
                    airport: Arc::clone(airport),
                })
                .collect(),
        );

        // Create runways hashmap (Fast in-memory)
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

        // Generate UI items
        // Aircraft items are relatively few (hundreds), so we keep them here
        let aircraft_items = services::aircraft_service::transform_to_list_items(&aircraft);

        // Routes and History require DB/Calculation, perform sequentially now or could parallelize
        let route_items = DataOperations::generate_random_routes(&route_generator, &aircraft, None);
        let history_items =
            DataOperations::load_history_data(&mut database_pool, &aircraft, &airports)?;

        // Note: airport_items are NO LONGER generated here to save startup time (approx 8%).
        // They are generated on-demand via `generate_airport_items`.

        Ok(Self {
            database_pool,
            route_generator,
            aircraft,
            airports,
            aircraft_items,
            route_items,
            history_items,
            cached_statistics: None,
            statistics_dirty: true,
        })
    }

    // --- Data Access ---

    /// Returns a slice of all loaded airports.
    pub fn airports(&self) -> &[Arc<Airport>] {
        &self.airports
    }

    /// Returns a slice of all loaded aircraft.
    pub fn aircraft(&self) -> &[Arc<Aircraft>] {
        &self.aircraft
    }

    /// Returns a slice of the currently loaded route items.
    pub fn route_items(&self) -> &[ListItemRoute] {
        &self.route_items
    }

    /// Replaces the current route items with a new set.
    pub fn set_route_items(&mut self, routes: Vec<ListItemRoute>) {
        self.route_items = routes;
    }

    // Constants for route fade-in animation
    const APPEND_STEP: std::time::Duration = std::time::Duration::from_millis(50);
    const APPEND_CATCHUP_STEP: std::time::Duration = std::time::Duration::from_millis(20);
    const MAX_QUEUE_DELAY: std::time::Duration = std::time::Duration::from_millis(500);

    /// Appends new routes to the existing list of route items.
    pub fn append_route_items(&mut self, mut new_routes: Vec<ListItemRoute>) {
        if let Some(last_route) = self.route_items.last() {
            let now = std::time::Instant::now();

            // Calculate start time based on last item, but prevented from being in the past
            // and capped to avoid infinite queuing delay.
            let base_time = last_route.created_at.max(now);
            let start_time = if base_time > now + Self::MAX_QUEUE_DELAY {
                now + Self::MAX_QUEUE_DELAY
            } else {
                base_time
            };

            // Use faster step if we are catching up
            let step = if start_time > now {
                Self::APPEND_CATCHUP_STEP
            } else {
                Self::APPEND_STEP
            };

            for (i, route) in new_routes.iter_mut().enumerate() {
                route.created_at = start_time + (step * (i as u32 + 1));
            }
        }
        self.route_items.extend(new_routes);
    }

    /// Returns a slice of the currently loaded history items.
    pub fn history_items(&self) -> &[ListItemHistory] {
        &self.history_items
    }

    /// Generates airport list items on demand.
    ///
    /// This operation is CPU intensive and should be run in a background thread.
    pub fn generate_airport_items(&self) -> Vec<ListItemAirport> {
        services::airport_service::transform_to_list_items_with_runways(
            &self.airports,
            &self.route_generator.longest_runway_cache,
        )
    }

    /// Returns a slice of the currently loaded aircraft items, formatted for the UI.
    pub fn aircraft_items(&self) -> &[ListItemAircraft] {
        &self.aircraft_items
    }

    /// Returns a mutable reference to the database pool for direct database operations.
    pub fn database_pool(&mut self) -> &mut DatabasePool {
        &mut self.database_pool
    }

    /// Returns a clone of the database pool.
    pub fn clone_pool(&self) -> DatabasePool {
        self.database_pool.clone()
    }

    /// Returns a shared reference to the `RouteGenerator`.
    pub fn route_generator(&self) -> &Arc<RouteGenerator> {
        &self.route_generator
    }

    // --- Business Logic Methods ---

    /// Returns a specified number of randomly selected airports.
    ///
    /// # Arguments
    ///
    /// * `count` - The number of random airports to return.
    ///
    /// # Returns
    ///
    /// A `Vec<Arc<Airport>>` containing the randomly selected airports.
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
        DataOperations::toggle_aircraft_flown_status(&mut self.database_pool, aircraft_id)?;
        self.reload_aircraft_data()?;
        Ok(())
    }

    pub fn mark_all_aircraft_as_not_flown(&mut self) -> Result<(), Box<dyn Error>> {
        DataOperations::mark_all_aircraft_as_not_flown(&mut self.database_pool)?;
        self.reload_aircraft_data()?;
        Ok(())
    }

    fn reload_aircraft_data(&mut self) -> Result<(), Box<dyn Error>> {
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
        DataOperations::add_history_entry(
            &mut self.database_pool,
            aircraft,
            departure,
            destination,
        )?;

        self.history_items = DataOperations::load_history_data(
            &mut self.database_pool,
            &self.aircraft,
            &self.airports,
        )?;

        self.invalidate_statistics_cache();

        Ok(())
    }

    pub fn mark_route_as_flown(&mut self, route: &ListItemRoute) -> Result<(), Box<dyn Error>> {
        DataOperations::mark_route_as_flown(&mut self.database_pool, route)?;

        self.history_items = DataOperations::load_history_data(
            &mut self.database_pool,
            &self.aircraft,
            &self.airports,
        )?;

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
        Ok(self.cached_statistics.as_ref().unwrap().clone())
    }

    pub fn invalidate_statistics_cache(&mut self) {
        self.statistics_dirty = true;
        self.cached_statistics = None;
    }

    pub fn get_setting(&mut self, key_str: &str) -> Result<Option<String>, Box<dyn Error>> {
        let mut conn = self.database_pool.aircraft_pool.get()?;
        let result = settings
            .filter(key.eq(key_str))
            .first::<Setting>(&mut conn)
            .optional()?;
        Ok(result.map(|s| s.value))
    }

    pub fn set_setting(&mut self, key_str: &str, value_str: &str) -> Result<(), Box<dyn Error>> {
        let mut conn = self.database_pool.aircraft_pool.get()?;
        let new_setting = Setting {
            key: key_str.to_string(),
            value: value_str.to_string(),
        };
        diesel::insert_into(settings)
            .values(&new_setting)
            .on_conflict(key)
            .do_update()
            .set(&new_setting)
            .execute(&mut conn)?;
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

    // Note: filter_airport_items takes a slice, so it's generic, but we removed self.airport_items
    // Users of this function must provide the list to filter.
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
