use std::collections::HashMap;
use std::sync::Arc;

use crate::database::DatabasePool;
use crate::gui::data::{
    ListItemAircraft, ListItemAirport, ListItemHistory, ListItemRoute, TableItem,
};
use crate::models::{Aircraft, Airport};
use crate::modules::routes::RouteGenerator;
use crate::traits::{AircraftOperations, HistoryOperations};

/// Service for handling route-related operations.
pub struct RouteService {
    route_generator: RouteGenerator,
}

impl RouteService {
    /// Creates a new `RouteService` with the given `RouteGenerator`.
    ///
    /// # Arguments
    ///
    /// * `route_generator` - The route generator to use for route operations
    ///
    /// # Returns
    ///
    /// Returns a new `RouteService` instance.
    pub const fn new(route_generator: RouteGenerator) -> Self {
        Self { route_generator }
    }

    /// Gets a reference to the available airports.
    ///
    /// # Returns
    ///
    /// Returns a slice of available airports.
    pub fn get_available_airports(&self) -> &[Arc<Airport>] {
        &self.route_generator.all_airports
    }

    /// Converts a vector of routes into a vector of route table items.
    ///
    /// # Arguments
    ///
    /// * `routes` - The routes to convert
    ///
    /// # Returns
    ///
    /// Returns a vector of route table items wrapped in Arc.
    fn wrap_routes_as_table_items(routes: Vec<ListItemRoute>) -> Vec<Arc<TableItem>> {
        routes
            .into_iter()
            .map(|route| Arc::new(TableItem::Route(route)))
            .collect()
    }

    /// Generates random routes for all aircraft.
    ///
    /// # Arguments
    ///
    /// * `aircraft` - All available aircraft
    /// * `departure_icao` - Optional departure airport ICAO
    ///
    /// # Returns
    ///
    /// Returns a vector of route table items.
    pub fn generate_random_routes(
        &self,
        aircraft: &[Arc<Aircraft>],
        departure_icao: Option<&str>,
    ) -> Vec<Arc<TableItem>> {
        let routes = self
            .route_generator
            .generate_random_routes(aircraft, departure_icao);
        Self::wrap_routes_as_table_items(routes)
    }

    /// Generates routes for not flown aircraft.
    ///
    /// # Arguments
    ///
    /// * `aircraft` - All available aircraft
    /// * `departure_icao` - Optional departure airport ICAO
    ///
    /// # Returns
    ///
    /// Returns a vector of route table items.
    pub fn generate_not_flown_routes(
        &self,
        aircraft: &[Arc<Aircraft>],
        departure_icao: Option<&str>,
    ) -> Vec<Arc<TableItem>> {
        let routes = self
            .route_generator
            .generate_random_not_flown_aircraft_routes(aircraft, departure_icao);
        Self::wrap_routes_as_table_items(routes)
    }

    /// Generates routes for a specific aircraft.
    ///
    /// # Arguments
    ///
    /// * `aircraft` - The specific aircraft
    /// * `departure_icao` - Optional departure airport ICAO
    ///
    /// # Returns
    ///
    /// Returns a vector of route table items.
    pub fn generate_routes_for_aircraft(
        &self,
        aircraft: &Arc<Aircraft>,
        departure_icao: Option<&str>,
    ) -> Vec<Arc<TableItem>> {
        let routes = self
            .route_generator
            .generate_routes_for_aircraft(aircraft, departure_icao);
        Self::wrap_routes_as_table_items(routes)
    }

    /// Marks a route as flown in the database and updates the aircraft.
    ///
    /// # Arguments
    ///
    /// * `database_pool` - The database pool
    /// * `route` - The route to mark as flown
    ///
    /// # Returns
    ///
    /// Returns a Result indicating success or failure.
    pub fn mark_route_as_flown(
        database_pool: &mut DatabasePool,
        route: &ListItemRoute,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Add route to history
        database_pool.add_to_history(
            route.departure.as_ref(),
            route.destination.as_ref(),
            route.aircraft.as_ref(),
        )?;

        // Update aircraft as flown
        let mut aircraft = (*route.aircraft).clone();
        aircraft.date_flown = Some(chrono::Local::now().format("%Y-%m-%d").to_string());
        aircraft.flown = 1;

        database_pool.update_aircraft(&aircraft)?;

        Ok(())
    }

