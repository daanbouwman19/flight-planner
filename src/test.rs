use super::*;

use sqlite;

fn create_airport_database() -> AirportDatabase {
    let connection = sqlite::open(":memory:").unwrap();
    initialize_airport_db(&connection);
    AirportDatabase { connection }
}

fn create_aircraft_database() -> AircraftDatabase {
    let connection = sqlite::open(":memory:").unwrap();
    initialize_aircraft_db(&connection);
    AircraftDatabase { connection }
}

// Common setup functions
fn setup_airport(id: i64, name: &str, icao_code: &str, latitude: f64, longitude: f64) -> Airport {
    Airport {
        id,
        name: name.to_string(),
        icao_code: icao_code.to_string(),
        latitude,
        longtitude: longitude,
        elevation: 0,
        runways: Vec::new(),
    }
}

fn setup_aircraft(id: i64, flown: bool, date_flown: Option<String>) -> Aircraft {
    Aircraft {
        id,
        manufacturer: "Test Manufacturer".to_string(),
        icao_code: "TEST".to_string(),
        variant: "Test Variant".to_string(),
        flown,
        aircraft_range: 100,
        category: "Test Category".to_string(),
        cruise_speed: 0,
        date_flown,
    }
}

#[test]
fn test_insert_airport() {
    let airport = setup_airport(1, "Test Airport", "TST", 0.0, 0.0);
    let airport_database = create_airport_database();

    airport_database.insert_airport(&airport).unwrap();
    let result = airport_database.get_random_airport().unwrap();

    assert_eq!(result.id, airport.id);
    assert_eq!(result.name, airport.name);
    assert_eq!(result.latitude, airport.latitude, "Latitude does not match");
    assert_eq!(
        result.longtitude, airport.longtitude,
        "Longitude does not match"
    );
}

#[test]
fn test_haversine_distance_nm() {
    let airport1 = setup_airport(1, "Test Airport 1", "TST1", 0.0, 0.0);
    let airport2 = setup_airport(2, "Test Airport 2", "TST2", 0.0, 1.0);
    let airport_database = create_airport_database();

    let distance = airport_database.haversine_distance_nm(&airport1, &airport2);
    assert_eq!(distance, 60);

    let distance_same = airport_database.haversine_distance_nm(&airport1, &airport1);
    assert_eq!(
        distance_same, 0,
        "Distance between the same airport should be zero"
    );
}

#[test]
fn test_insert_aircraft() {
    let aircraft = setup_aircraft(1, false, None);
    let aircraft_database = create_aircraft_database();

    aircraft_database.insert_aircraft(&aircraft).unwrap();
    let result = aircraft_database.get_all_aircraft().unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], aircraft);
}

#[test]
fn test_get_unflown_aircraft_count() {
    let aircraft_database = create_aircraft_database();

    let mut unflown_aircraft = setup_aircraft(1, false, None);
    aircraft_database
        .insert_aircraft(&unflown_aircraft)
        .unwrap();

    let count = aircraft_database.get_unflown_aircraft_count().unwrap();
    assert_eq!(count, 1);

    // Test after marking aircraft as flown
    unflown_aircraft.flown = true;
    aircraft_database
        .update_aircraft(&unflown_aircraft)
        .unwrap();
    let count_after_update = aircraft_database.get_unflown_aircraft_count().unwrap();
    assert_eq!(count_after_update, 0);
}

#[test]
fn test_insert_runway() {
    let runway = Runway {
        id: 1,
        airport_id: 1,
        ident: "09".to_string(),
        true_heading: 90.0,
        length: 3000,
        width: 45,
        surface: "Asphalt".to_string(),
        latitude: 0.0,
        longtitude: 0.0,
        elevation: 0,
    };

    let airport = setup_airport(1, "Test Airport", "TST", 0.0, 0.0);
    let airport_database = create_airport_database();

    airport_database.insert_airport(&airport).unwrap();
    airport_database.insert_runway(&runway).unwrap();

    let result = airport_database
        .get_runways_for_airport(airport.id)
        .unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], runway);
}

