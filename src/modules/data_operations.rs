/// High-level data operations that combine multiple lower-level operations.
/// This module provides business-logic operations for the GUI layer.
use std::collections::HashMap;
use std::sync::Arc;

use crate::database::DatabasePool;
use crate::models::{Aircraft, Airport};
use crate::traits::HistoryOperations;

#[cfg(feature = "gui")]
use crate::{
    gui::data::{ListItemHistory, ListItemRoute},
    modules::routes::RouteGenerator,
};

/// Provides high-level data operations that combine multiple lower-level
/// database and business logic functions.
///
/// This struct serves as a central point for the GUI layer to interact with
/// the application's data, encapsulating complex operations into simpler methods.
pub struct DataOperations;

impl DataOperations {
    /// Marks a route as flown by adding it to history and updating the aircraft.
    ///
    /// # Arguments
    ///
    /// * `database_pool` - The database pool
    /// * `route` - The route to mark as flown
    ///
    /// # Returns
    ///
    /// Returns a Result indicating success or failure.
    #[cfg(feature = "gui")]
    pub fn mark_route_as_flown(
        database_pool: &mut DatabasePool,
        route: &ListItemRoute,
    ) -> Result<(), Box<dyn std::error::Error>> {
        crate::modules::history::mark_flight_completed(
            database_pool,
            route.departure.as_ref(),
            route.destination.as_ref(),
            route.aircraft.as_ref(),
        )
    }

    /// Adds a new flight log entry to the history.
    ///
    /// # Arguments
    ///
    /// * `database_pool` - The database pool
    /// * `aircraft` - The aircraft used for the flight
    /// * `departure` - The departure airport
    /// * `destination` - The destination airport
    ///
    /// # Returns
    ///
    /// Returns a Result indicating success or failure.
    pub fn add_history_entry(
        database_pool: &mut DatabasePool,
        aircraft: &Arc<Aircraft>,
        departure: &Arc<Airport>,
        destination: &Arc<Airport>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        crate::modules::history::add_flight_log_entry(
            database_pool,
            departure,
            destination,
            aircraft,
        )
    }

    /// Toggles the flown status of an aircraft.
    ///
    /// # Arguments
    ///
    /// * `database_pool` - The database pool
    /// * `aircraft_id` - The ID of the aircraft to toggle
    ///
    /// # Returns
    ///
    /// Returns a Result indicating success or failure.
    pub fn toggle_aircraft_flown_status(
        database_pool: &mut DatabasePool,
        aircraft_id: i32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use crate::date_utils;
        use crate::traits::AircraftOperations;

        // Get the current aircraft from the database
        let mut aircraft = database_pool.get_aircraft_by_id(aircraft_id)?;

        // Toggle the flown status
        aircraft.flown = i32::from(aircraft.flown == 0);

        // Update date_flown based on flown status
        if aircraft.flown == 1 {
            // Set current date in UTC when marking as flown
            aircraft.date_flown = Some(date_utils::get_current_date_utc());
        } else {
            // Clear date when marking as not flown
            aircraft.date_flown = None;
        }

        // Update in database
        database_pool.update_aircraft(&aircraft)?;

        Ok(())
    }

    /// Marks all aircraft as not flown.
    ///
    /// # Arguments
    ///
    /// * `database_pool` - The database pool
    ///
    /// # Returns
    ///
    /// Returns a Result indicating success or failure.
    pub fn mark_all_aircraft_as_not_flown(
        database_pool: &mut DatabasePool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use crate::traits::AircraftOperations;
        database_pool.mark_all_aircraft_not_flown()?;
        Ok(())
    }

