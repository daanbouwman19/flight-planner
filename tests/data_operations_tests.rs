use flight_planner::models::{Aircraft, Airport, History};
use flight_planner::modules::data_operations::DataOperations;
use flight_planner::test_helpers;
use flight_planner::traits::AircraftOperations;
use std::sync::Arc;

#[test]
fn test_calculate_statistics_from_history() {
    let aircraft = Arc::new(Aircraft {
        id: 1,
        manufacturer: "Boeing".to_string(),
        variant: "737".to_string(),
        icao_code: "B737".to_string(),
        flown: 0,
        aircraft_range: 2000,
        category: "A".to_string(),
        cruise_speed: 450,
        date_flown: None,
        takeoff_distance: None,
    });

    let aircraft_list = vec![aircraft];

    let history = vec![
        History {
            id: 1,
            aircraft: 1,
            departure_icao: "JFK".to_string(),
            arrival_icao: "LHR".to_string(),
            date: "2023-01-01".to_string(),
            distance: Some(3000),
        },
        History {
            id: 2,
            aircraft: 1,
            departure_icao: "LHR".to_string(),
            arrival_icao: "CDG".to_string(),
            date: "2023-01-02".to_string(),
            distance: Some(200),
        },
    ];

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
        airports.push(Arc::new(Airport {
            ID: i,
            Name: format!("Airport {}", i),
            ICAO: format!("ICAO{}", i),
            PrimaryID: None,
            Latitude: 0.0,
            Longtitude: 0.0,
            Elevation: 0,
            TransitionAltitude: None,
            TransitionLevel: None,
            SpeedLimit: None,
            SpeedLimitAltitude: None,
        }));
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
