/// High-level data operations that combine multiple lower-level operations.
/// This module provides business-logic operations for the GUI layer.
use std::sync::Arc;

use crate::database::DatabasePool;
use crate::gui::data::{ListItemHistory, ListItemRoute};
use crate::models::{Aircraft, Airport};
use crate::modules::routes::RouteGenerator;
use crate::traits::HistoryOperations;
use std::collections::HashMap;

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
                    departure_icao: history.departure_icao,
                    departure_airport_name,
                    arrival_icao: history.arrival_icao,
                    arrival_airport_name,
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
        let total_flights = history.len();
        let total_distance: i32 = history.iter().map(|h| h.distance.unwrap_or(0)).sum();
        let average_flight_distance = if total_flights > 0 {
            total_distance as f64 / total_flights as f64
        } else {
            0.0
        };

        // Find longest and shortest flights
        let longest_flight = history
            .iter()
            .max_by_key(|h| h.distance)
            .map(|h| format!("{} to {}", h.departure_icao, h.arrival_icao));
        let shortest_flight = history
            .iter()
            .min_by_key(|h| h.distance)
            .map(|h| format!("{} to {}", h.departure_icao, h.arrival_icao));

        // Build aircraft lookup map for O(1) lookups
        let aircraft_map: HashMap<i32, &Arc<Aircraft>> =
            aircraft.iter().map(|a| (a.id, a)).collect();

        // Find most flown aircraft
        let mut aircraft_counts = HashMap::new();
        for h in history {
            *aircraft_counts.entry(h.aircraft).or_insert(0) += 1;
        }
        let mut aircraft_counts: Vec<(i32, usize)> = aircraft_counts.into_iter().collect();
        aircraft_counts.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

        let most_flown_aircraft = aircraft_counts.first().and_then(|(id, _)| {
            aircraft_map
                .get(id)
                .map(|a| format!("{} {}", a.manufacturer, a.variant))
        });

        // Find favorite departure airport
        let mut departure_counts = HashMap::new();
        for h in history {
            *departure_counts
                .entry(h.departure_icao.as_str())
                .or_insert(0) += 1;
        }
        let mut departure_counts: Vec<(&str, usize)> = departure_counts.into_iter().collect();
        departure_counts.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
        let favorite_departure_airport = departure_counts.first().map(|(icao, _)| icao.to_string());

        // Find favorite arrival airport
        let mut arrival_counts = HashMap::new();
        for h in history {
            *arrival_counts.entry(h.arrival_icao.as_str()).or_insert(0) += 1;
        }
        let mut arrival_counts: Vec<(&str, usize)> = arrival_counts.into_iter().collect();
        arrival_counts.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
        let favorite_arrival_airport = arrival_counts.first().map(|(icao, _)| icao.to_string());

        // Find most visited airport
        let mut airport_counts = HashMap::new();
        for h in history {
            *airport_counts.entry(h.departure_icao.as_str()).or_insert(0) += 1;
            *airport_counts.entry(h.arrival_icao.as_str()).or_insert(0) += 1;
        }
        let mut airport_counts: Vec<(&str, usize)> = airport_counts.into_iter().collect();
        airport_counts.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));

        let most_visited_airport = airport_counts.first().map(|(icao, _)| icao.to_string());

        FlightStatistics {
            total_flights,
            total_distance,
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
#[derive(Debug, Clone)]
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
