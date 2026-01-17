mod common;

use common::{create_test_aircraft, create_test_airport, create_test_history};
use flight_planner::models::{Aircraft, Airport, History};
use flight_planner::modules::data_operations::DataOperations;
use flight_planner::test_helpers;
use flight_planner::traits::{AircraftOperations, AirportOperations};
use std::sync::Arc;

#[test]
fn test_calculate_statistics_from_history() {
    // Arrange
    let aircraft = Arc::new(create_test_aircraft(1, "Boeing", "737", "B737"));
    let aircraft_list = vec![aircraft];

    let history = vec![
        create_test_history(1, 1, "JFK", "LHR", "2023-01-01", 3000),
        create_test_history(2, 1, "LHR", "CDG", "2023-01-02", 200),
    ];

    // Act
    let stats = DataOperations::calculate_statistics_from_history(&history, &aircraft_list);

    assert_eq!(stats.total_flights, 2);
    assert_eq!(stats.total_distance, 3200);
    assert_eq!(stats.average_flight_distance, 1600.0);
    assert_eq!(stats.most_flown_aircraft.as_deref(), Some("Boeing 737"));
    assert_eq!(stats.longest_flight.as_deref(), Some("JFK to LHR"));
    assert_eq!(stats.shortest_flight.as_deref(), Some("LHR to CDG"));

    // Testing favorites is a bit tricky with ties, but let's check basic structure
    assert!(stats.favorite_departure_airport.is_some());
    assert!(stats.favorite_arrival_airport.is_some());
    assert!(stats.most_visited_airport.is_some());
}

#[test]
fn test_calculate_statistics_empty() {
    let aircraft_list: Vec<Arc<Aircraft>> = Vec::new();
    let history: Vec<History> = Vec::new();

    let stats = DataOperations::calculate_statistics_from_history(&history, &aircraft_list);

    assert_eq!(stats.total_flights, 0);
    assert_eq!(stats.total_distance, 0);
    assert_eq!(stats.average_flight_distance, 0.0);
    assert!(stats.most_flown_aircraft.is_none());
}

#[test]
fn test_generate_random_airports() {
    let mut airports = Vec::new();
    for i in 0..10 {
        airports.push(Arc::new(create_test_airport(
            i,
            &format!("Airport {}", i),
            &format!("ICAO{}", i),
        )));
    }

    // Request fewer than available
    let random_subset = DataOperations::generate_random_airports(&airports, 3);
    assert_eq!(random_subset.len(), 3);

    // Request more than available (should fallback to replacement or just return randoms? impl uses choose vs choose_multiple)
    // Actually implementation says: if amount <= len, use choose_multiple (unique).
    // If amount > len, use (0..amount).filter_map(|_| choose). So it allows duplicates if requested > len.
    let random_many = DataOperations::generate_random_airports(&airports, 15);
    assert_eq!(random_many.len(), 15);
}

#[test]
fn test_toggle_aircraft_flown_status() {
    let mut db = test_helpers::setup_database();

    // Get an aircraft (ID 1 should exist from setup_database)
    let aircraft = db.get_aircraft_by_id(1).expect("Aircraft 1 should exist");
    assert_eq!(aircraft.flown, 0); // Default is 0

    // Toggle to flown
    DataOperations::toggle_aircraft_flown_status(&mut db, 1).unwrap();

    let updated = db.get_aircraft_by_id(1).unwrap();
    assert_eq!(updated.flown, 1);
    assert!(updated.date_flown.is_some());

    // Toggle back to not flown
    DataOperations::toggle_aircraft_flown_status(&mut db, 1).unwrap();

    let reverted = db.get_aircraft_by_id(1).unwrap();
    assert_eq!(reverted.flown, 0);
    assert!(reverted.date_flown.is_none());
}

#[test]
fn test_mark_all_aircraft_as_not_flown() {
    let mut db = test_helpers::setup_database();

    // Set aircraft 1 to flown
    DataOperations::toggle_aircraft_flown_status(&mut db, 1).unwrap();
    assert_eq!(db.get_aircraft_by_id(1).unwrap().flown, 1);

    // Mark all as not flown
    DataOperations::mark_all_aircraft_as_not_flown(&mut db).unwrap();

    // Verify
    assert_eq!(db.get_aircraft_by_id(1).unwrap().flown, 0);
}

#[test]
#[cfg(feature = "gui")]
fn test_load_history_data() {
    let mut db = test_helpers::setup_database();

    // Setup data: Aircraft 1, Airports (created in setup_database or need to add?)
    // setup_database creates basic airports (ID 1, 2) and aircraft (1, 2) usually.
    // Let's verify by fetching.
    // Need to wrap items in Arc for load_history_data
    let aircraft: Vec<Arc<Aircraft>> = db
        .get_all_aircraft()
        .unwrap()
        .into_iter()
        .map(Arc::new)
        .collect();
    let airports: Vec<Arc<Airport>> = db
        .get_airports()
        .unwrap()
        .into_iter()
        .map(Arc::new)
        .collect();

    DataOperations::add_history_entry(&mut db, &aircraft[0], &airports[0], &airports[1]).unwrap();

    // Call load_history_data
    let items = DataOperations::load_history_data(&mut db, &aircraft, &airports).unwrap();

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].departure_icao, airports[0].ICAO);
    assert_eq!(items[0].arrival_icao, airports[1].ICAO);
    assert_eq!(
        items[0].date,
        flight_planner::date_utils::get_current_date_utc()
    );
    // Verify formatted strings contain expected data
    assert!(items[0].aircraft_name.contains(&aircraft[0].manufacturer));
}
