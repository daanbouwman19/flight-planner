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

#[test]
fn test_insert_airport() {
    let airport = Airport {
        id: 1,
        name: "Test Airport".to_string(),
        icao_code: "".to_string(),
        latitude: 0.0,
        longtitude: 0.0,
        elevation: 0,
    };

    let airport_database = create_airport_database();
    airport_database.insert_airport(&airport).unwrap();
    let result = airport_database.get_random_airport().unwrap();
    assert_eq!(result, airport);
}

#[test]
fn test_haversine_distance_nm() {
    let airport1 = Airport {
        id: 1,
        name: "Test Airport 1".to_string(),
        icao_code: "".to_string(),
        latitude: 0.0,
        longtitude: 0.0,
        elevation: 0,
    };

    let airport2 = Airport {
        id: 2,
        name: "Test Airport 2".to_string(),
        icao_code: "".to_string(),
        latitude: 0.0,
        longtitude: 1.0,
        elevation: 0,
    };

    let airport_database = create_airport_database();
    let distance = airport_database.haversine_distance_nm(&airport1, &airport2);
    assert_eq!(distance, 60);
}

#[test]
fn test_insert_aircarft() {
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

    let aircraft_database = create_aircraft_database();
    aircraft_database.insert_aircraft(&aircraft).unwrap();
    let result = aircraft_database.get_all_aircraft().unwrap();
    assert_eq!(result.len(), 1);

    let result = result.get(0).unwrap();
    assert_eq!(result, &aircraft);
}

#[test]
fn test_get_unflown_aircraft_count() {
    let aircraft_database = create_aircraft_database();

    let count = aircraft_database.get_unflown_aircraft_count().unwrap();
    assert_eq!(count, 0);

    let unflown_aircraft = Aircraft {
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

    aircraft_database
        .insert_aircraft(&unflown_aircraft)
        .unwrap();

    let count = aircraft_database.get_unflown_aircraft_count().unwrap();
    assert_eq!(count, 1);

    let flown_aircraft = Aircraft {
        id: 2,
        manufacturer: "Test Manufacturer".to_string(),
        icao_code: "TEST".to_string(),
        variant: "Test Variant".to_string(),
        flown: true,
        aircraft_range: 0,
        category: "Test Category".to_string(),
        cruise_speed: 0,
        date_flown: None,
    };

    aircraft_database.insert_aircraft(&flown_aircraft).unwrap();

    let count = aircraft_database.get_unflown_aircraft_count().unwrap();
    assert_eq!(count, 1);
}