#[test]
fn test_get_runways_for_airport() {
    let airport = setup_airport(1, "Test Airport", "TST", 0.0, 0.0);
    let airport_database = create_airport_database();

    airport_database.insert_airport(&airport).unwrap();

    let runway1 = Runway {
        id: 1,
        airport_id: airport.id,
        ident: "09".to_string(),
        true_heading: 90.0,
        length: 3000,
        width: 45,
        surface: "Asphalt".to_string(),
        latitude: 0.0,
        longtitude: 0.0,
        elevation: 0,
    };
    let runway2 = Runway {
        id: 2,
        airport_id: airport.id,
        ident: "27".to_string(),
        true_heading: 270.0,
        length: 3000,
        width: 45,
        surface: "Asphalt".to_string(),
        latitude: 0.0,
        longtitude: 0.0,
        elevation: 0,
    };

    airport_database.insert_runway(&runway1).unwrap();
    airport_database.insert_runway(&runway2).unwrap();

    let result = airport_database
        .get_runways_for_airport(airport.id)
        .unwrap();
    assert_eq!(result.len(), 2);
    assert!(result.contains(&runway1));
    assert!(result.contains(&runway2));
}

#[test]
fn test_get_random_airport() {
    let airport1 = setup_airport(1, "Test Airport 1", "TST1", 0.0, 0.0);
    let airport2 = setup_airport(2, "Test Airport 2", "TST2", 0.0, 1.0);

    let runway1 = Runway {
        id: 1,
        airport_id: airport1.id,
        ident: "09".to_string(),
        true_heading: 90.0,
        length: 3000,
        width: 45,
        surface: "Asphalt".to_string(),
        latitude: 0.0,
        longtitude: 0.0,
        elevation: 0,
    };

    let runway2 = Runway {
        id: 2,
        airport_id: airport2.id,
        ident: "09".to_string(),
        true_heading: 90.0,
        length: 3000,
        width: 45,
        surface: "Asphalt".to_string(),
        latitude: 0.0,
        longtitude: 0.0,
        elevation: 0,
    };

    let airport_database = create_airport_database();

    airport_database.insert_airport(&airport1).unwrap();
    airport_database.insert_airport(&airport2).unwrap();
    airport_database.insert_runway(&runway1).unwrap();
    airport_database.insert_runway(&runway2).unwrap();

    let result = airport_database.get_random_airport().unwrap();
    assert!(result == airport1 || result == airport2);
}

#[test]
fn test_get_destination_airport() {
    let airport_database = create_airport_database();
    let aircraft = setup_aircraft(1, false, None);

    // Assign unique ICAO codes to each airport
    let airport_departure = setup_airport(1, "Departure Airport", "DEP", 0.0, 0.0);
    let airport_within_range = setup_airport(2, "Within Range Airport", "WR1", 0.0, 1.0);
    let airport_outside_range = setup_airport(3, "Outside Range Airport", "OR1", 0.0, 5.0);

    airport_database.insert_airport(&airport_departure).unwrap();
    airport_database
        .insert_airport(&airport_within_range)
        .unwrap();
    airport_database
        .insert_airport(&airport_outside_range)
        .unwrap();

    // Loop 5 times to test consistency
    for _ in 0..5 {
        let result = airport_database
            .get_destination_airport(&aircraft, &airport_departure)
            .unwrap();
        assert_eq!(result, airport_within_range);
    }
}

#[test]
fn test_mark_all_aircraft_unflown() {
    let aircraft_database = create_aircraft_database();
    let aircraft = setup_aircraft(1, true, Some("2021-01-01".to_string()));

    aircraft_database.insert_aircraft(&aircraft).unwrap();
    aircraft_database.mark_all_aircraft_unflown().unwrap();

    let count = aircraft_database.get_unflown_aircraft_count().unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_all_aircraft() {
    let aircraft_database = create_aircraft_database();

    let aircraft1 = setup_aircraft(1, false, None);
    let aircraft2 = setup_aircraft(2, false, None);

    aircraft_database.insert_aircraft(&aircraft1).unwrap();
    aircraft_database.insert_aircraft(&aircraft2).unwrap();

    let result = aircraft_database.get_all_aircraft().unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result.get(0).unwrap(), &aircraft1);
    assert_eq!(result.get(1).unwrap(), &aircraft2);
}

#[test]
fn test_add_to_history() {
    let aircraft_database = create_aircraft_database();
    let aircraft = setup_aircraft(1, true, Some("2021-01-01".to_string()));

    let departure = setup_airport(1, "Departure Airport", "DEP", 0.0, 0.0);
    let arrival = setup_airport(2, "Arrival Airport", "ARR", 0.0, 1.0);

    aircraft_database.insert_aircraft(&aircraft).unwrap();

    aircraft_database
        .add_to_history(&departure, &arrival, &aircraft)
        .unwrap();

    let result = aircraft_database.get_history().unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].arrival_icao, arrival.icao_code);
    assert_eq!(result[0].departure_icao, departure.icao_code);
    assert_eq!(result[0].aircraft_id, aircraft.id);
    // Add assertion for date if needed
}

