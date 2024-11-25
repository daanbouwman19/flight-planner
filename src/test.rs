use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use modules::aircraft::tests::*;
use modules::airport::tests::*;
use modules::runway::tests::*;

use super::*;
use crate::models::*;

pub const MIGRATION: EmbeddedMigrations = embed_migrations!("test_migration");
const M_TO_FEET: f64 = 3.28084;

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
    let aircraft: Aircraft = setup_aircraft(1, 0, None, None);
    let connection = &mut initialize_aircraft_db();

    insert_aircraft(connection, &aircraft).unwrap();
    let result = get_all_aircraft(connection).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], aircraft);
}

#[test]
fn test_get_unflown_aircraft_count() {
    let connection = &mut initialize_aircraft_db();

    let mut unflown_aircraft = setup_aircraft(1, 0, None, None);
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
    let runway = setup_runway(1, 1, "09", 3000);

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

    let runway1 = setup_runway(1, airport.ID, "09", 3000);
    let runway2 = setup_runway(2, airport.ID, "27", 3000);

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

    let runway1 = setup_runway(1, 1, "09", 3000);
    let runway2 = setup_runway(2, 2, "27", 3000);

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
    let aircraft_with_distance = setup_aircraft(1, 0, None, Some(3000));
    let aircraft_without_distance = setup_aircraft(2, 0, None, None);

    let departure_airport = setup_airport(1, "Departure Airport", "DEP", 0.0, 0.0);
    let suitable_airport = setup_airport(2, "Suitable Airport", "SAT", 0.5, 0.5);
    let unsuitable_airport = setup_airport(3, "Unsuitable Airport", "USAT", 0.5, 0.5);

    let suitable_runway = setup_runway(1, suitable_airport.ID, "09", (3500.0 * M_TO_FEET) as i32);
    let short_runway = setup_runway(2, unsuitable_airport.ID, "27", (2500.0 * M_TO_FEET) as i32);

    insert_airport(connection, &departure_airport).unwrap();
    insert_airport(connection, &suitable_airport).unwrap();
    insert_airport(connection, &unsuitable_airport).unwrap();
    insert_runway(connection, &suitable_runway).unwrap();
    insert_runway(connection, &short_runway).unwrap();

    let result = get_destination_airport(connection, &aircraft_with_distance, &departure_airport);
    match result {
        Ok(airport) => {
            assert_eq!(airport, suitable_airport);
        }
        Err(e) => {
            panic!("Failed to get destination airport: {:?}", e);
        }
    }

    let result =
        get_destination_airport(connection, &aircraft_without_distance, &departure_airport);
    match result {
        Ok(airport) => {
            assert!(airport == suitable_airport || airport == unsuitable_airport);
        }
        Err(e) => {
            panic!("Failed to get destination airport: {:?}", e);
        }
    }
}

#[test]
fn test_mark_all_aircraft_unflown() {
    let connection = &mut initialize_aircraft_db();
    let aircraft = setup_aircraft(1, 1, Some("2021-01-01".to_string()), None);

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

    let aircraft1 = setup_aircraft(1, 0, None, None);
    let aircraft2 = setup_aircraft(2, 0, None, None);

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
    let aircraft = setup_aircraft(1, 1, Some("2021-01-01".to_string()), None);

    let departure = setup_airport(1, "Departure Airport", "DEP", 0.0, 0.0);
    let arrival = setup_airport(2, "Arrival Airport", "ARR", 0.0, 1.0);

    insert_aircraft(connection, &aircraft).unwrap();
    add_to_history(connection, &departure, &arrival, &aircraft).unwrap();

    let result = get_history(connection).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].arrival_icao, arrival.ICAO);
    assert_eq!(result[0].departure_icao, departure.ICAO);
    assert_eq!(result[0].aircraft, aircraft.id);

    let result = add_to_history(connection, &departure, &arrival, &aircraft);
    assert!(result.is_ok(), "Expected success when adding to history");
}

#[test]
fn test_random_unflown_aircraft() {
    let connection = &mut initialize_aircraft_db();

    let aircraft1 = setup_aircraft(1, 0, None, None);
    let aircraft2 = setup_aircraft(2, 1, Some("2021-01-01".to_string()), None);

    insert_aircraft(connection, &aircraft1).unwrap();
    insert_aircraft(connection, &aircraft2).unwrap();

    let result = random_unflown_aircraft(connection).unwrap();
    assert_eq!(result, aircraft1);
}

