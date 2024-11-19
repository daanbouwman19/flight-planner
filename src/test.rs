use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

use super::*;
use crate::models::*;

pub const MIGRATION: EmbeddedMigrations = embed_migrations!("test_migration");

fn initialize_aircraft_db() -> SqliteConnection {
    let mut connection = establish_database_connection(":memory:");
    connection
        .run_pending_migrations(MIGRATION)
        .expect("Failed to run migrations");

    connection
}

fn setup_airport(id: i32, name: &str, icao: &str, latitude: f64, longtitude: f64) -> Airport {
    Airport {
        ID: id,
        Name: name.to_string(),
        ICAO: icao.to_string(),
        PrimaryID: None,
        Latitude: latitude,
        Longtitude: longtitude,
        Elevation: 0,
        TransitionAltitude: None,
        TransitionLevel: None,
        SpeedLimit: None,
        SpeedLimitAltitude: None,
    }
}

fn setup_aircraft(id: i32, flown: i32, date_flown: Option<String>) -> Aircraft {
    Aircraft {
        id,
        manufacturer: "Test Manufacturer".to_string(),
        variant: "Test Variant".to_string(),
        icao_code: "TEST".to_string(),
        flown,
        aircraft_range: 100,
        category: "Test Category".to_string(),
        cruise_speed: 0,
        date_flown: date_flown,
    }
}

fn setup_runway(id: i32, airport_id: i32, ident: &str) -> Runway {
    Runway {
        ID: id,
        AirportID: airport_id,
        Ident: ident.to_string(),
        TrueHeading: 90.0,
        Length: 3000,
        Width: 45,
        Surface: "Asphalt".to_string(),
        Latitude: 0.0,
        Longtitude: 0.0,
        Elevation: 0,
    }
}

#[test]
fn test_insert_airport() {
    let airport = setup_airport(1, "Test Airport", "TST", 0.0, 0.0);
    let connection = &mut initialize_aircraft_db();

    insert_airport(connection, &airport).unwrap();
    let result = get_random_airport(connection).unwrap();

    assert_eq!(result.ID, airport.ID);
    assert_eq!(result.Name, airport.Name);
    assert_eq!(result.Latitude, airport.Latitude, "Latitude does not match");
    assert_eq!(
        result.Longtitude, airport.Longtitude,
        "Longitude does not match"
    );
}

#[test]
fn test_haversine_distance_nm() {
    let airport1 = setup_airport(1, "Test Airport 1", "TST1", 0.0, 0.0);
    let airport2 = setup_airport(2, "Test Airport 2", "TST2", 0.0, 1.0);
    let distance = haversine_distance_nm(&airport1, &airport2);
    assert_eq!(distance, 60);

    let distance_same = haversine_distance_nm(&airport1, &airport1);
    assert_eq!(
        distance_same, 0,
        "Distance between the same airport should be zero"
    );
}

#[test]
fn test_insert_aircraft() {
    let aircraft: Aircraft = setup_aircraft(1, 0, None);
    let connection = &mut initialize_aircraft_db();

    insert_aircraft(connection, &aircraft).unwrap();
    let result = get_all_aircraft(connection).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], aircraft);
}

#[test]
fn test_get_unflown_aircraft_count() {
    let connection = &mut initialize_aircraft_db();

    let mut unflown_aircraft = setup_aircraft(1, 0, None);
    insert_aircraft(connection, &unflown_aircraft).unwrap();

    let count = get_unflown_aircraft_count(connection).unwrap();
    assert_eq!(count, 1);

    unflown_aircraft.flown = 1;
    update_aircraft(connection, &unflown_aircraft).unwrap();

    let count_after_update = get_unflown_aircraft_count(connection).unwrap();
    assert_eq!(count_after_update, 0);
}

#[test]
fn test_insert_runway() {
    let runway = setup_runway(1, 1, "09");

    let airport = setup_airport(1, "Test Airport", "TST", 0.0, 0.0);
    let connection = &mut initialize_aircraft_db();

    insert_airport(connection, &airport).unwrap();
    insert_runway(connection, &runway).unwrap();

    let result = get_runways_for_airport(connection, &airport).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], runway);
}

#[test]
fn test_get_runways_for_airport() {
    let airport = setup_airport(1, "Test Airport", "TST", 0.0, 0.0);
    let connection = &mut initialize_aircraft_db();
    insert_airport(connection, &airport).unwrap();

    let runway1 = setup_runway(1, airport.ID, "09");
    let runway2 = setup_runway(2, airport.ID, "27");

    insert_runway(connection, &runway1).unwrap();
    insert_runway(connection, &runway2).unwrap();

    let result = get_runways_for_airport(connection, &airport).unwrap();

    assert_eq!(result.len(), 2);
    assert!(result.contains(&runway1));
    assert!(result.contains(&runway2));
}