#[test]
fn test_random_unflown_aircraft() {
    let aircraft_database = create_aircraft_database();

    let aircraft1 = setup_aircraft(1, false, None);
    let aircraft2 = setup_aircraft(2, true, Some("2021-01-01".to_string()));

    aircraft_database.insert_aircraft(&aircraft1).unwrap();
    aircraft_database.insert_aircraft(&aircraft2).unwrap();

    let result = aircraft_database.random_unflown_aircraft().unwrap();
    assert_eq!(result, aircraft1);
}

#[test]
fn test_show_functions() {
    let aircraft_database = create_aircraft_database();
    let airport_database = create_airport_database();

    show_all_aircraft(&aircraft_database);
    show_history(&aircraft_database);
    show_random_aircraft_and_route(&aircraft_database, &airport_database);
    show_random_aircraft_with_random_airport(&aircraft_database, &airport_database);
    show_random_airport(&airport_database);
    show_random_unflown_aircraft(&aircraft_database);

    let aircraft = Aircraft {
        id: 1,
        manufacturer: "Test Manufacturer".to_string(),
        icao_code: "TEST".to_string(),
        variant: "Test Variant".to_string(),
        flown: false,
        aircraft_range: 0,
        category: "Test Category".to_string(),
        cruise_speed: 0,
        date_flown: None,
    };

    let airport = Airport {
        id: 1,
        name: "Test Airport".to_string(),
        icao_code: "tst1".to_string(),
        latitude: 0.0,
        longtitude: 0.0,
        elevation: 0,
        runways: Vec::new(),
    };

    let runway = Runway {
        id: 1,
        airport_id: 1,
        ident: "09".to_string(),
        true_heading: 90.0,
        length: 3000,
        width: 45,
        surface: "Asphalt".to_string(),
        latitude: 0.0,
        longtitude: 0.0,
        elevation: 0,
    };

    aircraft_database.insert_aircraft(&aircraft).unwrap();
    airport_database.insert_airport(&airport).unwrap();
    airport_database.insert_runway(&runway).unwrap();

    show_random_aircraft_and_route(&aircraft_database, &airport_database);
    show_random_aircraft_with_random_airport(&aircraft_database, &airport_database);
    show_random_airport(&airport_database);
}

#[test]
fn test_fmt_debug() {
    let aircraft = Aircraft {
        id: 1,
        manufacturer: "Test Manufacturer".to_string(),
        icao_code: "TEST".to_string(),
        variant: "Test Variant".to_string(),
        flown: false,
        aircraft_range: 0,
        category: "Test Category".to_string(),
        cruise_speed: 0,
        date_flown: None,
    };
    println!("{:?}", aircraft);

    let airport = Airport {
        id: 1,
        name: "Test Airport".to_string(),
        icao_code: "tst1".to_string(),
        latitude: 0.0,
        longtitude: 0.0,
        elevation: 0,
        runways: Vec::new(),
    };
    println!("{:?}", airport);

    let runway = Runway {
        id: 1,
        airport_id: 1,
        ident: "09".to_string(),
        true_heading: 90.0,
        length: 3000,
        width: 45,
        surface: "Asphalt".to_string(),
        latitude: 0.0,
        longtitude: 0.0,
        elevation: 0,
    };
    println!("{:?}", runway);

    let history = History {
        id: 1,
        aircraft_id: 1,
        departure_icao: "tst1".to_string(),
        arrival_icao: "tst2".to_string(),
        date: "2021-01-01".to_string(),
    };
    println!("{:?}", history);
}

#[test]
fn test_get_destination_airport_no_options() {
    let airport_database = create_airport_database();
    let aircraft = setup_aircraft(1, false, None);
    let airport_departure = setup_airport(1, "Departure Airport", "DEP", 0.0, 0.0);
    airport_database.insert_airport(&airport_departure).unwrap();
    // No other airports added
    let result = airport_database.get_destination_airport(&aircraft, &airport_departure);
    assert!(
        result.is_err(),
        "Expected error when no destination airports are available"
    );
}

#[test]
fn test_random_unflown_aircraft_empty_database() {
    let aircraft_database = create_aircraft_database();
    let result = aircraft_database.random_unflown_aircraft();
    assert!(
        result.is_err(),
        "Expected error when no unflown aircraft are available"
    );
}
