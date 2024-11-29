use diesel::Connection;
use diesel::SqliteConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use crate::haversine_distance_nm;
use crate::AircraftOperations;
use crate::AirportOperations;
use crate::DatabaseConnections;
use crate::HistoryOperations;

use crate::models::*;

const AIRCRAFT_MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations_aircraft_database");
const AIRPORT_MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations_airport_database");
const M_TO_FEET: f64 = 3.28084;

fn initialize_aircraft_db() -> DatabaseConnections {
    fn establish_database_connection(database_name: &str) -> SqliteConnection {
        SqliteConnection::establish(database_name).unwrap_or_else(|_| {
            panic!("Error connecting to {}", database_name);
        })
    }

    let mut aircraft_connection = establish_database_connection(":memory:");
    let mut airport_connection = establish_database_connection(":memory:");

    airport_connection.run_pending_migrations(AIRPORT_MIGRATIONS).unwrap();
    aircraft_connection.run_pending_migrations(AIRCRAFT_MIGRATIONS).unwrap();
  
    DatabaseConnections {
        aircraft_connection,
        airport_connection,
    }
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

fn setup_aircraft(
    id: i32,
    flown: i32,
    date_flown: Option<String>,
    takeoff_distance: Option<i32>,
) -> Aircraft {
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
        takeoff_distance: takeoff_distance,
    }
}

fn setup_runway(id: i32, airport_id: i32, ident: &str, length: i32) -> Runway {
    Runway {
        ID: id,
        AirportID: airport_id,
        Ident: ident.to_string(),
        TrueHeading: 90.0,
        Length: length,
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
    let mut connection = initialize_aircraft_db();

    connection.insert_airport( &airport).unwrap();
    let result = connection.get_random_airport().unwrap();

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
    let aircraft: Aircraft = setup_aircraft(1, 0, None, None);
    let connection = &mut initialize_aircraft_db();

    connection.insert_aircraft(&aircraft).unwrap();
    let result = connection.get_all_aircraft().unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], aircraft);
}

#[test]
fn test_get_unflown_aircraft_count() {
    let connection = &mut initialize_aircraft_db();

    let mut unflown_aircraft = setup_aircraft(1, 0, None, None);
    connection.insert_aircraft(&unflown_aircraft).unwrap();

    let count = connection.get_unflown_aircraft_count().unwrap();
    assert_eq!(count, 1);

    unflown_aircraft.flown = 1;
    connection.update_aircraft(&unflown_aircraft).unwrap();

    let count_after_update = connection.get_unflown_aircraft_count().unwrap();
    assert_eq!(count_after_update, 0);
}

#[test]
fn test_insert_runway() {
    let runway = setup_runway(1, 1, "09", 3000);

    let airport = setup_airport(1, "Test Airport", "TST", 0.0, 0.0);
    let connection = &mut initialize_aircraft_db();

    connection.insert_airport(&airport).unwrap();
    connection.insert_runway(&runway).unwrap();

    let result = connection.get_runways_for_airport(&airport).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], runway);
}

#[test]
fn test_get_runways_for_airport() {
    let airport = setup_airport(1, "Test Airport", "TST", 0.0, 0.0);
    let connection = &mut initialize_aircraft_db();
    
    connection.insert_airport(&airport).unwrap();

    let runway1 = setup_runway(1, airport.ID, "09", 3000);
    let runway2 = setup_runway(2, airport.ID, "27", 3000);

    connection.insert_runway(&runway1).unwrap();
    connection.insert_runway(&runway2).unwrap();

    let result = connection.get_runways_for_airport(&airport).unwrap();

    assert_eq!(result.len(), 2);
    assert!(result.contains(&runway1));
    assert!(result.contains(&runway2));
}