#[test]
fn test_get_destination_airport_no_options() {
    let connection = &mut initialize_aircraft_db();
    let aircraft = setup_aircraft(1, 0, None, None);
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
fn test_get_random_airport_for_aircraft() {
    let connection = &mut initialize_aircraft_db();

    let aircraft_with_distance = setup_aircraft(1, 0, None, Some(3000));
    let aircraft_without_distance = setup_aircraft(2, 0, None, None);
    let departure_airport = setup_airport(1, "Departure Airport", "DEP", 0.0, 0.0);
    let too_short_runway = setup_runway(1, 1, "09", (2000.0 * M_TO_FEET) as i32);
    let long_enough_runway = setup_runway(2, 1, "27", (3000.0 * M_TO_FEET) as i32);

    insert_airport(connection, &departure_airport).unwrap();
    insert_runway(connection, &too_short_runway).unwrap();

    let result = get_random_airport_for_aircraft(connection, &aircraft_with_distance);
    assert!(
        result.is_err(),
        "Expected error when no suitable airports are available"
    );

    let result = get_random_airport_for_aircraft(connection, &aircraft_without_distance);
    assert!(
        result.is_ok(),
        "Expected success when no runway length is available"
    );

    insert_runway(connection, &long_enough_runway).unwrap();
    let result = get_random_airport_for_aircraft(connection, &aircraft_with_distance).unwrap();
    assert_eq!(result, departure_airport);
}

#[test]

fn test_get_destination_airport_with_suitable_runway_haversine() {
    let connection = &mut initialize_aircraft_db();

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

    insert_airport(connection, &departure_airport).unwrap();
    insert_airport(connection, &airport_within_bounding_box).unwrap();
    insert_runway(connection, &runway_within_bounding_box).unwrap();

    let result = get_destination_airport_with_suitable_runway(
        connection,
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
        "Expected error when no suitable airports are available"
    );
}

#[test]
fn test_get_destination_airport_with_suitable_runway() {
    let connection = &mut initialize_aircraft_db();

    // Setup aircraft with minimum takeoff distance
    let aircraft = setup_aircraft(1, 0, None, Some(3000));

    // Setup airports and runways
    let departure_airport = setup_airport(1, "Departure Airport", "DEP", 0.0, 0.0);
    let suitable_airport = setup_airport(2, "Suitable Airport", "SAT", 0.5, 0.5);
    let unsuitable_airport = setup_airport(3, "Unsuitable Airport", "USAT", 0.5, 0.5);

    let suitable_runway = setup_runway(1, suitable_airport.ID, "09", (3500.0 * M_TO_FEET) as i32);
    let short_runway = setup_runway(2, unsuitable_airport.ID, "27", (2500.0 * M_TO_FEET) as i32);

    // Test - Unsuitable airport
    let result = get_destination_airport_with_suitable_runway(
        connection,
        &departure_airport,
        aircraft.aircraft_range,
        aircraft.takeoff_distance.unwrap(),
    );

    assert!(
        result.is_err(),
        "Expected error when no suitable airports are available"
    );

    // Insert data into the database
    insert_airport(connection, &departure_airport).unwrap();
    insert_airport(connection, &suitable_airport).unwrap();
    insert_airport(connection, &unsuitable_airport).unwrap();
    insert_runway(connection, &suitable_runway).unwrap();
    insert_runway(connection, &short_runway).unwrap();

    // Test - Suitable airport
    let result = get_destination_airport_with_suitable_runway(
        connection,
        &departure_airport,
        aircraft.aircraft_range,
        aircraft.takeoff_distance.unwrap(),
    )
    .unwrap();

    assert_eq!(result, suitable_airport);
}

#[test]
fn test_get_airport_within_distance() {
    let connection = &mut initialize_aircraft_db();

    // Setup aircraft without minimum takeoff distance
    let aircraft = setup_aircraft(1, 0, None, None);

    // Setup airports
    let departure_airport = setup_airport(1, "Departure Airport", "DEP", 0.0, 0.0);
    let within_range_airport = setup_airport(2, "Within Range Airport", "WRA", 0.5, 0.5);
    let outside_range_airport = setup_airport(3, "Outside Range Airport", "ORA", 10.0, 10.0);

    // Insert data into the database
    insert_airport(connection, &departure_airport).unwrap();
    insert_airport(connection, &within_range_airport).unwrap();
    insert_airport(connection, &outside_range_airport).unwrap();

    // Test function
    let result =
        get_airport_within_distance(connection, &departure_airport, aircraft.aircraft_range)
            .unwrap();

    assert_eq!(result, within_range_airport);
}

#[test]
fn test_get_airport_within_distance_haversine() {
    let connection = &mut initialize_aircraft_db();

    // Setup aircraft with minimum takeoff distance
    let aircraft = setup_aircraft(1, 0, None, Some(3000));

    // Setup airports
    let departure_airport = setup_airport(1, "Departure Airport", "DEP", 0.0, 0.0);
    let outside_range_airport = setup_airport(2, "Within Range Airport", "WRA", 1.66, 1.66);
    insert_airport(connection, &departure_airport).unwrap();

    // Test - Outside range
    let result =
        get_airport_within_distance(connection, &departure_airport, aircraft.aircraft_range);
    assert!(
        result.is_err(),
        "Expected error when no airports are within range"
    );

    // Insert data into the database
    insert_airport(connection, &outside_range_airport).unwrap();

    // Test - outside haversine range
    let result =
        get_airport_within_distance(connection, &departure_airport, aircraft.aircraft_range);

    assert!(
        result.is_err(),
        "Expected error when no airports are within range"
    );
}

#[test]
fn test_get_aircraft_by_id() {
    let connection = &mut initialize_aircraft_db();

    let aircraft = setup_aircraft(1, 0, None, None);
    insert_aircraft(connection, &aircraft).unwrap();

    let retrieved_aircraft = get_aircraft_by_id(connection, aircraft.id).unwrap();

    assert_eq!(retrieved_aircraft, aircraft);

    let result = get_aircraft_by_id(connection, 2);
    assert!(result.is_err());

    let result = get_aircraft_by_id(connection, -1);
    assert!(result.is_err());
}

#[test]
fn test_insert_airport_with_invalid_data() {
    let connection = &mut initialize_aircraft_db();
    let invalid_airport = setup_airport(1, "", "", 0.0, 0.0);

    let result = insert_airport(connection, &invalid_airport);
    assert!(
        result.is_err(),
        "Expected error when inserting invalid airport data"
    );
}

#[test]
fn test_insert_aircraft_with_invalid_data() {
    let connection = &mut initialize_aircraft_db();
    let invalid_aircraft = setup_aircraft(1, -1, None, None);

    let result = insert_aircraft(connection, &invalid_aircraft);
    assert!(
        result.is_err(),
        "Expected error when inserting invalid aircraft data"
    );
}

#[test]
fn test_insert_runway_with_invalid_data() {
    let connection = &mut initialize_aircraft_db();
    let invalid_runway = setup_runway(1, 1, "", -100);

    let result = insert_runway(connection, &invalid_runway);
    assert!(
        result.is_err(),
        "Expected error when inserting invalid runway data"
    );
}

#[test]
fn test_get_aircraft_with_invalid_id() {
    let connection = &mut initialize_aircraft_db();
    let result = get_aircraft_by_id(connection, -1);

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
    let connection = &mut initialize_aircraft_db();
    let aircraft = setup_aircraft(1, 0, None, Some(3000));
    let airport = setup_airport(1, "Test Airport", "TST", 0.0, 0.0);

    insert_airport(connection, &airport).unwrap();

    let result = get_destination_airport(connection, &aircraft, &airport);
    assert!(
        result.is_err(),
        "Expected error when no runways are available"
    );
}

#[test]
fn test_get_random_airport_with_empty_database() {
    let connection = &mut initialize_aircraft_db();
    let result = get_random_airport(connection);

    assert!(
        result.is_err(),
        "Expected error when no airports are available"
    );
}

#[test]
fn test_validate_errors() {
    let invalid_data = ValidationError::InvalidData("Test".to_string());
    let database_error = ValidationError::DatabaseError("Test".to_string());
    let invalid_id = ValidationError::InvalidId(-1);

    assert_eq!(format!("{}", invalid_data), "Invalid data: Test");
    assert_eq!(format!("{}", database_error), "Database error: Test");
    assert_eq!(format!("{}", invalid_id), "Invalid ID: -1");
}

#[test]
fn test_read_id() {
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

#[test]
fn test_random_route_for_selected_aircraft() {
    let connection_aircraft = &mut initialize_aircraft_db();
    let connection_airport = &mut initialize_aircraft_db();
    let aircraft_id = || Ok("1".to_string());

    // Test with empty database
    assert!(random_route_for_selected_aircraft(
        connection_aircraft,
        connection_airport,
        aircraft_id
    )
    .is_err());

    // Setup test data
    let aircraft = setup_aircraft(1, 0, None, None);
    let departure = setup_airport(1, "Departure", "DEP", 0.0, 0.0);
    let arrival = setup_airport(2, "Arrival", "ARR", 0.0, 1.0);

    insert_aircraft(connection_aircraft, &aircraft).unwrap();
    insert_airport(connection_airport, &departure).unwrap();
    insert_airport(connection_airport, &arrival).unwrap();
    insert_runway(
        connection_airport,
        &setup_runway(1, departure.ID, "09", 3500),
    )
    .unwrap();
    insert_runway(connection_airport, &setup_runway(2, arrival.ID, "27", 3500)).unwrap();

    // Test with complete setup
    assert!(random_route_for_selected_aircraft(
        connection_aircraft,
        connection_airport,
        aircraft_id
    )
    .is_ok());
}

#[test]
fn test_show_random_airport() {
    let connection = &mut initialize_aircraft_db();
    let result = show_random_airport(connection);
    assert!(result.is_err());

    let airport = setup_airport(1, "Test Airport", "TST", 0.0, 0.0);
    let runway = setup_runway(1, airport.ID, "09", (3500.0 * M_TO_FEET) as i32);
    insert_airport(connection, &airport).unwrap();
    insert_runway(connection, &runway).unwrap();

    let result = show_random_airport(connection);
    assert!(result.is_ok());
}

#[test]
fn test_show_random_unflown_aircraft() {
    let connection = &mut initialize_aircraft_db();
    let result = show_random_unflown_aircraft(connection);
    assert!(result.is_err());

    let aircraft = setup_aircraft(1, 0, None, None);
    insert_aircraft(connection, &aircraft).unwrap();

    let result = show_random_unflown_aircraft(connection);
    assert!(result.is_ok());
}

#[test]
fn test_show_random_aircraft_with_random_airport() {
    let connection_aircraft = &mut initialize_aircraft_db();
    let connection_airport = &mut initialize_aircraft_db();

    let result = show_random_aircraft_with_random_airport(connection_aircraft, connection_airport);
    assert!(result.is_err());

    let aircraft = setup_aircraft(1, 0, None, None);
    insert_aircraft(connection_aircraft, &aircraft).unwrap();

    let result = show_random_aircraft_with_random_airport(connection_aircraft, connection_airport);
    assert!(result.is_err());

    let airport = setup_airport(1, "Test Airport", "TST", 0.0, 0.0);
    let runway = setup_runway(1, airport.ID, "09", (3500.0 * M_TO_FEET) as i32);
    insert_airport(connection_airport, &airport).unwrap();
    insert_runway(connection_airport, &runway).unwrap();

    let result = show_random_aircraft_with_random_airport(connection_aircraft, connection_airport);
    assert!(result.is_ok());
}

#[test]
fn test_show_random_aircraft_and_route() {
    let connection_aircraft = &mut initialize_aircraft_db();
    let connection_airport = &mut initialize_aircraft_db();

    // Test 1: Empty databases
    {
        let result = show_random_aircraft_and_route(connection_aircraft, connection_airport);
        assert!(result.is_err(), "Should fail with empty databases");
        assert!(result.is_err());
    }

    // Test 2: With aircraft but no airports
    {
        let aircraft = setup_aircraft(1, 0, None, Some(3000));
        insert_aircraft(connection_aircraft, &aircraft).unwrap();

        let result = show_random_aircraft_and_route(connection_aircraft, connection_airport);
        assert!(result.is_err(), "Should fail with no airports");
    }

    // Test 3: With airports but unsuitable runways
    {
        let departure = setup_airport(1, "Departure", "DEP", 0.0, 0.0);
        let arrival = setup_airport(2, "Arrival", "ARR", 0.5, 0.5);
        let short_runway = setup_runway(1, departure.ID, "09", (2000.0 * M_TO_FEET) as i32);

        insert_airport(connection_airport, &departure).unwrap();
        insert_airport(connection_airport, &arrival).unwrap();
        insert_runway(connection_airport, &short_runway).unwrap();

        let result = show_random_aircraft_and_route(connection_aircraft, connection_airport);
        assert!(result.is_err(), "Should fail with unsuitable runways");
    }

    // Test 4: Complete valid setup
    {
        // Add suitable runways to both airports
        let suitable_runway1 = setup_runway(2, 1, "27", (3500.0 * M_TO_FEET) as i32);
        let suitable_runway2 = setup_runway(3, 2, "09", (3500.0 * M_TO_FEET) as i32);

        insert_runway(connection_airport, &suitable_runway1).unwrap();
        insert_runway(connection_airport, &suitable_runway2).unwrap();

        let result = show_random_aircraft_and_route(connection_aircraft, connection_airport);
        assert!(result.is_ok(), "Should succeed with valid setup");
    }
}

#[test]
fn test_show_all_aircraft() {
    let connection = &mut initialize_aircraft_db();
    let result = show_all_aircraft(connection);
    assert!(result.is_ok());

    let aircraft = setup_aircraft(1, 0, None, None);
    insert_aircraft(connection, &aircraft).unwrap();

    let result = show_all_aircraft(connection);
    assert!(result.is_ok());
}

#[test]
fn test_show_history() {
    let connection = &mut initialize_aircraft_db();
    let result = show_history(connection);
    assert!(result.is_ok());

    let airport = setup_airport(1, "Test Airport", "TST", 0.0, 0.0);
    let aircraft = setup_aircraft(1, 0, None, None);
    add_to_history(connection, &airport, &airport, &aircraft).unwrap();

    let result = show_history(connection);
    assert!(result.is_err());

    insert_aircraft(connection, &aircraft).unwrap();
    let result = show_history(connection);
    assert!(result.is_ok());
}

#[test]
fn test_random_unflown_aircraft_and_route() {
    let connection_aircraft = &mut initialize_aircraft_db();
    let connection_airport = &mut initialize_aircraft_db();

    let y = || Ok('y');
    let n = || Ok('n');

    let result = random_unflown_aircraft_and_route(connection_aircraft, connection_airport, y);
    assert!(result.is_err());

    let aircraft = setup_aircraft(1, 0, None, None);
    insert_aircraft(connection_aircraft, &aircraft).unwrap();

    let result = random_unflown_aircraft_and_route(connection_aircraft, connection_airport, y);
    assert!(result.is_err());

    let airport = setup_airport(1, "Test Airport", "TST", 0.0, 0.0);
    let runway = setup_runway(1, airport.ID, "09", (3500.0 * M_TO_FEET) as i32);
    insert_airport(connection_airport, &airport).unwrap();
    insert_runway(connection_airport, &runway).unwrap();

    let result = random_unflown_aircraft_and_route(connection_aircraft, connection_airport, y);
    assert!(result.is_err());

    let destination_airport = setup_airport(2, "Destination Airport", "DST", 0.0, 1.0);
    let destination_runway =
        setup_runway(2, destination_airport.ID, "27", (3500.0 * M_TO_FEET) as i32);
    insert_airport(connection_airport, &destination_airport).unwrap();
    insert_runway(connection_airport, &destination_runway).unwrap();

    let result = random_unflown_aircraft_and_route(connection_aircraft, connection_airport, n);
    assert!(result.is_ok());

    let result = random_unflown_aircraft_and_route(connection_aircraft, connection_airport, y);
    assert!(result.is_ok());

    let result = random_unflown_aircraft_and_route(connection_aircraft, connection_airport, y);
    assert!(result.is_err());

    let aircraft = get_aircraft_by_id(connection_aircraft, aircraft.id).unwrap();
    assert_eq!(aircraft.flown, 1);
}

#[test]
fn test_ask_mask_flown() {
    let aircraft_connection = &mut initialize_aircraft_db();
    let mut aircraft = setup_aircraft(1, 0, None, None);
    insert_aircraft(aircraft_connection, &aircraft).unwrap();

    let y = || Ok('y');
    let n = || Ok('n');

    let result = ask_mark_flown(aircraft_connection, &mut aircraft, n);
    assert!(result.is_ok());

    let aircraft_result = get_aircraft_by_id(aircraft_connection, aircraft.id).unwrap();
    assert_eq!(aircraft_result.flown, 0);

    let result = ask_mark_flown(aircraft_connection, &mut aircraft, y);
    assert!(result.is_ok());

    let aircraft_result = get_aircraft_by_id(aircraft_connection, aircraft.id).unwrap();
    assert_eq!(aircraft_result.flown, 1);
}