    /// Loads airport list items from airports.
    ///
    /// # Arguments
    ///
    /// * `airports` - The airports to convert
    ///
    /// # Returns
    ///
    /// Returns a vector of airport table items.
    pub fn load_airport_items(airports: &[Arc<Airport>]) -> Vec<Arc<TableItem>> {
        airports
            .iter()
            .map(|airport| {
                Arc::new(TableItem::Airport(ListItemAirport::new(
                    airport.Name.clone(),
                    airport.ICAO.clone(),
                    "N/A".to_string(), // We don't have runway data in this context
                )))
            })
            .collect()
    }

    /// Generates random airports with runway information.
    ///
    /// # Arguments
    ///
    /// * `amount` - The number of airports to generate
    ///
    /// # Returns
    ///
    /// Returns a vector of airport table items with runway information.
    pub fn generate_random_airports(&self, amount: usize) -> Vec<Arc<TableItem>> {
        use rand::prelude::*;

        let mut rng = rand::rng();

        // If we have enough airports for unique selection, use choose_multiple
        // Otherwise, fall back to choose with potential duplicates
        let airport_items = if amount <= self.route_generator.all_airports.len() {
            // Use choose_multiple for unique airports when possible
            self.route_generator
                .all_airports
                .choose_multiple(&mut rng, amount)
                .map(|airport| self.create_airport_item_with_runway(airport))
                .collect()
        } else {
            // Fall back to choose with potential duplicates when we need more than available
            (0..amount)
                .filter_map(|_| {
                    self.route_generator
                        .all_airports
                        .choose(&mut rng)
                        .map(|airport| self.create_airport_item_with_runway(airport))
                })
                .collect()
        };

        airport_items
    }

    /// Helper method to create an airport item with runway information.
    ///
    /// # Arguments
    ///
    /// * `airport` - The airport to create an item for
    ///
    /// # Returns
    ///
    /// Returns an airport table item with runway information.
    fn create_airport_item_with_runway(&self, airport: &Airport) -> Arc<TableItem> {
        // Get the longest runway length for this airport using and_then
        let longest_runway_length = self
            .route_generator
            .all_runways
            .get(&airport.ID)
            .and_then(|runways| runways.iter().map(|runway| runway.Length).max())
            .map_or_else(|| "N/A".to_string(), |length| format!("{length} ft"));

        Arc::new(TableItem::Airport(ListItemAirport::new(
            airport.Name.clone(),
            airport.ICAO.clone(),
            longest_runway_length,
        )))
    }

    /// Loads history items from the database.
    ///
    /// # Arguments
    ///
    /// * `database_pool` - The database pool
    /// * `aircraft` - All available aircraft for name lookups
    ///
    /// # Returns
    ///
    /// Returns a Result with history table items or an error.
    pub fn load_history_items(
        database_pool: &mut DatabasePool,
        aircraft: &[Arc<Aircraft>],
    ) -> Result<Vec<Arc<TableItem>>, Box<dyn std::error::Error>> {
        let history = database_pool.get_history()?;

        // Create a HashMap for O(1) aircraft lookups
        let aircraft_map: HashMap<i32, &Arc<Aircraft>> = aircraft
            .iter()
            .map(|aircraft| (aircraft.id, aircraft))
            .collect();

        let history_items = history
            .into_iter()
            .map(|history| {
                // Use HashMap for O(1) aircraft lookup
                let aircraft_name = aircraft_map.get(&history.aircraft).map_or_else(
                    || format!("Unknown Aircraft (ID: {})", history.aircraft),
                    |aircraft| format!("{} {}", aircraft.manufacturer, aircraft.variant),
                );

                Arc::new(TableItem::History(ListItemHistory {
                    id: history.id.to_string(),
                    departure_icao: history.departure_icao,
                    arrival_icao: history.arrival_icao,
                    aircraft_name,
                    date: history.date,
                }))
            })
            .collect();

        Ok(history_items)
    }

    /// Creates an aircraft list item from a single aircraft.
    ///
    /// # Arguments
    ///
    /// * `aircraft` - The aircraft to convert
    ///
    /// # Returns
    ///
    /// Returns an aircraft table item.
    pub fn create_aircraft_item(aircraft: &Aircraft) -> Arc<TableItem> {
        Arc::new(TableItem::Aircraft(ListItemAircraft::from_aircraft(
            aircraft,
        )))
    }
}