#[test]
fn test_get_random_airport() {
    let connection = &mut initialize_aircraft_db();

    // Test with empty database
    let result = connection.get_random_airport();
    assert!(result.is_err(), "Expected error with empty database");

    // Add a single airport
    let airport1 = setup_airport(1, "Test Airport 1", "TST1", 0.0, 0.0);
    connection.insert_airport(&airport1).unwrap();

    // Test with single airport
    let result = connection.get_random_airport().unwrap();
    assert_eq!(result, airport1);

    // Add another airport
    let airport2 = setup_airport(2, "Test Airport 2", "TST2", 1.0, 1.0);
    connection.insert_airport(&airport2).unwrap();

    // Test with multiple airports
    let result = connection.get_random_airport().unwrap();
    assert!(result == airport1 || result == airport2);
}

#[test]
fn test_get_destination_airport() {
    let connection = &mut initialize_aircraft_db();
    
    // Setup test data
    let aircraft_with_distance = setup_aircraft(1, 0, None, Some(3000));
    let aircraft_without_distance = setup_aircraft(2, 0, None, None);
    
    let departure_airport = setup_airport(1, "Departure Airport", "DEP", 0.0, 0.0);
    let suitable_airport = setup_airport(2, "Suitable Airport", "SAT", 0.5, 0.5);
    let unsuitable_airport = setup_airport(3, "Unsuitable Airport", "USAT", 0.5, 0.5);
    
    // Setup runways with different lengths
    let suitable_runway = setup_runway(1, suitable_airport.ID, "09", (3500.0 * M_TO_FEET) as i32);
    let short_runway = setup_runway(2, unsuitable_airport.ID, "27", (2500.0 * M_TO_FEET) as i32);
    
    // Insert test data into database
    connection.insert_airport(&departure_airport).unwrap();
    connection.insert_airport(&suitable_airport).unwrap();
    connection.insert_airport(&unsuitable_airport).unwrap();
    connection.insert_runway(&suitable_runway).unwrap();
    connection.insert_runway(&short_runway).unwrap();
    
    // Test aircraft with takeoff distance requirement
    let result = connection.get_destination_airport(&aircraft_with_distance, &departure_airport);
    assert!(result.is_ok(), "Failed to get destination airport");
    let destination = result.unwrap();
    assert_eq!(destination, suitable_airport, "Should select airport with suitable runway");
    
    // Test aircraft without takeoff distance requirement
    let result = connection.get_destination_airport(&aircraft_without_distance, &departure_airport);
    assert!(result.is_ok(), "Failed to get destination airport");
    let destination = result.unwrap();
    assert!(
        destination == suitable_airport || destination == unsuitable_airport,
        "Should be able to select any airport when no takeoff distance is specified"
    );
}

#[test]
fn test_mark_all_aircraft_unflown() {
    let connection = &mut initialize_aircraft_db();
    
    // Insert aircraft with flown status
    let aircraft1 = setup_aircraft(1, 1, Some("2021-01-01".to_string()), None);
    let aircraft2 = setup_aircraft(2, 1, Some("2021-01-01".to_string()), None);
    connection.insert_aircraft(&aircraft1).unwrap();
    connection.insert_aircraft(&aircraft2).unwrap();

    // Verify initial flown status
    let count = connection.get_unflown_aircraft_count().unwrap();
    assert_eq!(count, 0, "Expected no unflown aircraft initially");

    // Mark all aircraft as unflown
    connection.mark_all_aircraft_unflown().unwrap();

    // Verify all aircraft are marked as unflown
    let count = connection.get_unflown_aircraft_count().unwrap();
    assert_eq!(count, 2, "Expected all aircraft to be marked as unflown");
}

#[test]
fn test_all_aircraft() {
    let mut connection = initialize_aircraft_db();

    let aircraft1 = setup_aircraft(1, 0, None, None);
    let aircraft2 = setup_aircraft(2, 0, None, None);

    connection.insert_aircraft(&aircraft1).unwrap();
    connection.insert_aircraft(&aircraft2).unwrap();

    let result = connection.get_all_aircraft().unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0], aircraft1);
    assert_eq!(result[1], aircraft2);
}

