/// High-level data operations that combine multiple lower-level operations.
/// This module provides business-logic operations for the GUI layer.
use std::sync::Arc;

use crate::database::DatabasePool;
use crate::gui::data::{ListItemHistory, ListItemRoute};
use crate::models::{Aircraft, Airport};
use crate::modules::routes::RouteGenerator;
use crate::traits::HistoryOperations;
use std::collections::HashMap;

/// High-level data operations for the GUI.
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
    ) -> Result<Vec<ListItemHistory>, Box<dyn std::error::Error>> {
        let history = database_pool.get_history()?;

        // Create a HashMap for O(1) aircraft lookups
        let aircraft_map: std::collections::HashMap<i32, &Arc<Aircraft>> = aircraft
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

                ListItemHistory {
                    id: history.id.to_string(),
                    departure_icao: history.departure_icao,
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
    ) -> Result<FlightStatistics, Box<dyn std::error::Error>> {
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

        // Build aircraft lookup map for O(1) lookups
        let aircraft_map: HashMap<i32, &Arc<Aircraft>> =
            aircraft.iter().map(|a| (a.id, a)).collect();

        // Find most flown aircraft with deterministic tie-breaking
        let mut aircraft_counts: Vec<(i32, usize)> = history
            .iter()
            .map(|h| h.aircraft)
            .fold(HashMap::new(), |mut acc, id| {
                *acc.entry(id).or_insert(0) += 1;
                acc
            })
            .into_iter()
            .collect();

        // Sort by count (descending), then by aircraft ID (ascending) for deterministic results
        aircraft_counts.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

        let most_flown_aircraft = aircraft_counts.first().and_then(|(id, _)| {
            aircraft_map
                .get(id)
                .map(|a| format!("{} {}", a.manufacturer, a.variant))
        });

        // Find most visited airport with deterministic tie-breaking
        let mut airport_counts: Vec<(String, usize)> = history
            .iter()
            .flat_map(|h| [h.departure_icao.as_str(), h.arrival_icao.as_str()])
            .fold(HashMap::<&str, usize>::new(), |mut acc, icao| {
                *acc.entry(icao).or_default() += 1;
                acc
            })
            .into_iter()
            .map(|(icao, count)| (icao.to_string(), count))
            .collect();

        // Sort by count (descending), then by airport ICAO (ascending) for deterministic results
        airport_counts.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

        let most_visited_airport = airport_counts.first().map(|(icao, _)| icao.clone());

        FlightStatistics {
            total_flights,
            total_distance,
            most_flown_aircraft,
            most_visited_airport,
        }
    }
}

/// Statistics about flight history.
#[derive(Debug, Clone)]
pub struct FlightStatistics {
    pub total_flights: usize,
    pub total_distance: i32,
    pub most_flown_aircraft: Option<String>,
    pub most_visited_airport: Option<String>,
}