    /// Loads history data and converts it to UI list items.
    ///
    /// # Arguments
    ///
    /// * `database_pool` - The database pool
    /// * `aircraft` - All available aircraft for name lookups
    ///
    /// # Returns
    ///
    /// Returns a Result with history items or an error.
    #[cfg(feature = "gui")]
    pub fn load_history_data(
        database_pool: &mut DatabasePool,
        aircraft: &[Arc<Aircraft>],
        airports: &[Arc<Airport>],
    ) -> Result<Vec<ListItemHistory>, Box<dyn std::error::Error>> {
        let history = database_pool.get_history()?;

        // Create a HashMap for O(1) aircraft lookups
        let aircraft_map: HashMap<i32, &Arc<Aircraft>> =
            aircraft.iter().map(|a| (a.id, a)).collect();

        // Create a HashMap for O(1) airport lookups by ICAO
        let airport_map: HashMap<&str, &Arc<Airport>> =
            airports.iter().map(|a| (a.ICAO.as_str(), a)).collect();

        // Helper closure to look up airport names
        let get_airport_name = |icao: &str| {
            airport_map
                .get(icao)
                .map_or_else(|| "Unknown Airport".to_string(), |a| a.Name.clone())
        };

        let history_items = history
            .into_iter()
            .map(|history| {
                // Use HashMap for O(1) aircraft lookup
                let aircraft_name = aircraft_map.get(&history.aircraft).map_or_else(
                    || format!("Unknown Aircraft (ID: {})", history.aircraft),
                    |a| format!("{} {}", a.manufacturer, a.variant),
                );

                let departure_airport_name = get_airport_name(&history.departure_icao);
                let arrival_airport_name = get_airport_name(&history.arrival_icao);

                ListItemHistory {
                    id: history.id.to_string(),
                    departure_info: format!(
                        "{} ({})",
                        departure_airport_name, history.departure_icao
                    ),
                    departure_icao: history.departure_icao,
                    arrival_info: format!("{} ({})", arrival_airport_name, history.arrival_icao),
                    arrival_icao: history.arrival_icao,
                    aircraft_name,
                    date: history.date,
                }
            })
            .collect();

        Ok(history_items)
    }

    /// Generates random routes using the route generator.
    ///
    /// # Arguments
    ///
    /// * `route_generator` - The route generator
    /// * `aircraft` - All available aircraft
    /// * `departure_icao` - Optional departure airport ICAO
    ///
    /// # Returns
    ///
    /// Returns a vector of route items.
    #[cfg(feature = "gui")]
    pub fn generate_random_routes(
        route_generator: &RouteGenerator,
        aircraft: &[Arc<Aircraft>],
        departure_icao: Option<&str>,
    ) -> Vec<ListItemRoute> {
        route_generator.generate_random_routes(aircraft, departure_icao)
    }

    /// Generates routes for not flown aircraft.
    ///
    /// # Arguments
    ///
    /// * `route_generator` - The route generator
    /// * `aircraft` - All available aircraft
    /// * `departure_icao` - Optional departure airport ICAO
    ///
    /// # Returns
    ///
    /// Returns a vector of route items.
    #[cfg(feature = "gui")]
    pub fn generate_not_flown_routes(
        route_generator: &RouteGenerator,
        aircraft: &[Arc<Aircraft>],
        departure_icao: Option<&str>,
    ) -> Vec<ListItemRoute> {
        route_generator.generate_random_not_flown_aircraft_routes(aircraft, departure_icao)
    }

    /// Generates routes for a specific aircraft.
    ///
    /// # Arguments
    ///
    /// * `route_generator` - The route generator  
    /// * `aircraft` - The specific aircraft
    /// * `departure_icao` - Optional departure airport ICAO
    ///
    /// # Returns
    ///
    /// Returns a vector of route items.
    #[cfg(feature = "gui")]
    pub fn generate_routes_for_aircraft(
        route_generator: &RouteGenerator,
        aircraft: &Arc<Aircraft>,
        departure_icao: Option<&str>,
    ) -> Vec<ListItemRoute> {
        route_generator.generate_routes_for_aircraft(aircraft, departure_icao)
    }