#[test]
fn test_add_to_history() {
    let mut connection = initialize_aircraft_db();
    let aircraft = setup_aircraft(1, 1, Some("2021-01-01".to_string()), None);

    let departure = setup_airport(1, "Departure Airport", "DEP", 0.0, 0.0);
    let arrival = setup_airport(2, "Arrival Airport", "ARR", 0.0, 1.0);

    connection.insert_aircraft(&aircraft).unwrap();
    connection.add_to_history(&departure, &arrival, &aircraft).unwrap();

    let result = connection.get_history().unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].arrival_icao, arrival.ICAO);
    assert_eq!(result[0].departure_icao, departure.ICAO);
    assert_eq!(result[0].aircraft, aircraft.id);

    let result = connection.add_to_history(&departure, &arrival, &aircraft);
    assert!(result.is_ok(), "Expected success when adding to history");
}

#[test]
fn test_random_unflown_aircraft() {
    let mut connection = initialize_aircraft_db();

    let aircraft1 = setup_aircraft(1, 0, None, None);
    let aircraft2 = setup_aircraft(2, 1, Some("2021-01-01".to_string()), None);

    connection.insert_aircraft(&aircraft1).unwrap();
    connection.insert_aircraft(&aircraft2).unwrap();

    let result = connection.random_unflown_aircraft().unwrap();
    assert_eq!(result, aircraft1);
}

#[test]
fn test_get_destination_airport_no_options() {
    let mut connection = initialize_aircraft_db();
    let aircraft = setup_aircraft(1, 0, None, None);
    let airport_departure = setup_airport(1, "Departure Airport", "DEP", 0.0, 0.0);
    connection.insert_airport(&airport_departure).unwrap();

    let result = connection.get_destination_airport(&aircraft, &airport_departure);
    assert!(
        result.is_err(),
        "Expected error when no destination airports are available"
    );
}

#[test]
fn test_random_unflown_aircraft_empty_database() {
    let mut connection = initialize_aircraft_db();
    let result = connection.random_unflown_aircraft();

    assert!(result.is_err());
}

#[test]
fn test_get_random_airport_for_aircraft() {
    let mut connection = initialize_aircraft_db();

    let aircraft_with_distance = setup_aircraft(1, 0, None, Some(3000));
    let aircraft_without_distance = setup_aircraft(2, 0, None, None);
    let departure_airport = setup_airport(1, "Departure Airport", "DEP", 0.0, 0.0);
    let too_short_runway = setup_runway(1, 1, "09", (2000.0 * M_TO_FEET) as i32);
    let long_enough_runway = setup_runway(2, 1, "27", (3000.0 * M_TO_FEET) as i32);

    connection.insert_airport(&departure_airport).unwrap();
    connection.insert_runway(&too_short_runway).unwrap();

    let result = connection.get_random_airport_for_aircraft(&aircraft_with_distance);
    assert!(
        result.is_err(),
        "Expected error when no suitable airports are available"
    );

    let result = connection.get_random_airport_for_aircraft(&aircraft_without_distance);
    assert!(
        result.is_ok(),
        "Expected success when no runway length is required"
    );

    connection.insert_runway(&long_enough_runway).unwrap();
    let result = connection.get_random_airport_for_aircraft(&aircraft_with_distance).unwrap();
    assert_eq!(result, departure_airport);
}

#[test]
fn test_get_destination_airport_with_suitable_runway_haversine() {
    let mut connection = initialize_aircraft_db();

    let aircraft = setup_aircraft(1, 0, None, Some(3000));
    let departure_airport = setup_airport(1, "Departure Airport", "DEP", 0.0, 0.0);

    let airport_within_bounding_box =
        setup_airport(4, "Airport within bounding box", "AWBB", 1.66, 1.66);
    let runway_within_bounding_box = setup_runway(
        3,
        airport_within_bounding_box.ID,
        "09",
        (3000.0 * M_TO_FEET) as i32,
    );

    connection.insert_airport(&departure_airport).unwrap();
    connection.insert_airport(&airport_within_bounding_box).unwrap();
    connection.insert_runway(&runway_within_bounding_box).unwrap();

    let result = connection.get_destination_airport_with_suitable_runway(
        &departure_airport,
        aircraft.aircraft_range,
        aircraft.takeoff_distance.unwrap(),
    );

    let aircraft_range = aircraft.aircraft_range;
    let distance = haversine_distance_nm(&departure_airport, &airport_within_bounding_box);

    assert!(
        distance > aircraft_range,
        "Expected distance to be greater than aircraft range",
    );

    assert!(
        result.is_err(),
        "Expected error when no suitable airports are within range"
    );
}