#[test]
fn test_get_random_airport() {
    let airport1 = setup_airport(1, "Test Airport 1", "TST1", 0.0, 0.0);
    let airport2 = setup_airport(2, "Test Airport 2", "TST2", 0.0, 1.0);

    let runway1 = setup_runway(1, 1, "09");
    let runway2 = setup_runway(2, 2, "27");

    let connection = &mut initialize_aircraft_db();

    insert_airport(connection, &airport1).unwrap();
    insert_airport(connection, &airport2).unwrap();
    insert_runway(connection, &runway1).unwrap();
    insert_runway(connection, &runway2).unwrap();

    let result = get_random_airport(connection).unwrap();
    assert!(result == airport1 || result == airport2);
}

#[test]
fn test_get_destination_airport() {
    let connection = &mut initialize_aircraft_db();
    let aircraft = setup_aircraft(1, 0, None);

    let airport_departure = setup_airport(1, "Departure Airport", "DEP", 0.0, 0.0);
    let airport_within_range = setup_airport(2, "Within Range Airport", "WR1", 0.0, 1.0);
    let airport_outside_range = setup_airport(3, "Outside Range Airport", "OR1", 0.0, 5.0);
    let another_airport_outside_range = setup_airport(4, "Outside Range Airport", "OR2", 0.0, 10.0);

    insert_airport(connection, &airport_departure).unwrap();
    insert_airport(connection, &airport_within_range).unwrap();
    insert_airport(connection, &airport_outside_range).unwrap();
    insert_airport(connection, &another_airport_outside_range).unwrap();

    // Loop 10 times to test consistency
    for _ in 0..10 {
        let result = get_destination_airport(connection, &aircraft, &airport_departure).unwrap();
        assert_eq!(result, airport_within_range);
    }
}

#[test]
fn test_mark_all_aircraft_unflown() {
    let connection = &mut initialize_aircraft_db();
    let aircraft = setup_aircraft(1, 1, Some("2021-01-01".to_string()));

    let mut count = get_unflown_aircraft_count(connection).unwrap();
    assert_eq!(count, 0);

    insert_aircraft(connection, &aircraft).unwrap();
    mark_all_aircraft_unflown(connection).unwrap();

    count = get_unflown_aircraft_count(connection).unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_all_aircraft() {
    let connection = &mut initialize_aircraft_db();

    let aircraft1 = setup_aircraft(1, 0, None);
    let aircraft2 = setup_aircraft(2, 0, None);

    insert_aircraft(connection, &aircraft1).unwrap();
    insert_aircraft(connection, &aircraft2).unwrap();

    let result = get_all_aircraft(connection).unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result.get(0).unwrap(), &aircraft1);
    assert_eq!(result.get(1).unwrap(), &aircraft2);
}

#[test]
fn test_add_to_history() {
    let connection = &mut initialize_aircraft_db();
    let aircraft = setup_aircraft(1, 1, Some("2021-01-01".to_string()));

    let departure = setup_airport(1, "Departure Airport", "DEP", 0.0, 0.0);
    let arrival = setup_airport(2, "Arrival Airport", "ARR", 0.0, 1.0);

    insert_aircraft(connection, &aircraft).unwrap();
    add_to_history(connection, &departure, &arrival, &aircraft).unwrap();

    let result = get_history(connection).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].arrival_icao, arrival.ICAO);
    assert_eq!(result[0].departure_icao, departure.ICAO);
    assert_eq!(result[0].aircraft, aircraft.id);
}

#[test]
fn test_random_unflown_aircraft() {
    let connection = &mut initialize_aircraft_db();

    let aircraft1 = setup_aircraft(1, 0, None);
    let aircraft2 = setup_aircraft(2, 1, Some("2021-01-01".to_string()));

    insert_aircraft(connection, &aircraft1).unwrap();
    insert_aircraft(connection, &aircraft2).unwrap();

    let result = random_unflown_aircraft(connection).unwrap();
    assert_eq!(result, aircraft1);
}

#[test]
fn test_get_destination_airport_no_options() {
    let connection = &mut initialize_aircraft_db();
    let aircraft = setup_aircraft(1, 0, None);
    let airport_departure = setup_airport(1, "Departure Airport", "DEP", 0.0, 0.0);
    insert_airport(connection, &airport_departure).unwrap();

    let result = get_destination_airport(connection, &aircraft, &airport_departure);
    assert!(
        result.is_err(),
        "Expected error when no destination airports are available"
    );
}

#[test]
fn test_random_unflown_aircraft_empty_database() {
    let mut connection = initialize_aircraft_db();
    let result = random_unflown_aircraft(&mut connection);

    assert!(result.is_err());
}

#[test]
fn test_format_functions() {
    let aircraft = setup_aircraft(1, 0, None);
    let aircraft_string = format_aircraft(&aircraft);
    assert_eq!(
        aircraft_string,
        "Test Manufacturer Test Variant (TEST), range: 100 nm, category: Test Category, cruise speed: 0 knots"
    );

    let airport = setup_airport(1, "Test Airport", "TST", 0.0, 0.0);
    let airport_string = format_airport(&airport);
    assert_eq!(airport_string, "Test Airport (TST), altitude: 0");

    let runway = setup_runway(1, 1, "09");
    let runway_string = format_runway(&runway);
    assert_eq!(
        runway_string,
        "Runway: 09, heading: 90.00, length: 3000, width: 45, surface: Asphalt, elevation: 0ft"
    );
}