    /// Generates random airports for display.
    ///
    /// # Arguments
    ///
    /// * `airports` - All available airports
    /// * `amount` - The number of airports to generate
    ///
    /// # Returns
    ///
    /// Returns a vector of randomly selected airports.
    pub fn generate_random_airports(airports: &[Arc<Airport>], amount: usize) -> Vec<Arc<Airport>> {
        use rand::prelude::*;
        let mut rng = rand::rng();

        if amount <= airports.len() {
            airports
                .choose_multiple(&mut rng, amount)
                .cloned()
                .collect()
        } else {
            (0..amount)
                .filter_map(|_| airports.choose(&mut rng).cloned())
                .collect()
        }
    }

    /// Loads aircraft data and converts it to UI list items.
    ///
    /// # Arguments
    ///
    /// * `aircraft` - All available aircraft
    ///
    /// # Returns
    ///
    /// Returns a Result with aircraft items or an error.
    #[cfg(feature = "gui")]
    pub fn load_aircraft_data(
        aircraft: &[Arc<Aircraft>],
    ) -> Result<Vec<crate::gui::data::ListItemAircraft>, Box<dyn std::error::Error>> {
        use crate::gui::services::aircraft_service;
        Ok(aircraft_service::transform_to_list_items(aircraft))
    }

    /// Calculates flight statistics from the history data.
    ///
    /// # Arguments
    ///
    /// * `database_pool` - The database pool
    /// * `aircraft` - All available aircraft for name lookups
    ///
    /// # Returns
    ///
    /// Returns a Result with flight statistics or an error.
    pub fn calculate_statistics(
        database_pool: &mut DatabasePool,
        aircraft: &[Arc<Aircraft>],
    ) -> Result<FlightStatistics, Box<dyn std::error::Error + Send + Sync>> {
        let history = database_pool.get_history()?;
        Ok(Self::calculate_statistics_from_history(&history, aircraft))
    }

    /// Calculates flight statistics from a given history slice.
    /// This function is separated for testability and to avoid code duplication.
    ///
    /// # Arguments
    ///
    /// * `history` - The flight history records
    /// * `aircraft` - All available aircraft for name lookups
    ///
    /// # Returns
    ///
    /// Returns the calculated flight statistics.
    pub fn calculate_statistics_from_history(
        history: &[crate::models::History],
        aircraft: &[Arc<Aircraft>],
    ) -> FlightStatistics {
        if history.is_empty() {
            return FlightStatistics::default();
        }

        let mut stats = StatsAccumulator::default();

        for h in history {
            stats.accumulate(h);
        }

        stats.finalize(aircraft)
    }
}

/// Helper struct to accumulate statistics during a single pass over history.
struct StatsAccumulator<'a> {
    count: usize,
    total_distance: i32,
    min_distance: i32,
    max_distance: i32,
    shortest_flight_record: Option<&'a crate::models::History>,
    longest_flight_record: Option<&'a crate::models::History>,
    aircraft_counts: HashMap<i32, usize>,
    departure_counts: HashMap<&'a str, usize>,
    arrival_counts: HashMap<&'a str, usize>,
    airport_counts: HashMap<&'a str, usize>,
}

impl<'a> Default for StatsAccumulator<'a> {
    fn default() -> Self {
        Self {
            count: 0,
            total_distance: 0,
            min_distance: i32::MAX,
            max_distance: i32::MIN,
            shortest_flight_record: None,
            longest_flight_record: None,
            aircraft_counts: HashMap::new(),
            departure_counts: HashMap::new(),
            arrival_counts: HashMap::new(),
            airport_counts: HashMap::new(),
        }
    }
}