#[test]
fn test_get_destination_airport_with_suitable_runway() {
    let mut connection = initialize_aircraft_db();

    let aircraft = setup_aircraft(1, 0, None, Some(3000));

    let departure_airport = setup_airport(1, "Departure Airport", "DEP", 0.0, 0.0);
    let suitable_airport = setup_airport(2, "Suitable Airport", "SAT", 0.5, 0.5);
    let unsuitable_airport = setup_airport(3, "Unsuitable Airport", "USAT", 0.5, 0.5);

    let suitable_runway = setup_runway(1, suitable_airport.ID, "09", (3500.0 * M_TO_FEET) as i32);
    let short_runway = setup_runway(2, unsuitable_airport.ID, "27", (2500.0 * M_TO_FEET) as i32);

    let result = connection.get_destination_airport_with_suitable_runway(
        &departure_airport,
        aircraft.aircraft_range,
        aircraft.takeoff_distance.unwrap(),
    );

    assert!(
        result.is_err(),
        "Expected error when no suitable airports are available"
    );

    connection.insert_airport(&departure_airport).unwrap();
    connection.insert_airport(&suitable_airport).unwrap();
    connection.insert_airport(&unsuitable_airport).unwrap();
    connection.insert_runway(&suitable_runway).unwrap();
    connection.insert_runway(&short_runway).unwrap();

    let result = connection.get_destination_airport_with_suitable_runway(
        &departure_airport,
        aircraft.aircraft_range,
        aircraft.takeoff_distance.unwrap(),
    )
    .unwrap();

    assert_eq!(result, suitable_airport);
}

#[test]
fn test_get_airport_within_distance() {
    let mut connection = initialize_aircraft_db();

    let aircraft = setup_aircraft(1, 0, None, None);

    let departure_airport = setup_airport(1, "Departure Airport", "DEP", 0.0, 0.0);
    let within_range_airport = setup_airport(2, "Within Range Airport", "WRA", 0.5, 0.5);
    let outside_range_airport = setup_airport(3, "Outside Range Airport", "ORA", 10.0, 10.0);

    connection.insert_airport(&departure_airport).unwrap();
    connection.insert_airport(&within_range_airport).unwrap();
    connection.insert_airport(&outside_range_airport).unwrap();

    let result = connection
        .get_airport_within_distance(&departure_airport, aircraft.aircraft_range)
        .unwrap();

    assert_eq!(result, within_range_airport);
}

#[test]
fn test_get_airport_within_distance_haversine() {
    let mut connection = initialize_aircraft_db();

    let aircraft = setup_aircraft(1, 0, None, Some(3000));

    let departure_airport = setup_airport(1, "Departure Airport", "DEP", 0.0, 0.0);
    let outside_range_airport = setup_airport(2, "Outside Range Airport", "ORA", 1.66, 1.66);

    connection.insert_airport(&departure_airport).unwrap();

    let result = connection.get_airport_within_distance(&departure_airport, aircraft.aircraft_range);
    assert!(
        result.is_err(),
        "Expected error when no airports are within range"
    );

    connection.insert_airport(&outside_range_airport).unwrap();

    let result = connection.get_airport_within_distance(&departure_airport, aircraft.aircraft_range);

    assert!(
        result.is_err(),
        "Expected error when no airports are within range"
    );
}

#[test]
fn test_get_aircraft_by_id() {
    let mut connection = initialize_aircraft_db();

    let aircraft = setup_aircraft(1, 0, None, None);
    connection.insert_aircraft(&aircraft).unwrap();

    let retrieved_aircraft = connection.get_aircraft_by_id(aircraft.id).unwrap();

    assert_eq!(retrieved_aircraft, aircraft);

    let result = connection.get_aircraft_by_id(2);
    assert!(result.is_err());

    let result = connection.get_aircraft_by_id(-1);
    assert!(result.is_err());
}

#[test]
fn test_insert_airport_with_invalid_data() {
    let mut connection = initialize_aircraft_db();
    let invalid_airport = setup_airport(1, "", "", 0.0, 0.0);

    let result = connection.insert_airport(&invalid_airport);
    assert!(
        result.is_err(),
        "Expected error when inserting invalid airport data"
    );
}

#[test]
fn test_insert_aircraft_with_invalid_data() {
    let mut connection = initialize_aircraft_db();
    let invalid_aircraft = setup_aircraft(1, -1, None, None);

    let result = connection.insert_aircraft(&invalid_aircraft);
    assert!(
        result.is_err(),
        "Expected error when inserting invalid aircraft data"
    );
}

#[test]
fn test_insert_runway_with_invalid_data() {
    let mut connection = initialize_aircraft_db();
    let invalid_runway = setup_runway(1, 1, "", -100);

    let result = connection.insert_runway(&invalid_runway);
    assert!(
        result.is_err(),
        "Expected error when inserting invalid runway data"
    );
}

#[test]
fn test_get_aircraft_with_invalid_id() {
    let mut connection = initialize_aircraft_db();
    let result = connection.get_aircraft_by_id(-1);

    assert!(
        result.is_err(),
        "Expected error when retrieving aircraft with invalid ID"
    );
}

#[test]
fn test_haversine_distance_with_same_coordinates() {
    let airport1 = setup_airport(1, "Test Airport 1", "TST1", 0.0, 0.0);
    let airport2 = setup_airport(2, "Test Airport 2", "TST2", 0.0, 0.0);
    let distance = haversine_distance_nm(&airport1, &airport2);

    assert_eq!(
        distance, 0,
        "Expected distance to be zero for same coordinates"
    );
}

#[test]
fn test_get_destination_airport_with_no_runways() {
    let mut connection = initialize_aircraft_db();
    let aircraft = setup_aircraft(1, 0, None, Some(3000));
    let airport = setup_airport(1, "Test Airport", "TST", 0.0, 0.0);

    connection.insert_airport(&airport).unwrap();

    let result = connection.get_destination_airport(&aircraft, &airport);
    assert!(
        result.is_err(),
        "Expected error when no runways are available"
    );
}

#[test]
fn test_get_random_airport_with_empty_database() {
    let mut connection = initialize_aircraft_db();
    let result = connection.get_random_airport();

    assert!(
        result.is_err(),
        "Expected error when no airports are available"
    );
}

#[test]
fn test_validate_errors() {
    use crate::ValidationError;

    let invalid_data = ValidationError::InvalidData("Test".to_string());
    let database_error = ValidationError::DatabaseError("Test".to_string());
    let invalid_id = ValidationError::InvalidId(-1);

    assert_eq!(format!("{}", invalid_data), "Invalid data: Test");
    assert_eq!(format!("{}", database_error), "Database error: Test");
    assert_eq!(format!("{}", invalid_id), "Invalid ID: -1");
}

#[test]
fn test_read_id() {
    use crate::read_id;

    let result = read_id(|| Ok("1".to_string()));
    assert!(result.is_ok());

    let result = read_id(|| Ok("a".to_string()));
    assert!(result.is_err());

    let result = read_id(|| Ok("-1".to_string()));
    assert!(result.is_err());

    let result = read_id(|| Ok("0".to_string()));
    assert!(result.is_err());

    let result = read_id(|| Ok("1.0".to_string()));
    assert!(result.is_err());

    let result = read_id(|| Ok("".to_string()));
    assert!(result.is_err());
}