impl<'a> StatsAccumulator<'a> {
    fn accumulate(&mut self, h: &'a crate::models::History) {
        self.count += 1;
        let dist = h.distance.unwrap_or(0);
        self.total_distance += dist;

        // Track min/max distance
        // min_by_key returns first element on tie, so use <
        if dist < self.min_distance {
            self.min_distance = dist;
            self.shortest_flight_record = Some(h);
        }
        // max_by_key returns last element on tie, so use >=
        if dist >= self.max_distance {
            self.max_distance = dist;
            self.longest_flight_record = Some(h);
        }

        *self.aircraft_counts.entry(h.aircraft).or_default() += 1;
        *self
            .departure_counts
            .entry(h.departure_icao.as_str())
            .or_default() += 1;
        *self
            .arrival_counts
            .entry(h.arrival_icao.as_str())
            .or_default() += 1;

        *self
            .airport_counts
            .entry(h.departure_icao.as_str())
            .or_default() += 1;
        *self
            .airport_counts
            .entry(h.arrival_icao.as_str())
            .or_default() += 1;
    }

    fn finalize(self, aircraft: &[Arc<Aircraft>]) -> FlightStatistics {
        let total_flights = self.count;
        let average_flight_distance = if total_flights > 0 {
            self.total_distance as f64 / total_flights as f64
        } else {
            0.0
        };

        let longest_flight = self
            .longest_flight_record
            .map(|h| format!("{} to {}", h.departure_icao, h.arrival_icao));
        let shortest_flight = self
            .shortest_flight_record
            .map(|h| format!("{} to {}", h.departure_icao, h.arrival_icao));

        // Helper to find key with max value in map, breaking ties by key (ascending)
        fn find_max_str(map: HashMap<&str, usize>) -> Option<String> {
            map.into_iter()
                .max_by(|a, b| a.1.cmp(&b.1).then_with(|| b.0.cmp(a.0)))
                .map(|(k, _)| k.to_string())
        }

        let favorite_departure_airport = find_max_str(self.departure_counts);
        let favorite_arrival_airport = find_max_str(self.arrival_counts);
        let most_visited_airport = find_max_str(self.airport_counts);

        // Find most flown aircraft
        let most_flown_aircraft_id = self
            .aircraft_counts
            .into_iter()
            .max_by(|a, b| a.1.cmp(&b.1).then_with(|| b.0.cmp(&a.0)))
            .map(|(id, _)| id);

        let most_flown_aircraft = most_flown_aircraft_id.and_then(|id| {
            aircraft
                .iter()
                .find(|a| a.id == id)
                .map(|a| format!("{} {}", a.manufacturer, a.variant))
        });

        FlightStatistics {
            total_flights,
            total_distance: self.total_distance,
            most_flown_aircraft,
            most_visited_airport,
            average_flight_distance,
            longest_flight,
            shortest_flight,
            favorite_departure_airport,
            favorite_arrival_airport,
        }
    }
}

/// Represents a collection of statistics calculated from flight history data.
///
/// This struct holds various metrics about flights, such as totals, averages,
/// and favorites, providing a comprehensive overview of the user's flight activity.
#[derive(Debug, Clone, Default)]
pub struct FlightStatistics {
    /// The total number of flights recorded in the history.
    pub total_flights: usize,
    /// The total distance of all flights, in nautical miles.
    pub total_distance: i32,
    /// The name of the aircraft that has been flown the most times.
    pub most_flown_aircraft: Option<String>,
    /// The ICAO code of the airport that has been visited most frequently.
    pub most_visited_airport: Option<String>,
    /// The average distance of a single flight, in nautical miles.
    pub average_flight_distance: f64,
    /// A string representing the longest flight, e.g., "ICAO to ICAO".
    pub longest_flight: Option<String>,
    /// A string representing the shortest flight, e.g., "ICAO to ICAO".
    pub shortest_flight: Option<String>,
    /// The ICAO code of the most frequent departure airport.
    pub favorite_departure_airport: Option<String>,
    /// The ICAO code of the most frequent arrival airport.
    pub favorite_arrival_airport: Option<String>,
}
