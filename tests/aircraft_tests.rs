use diesel::RunQueryDsl;
use diesel::connection::SimpleConnection;
use diesel::{Connection, SqliteConnection};
mod common;

use common::create_test_aircraft;
use flight_planner::database::DatabaseConnections;
use flight_planner::models::{Aircraft, NewAircraft};
use flight_planner::modules::aircraft::*;
use flight_planner::traits::AircraftOperations;

pub fn setup_test_db() -> DatabaseConnections {
    let mut aircraft_connection =
        SqliteConnection::establish(":memory:").expect("Failed to create in-memory database");
    let airport_connection =
        SqliteConnection::establish(":memory:").expect("Failed to create in-memory database");

    common::create_aircraft_schema(&mut aircraft_connection);

    let mut database_connections = DatabaseConnections {
        aircraft_connection,
        airport_connection,
    };

    let aircrafts = vec![
        NewAircraft {
            date_flown: Some("2024-12-10".to_string()),
            ..common::create_test_new_aircraft()
        },
        NewAircraft {
            manufacturer: "Airbus".to_string(),
            variant: "A320".to_string(),
            icao_code: "A320".to_string(),
            flown: 1,
            aircraft_range: 2500,
            cruise_speed: 430,
            takeoff_distance: Some(1800),
            ..common::create_test_new_aircraft()
        },
        NewAircraft {
            variant: "777-300ER".to_string(),
            icao_code: "B77W".to_string(),
            aircraft_range: 6000,
            cruise_speed: 500,
            takeoff_distance: Some(2500),
            ..common::create_test_new_aircraft()
        },
    ];

    use flight_planner::schema::aircraft::dsl::aircraft;
    diesel::insert_into(aircraft)
        .values(&aircrafts)
        .execute(&mut database_connections.aircraft_connection)
        .expect("Failed to create test data");

    database_connections
}

#[test]
fn test_get_not_flown_count() {
    let mut database_connections = setup_test_db();
    let count = database_connections.get_not_flown_count().unwrap();
    assert_eq!(count, 2);
}

#[test]
fn test_random_not_flown_aircraft() {
    let mut database_connections = setup_test_db();
    let record = database_connections.random_not_flown_aircraft().unwrap();
    assert_eq!(record.flown, 0);
}

#[test]
fn test_get_all_aircraft() {
    let mut database_connections = setup_test_db();
    let record = database_connections.get_all_aircraft().unwrap();
    assert_eq!(record.len(), 3);
}

#[test]
fn test_update_aircraft() {
    let mut database_connections = setup_test_db();
    let mut record = database_connections.random_aircraft().unwrap();
    record.flown = 1;
    database_connections.update_aircraft(&record).unwrap();

    let updated_aircraft = database_connections.get_aircraft_by_id(record.id).unwrap();
    assert_eq!(updated_aircraft.flown, 1);
}

#[test]
fn test_random_aircraft() {
    let mut database_connections = setup_test_db();
    let record = database_connections.random_aircraft().unwrap();
    assert!(record.id > 0);
}

#[test]
fn test_get_aircraft_by_id_parameterized() {
    let mut database_connections = setup_test_db();

    let test_cases = vec![
        (1, true, "Valid ID"),
        (999, false, "Non-existent ID"),
        (0, false, "Invalid ID (0)"),
        (-1, false, "Invalid ID (-1)"),
    ];

    for (id, should_succeed, description) in test_cases {
        let result = database_connections.get_aircraft_by_id(id);

        if should_succeed {
            assert!(result.is_ok(), "Failed case: {}", description);
            let record = result.unwrap();
            assert_eq!(record.id, id);
            if id == 1 {
                assert_eq!(record.manufacturer, "Boeing");
            }
        } else {
            assert!(result.is_err(), "Failed case: {}", description);
            let err = result.unwrap_err();
            assert!(
                matches!(
                    err,
                    flight_planner::errors::Error::Diesel(diesel::result::Error::NotFound)
                ),
                "Expected NotFound error for case '{}', got: {:?}",
                description,
                err
            );
        }
    }
}

#[test]
fn test_mark_all_aircraft_not_flown() {
    let mut database_connections = setup_test_db();
    database_connections.mark_all_aircraft_not_flown().unwrap();

    let record = database_connections.get_all_aircraft().unwrap();
    assert!(record.iter().all(|a| a.flown == 0));
}

#[test]
fn test_format_aircraft() {
    let base_aircraft = create_test_aircraft(1, "Boeing", "737-800", "B738");

    let test_cases = vec![
        (
            "Full details",
            Aircraft {
                date_flown: Some("2024-12-10".to_string()),
                ..base_aircraft.clone()
            },
            "id: 1, Boeing 737-800 (B738), range: 3000, category: A, cruise speed: 450 knots, takeoff distance: 2000 m",
        ),
        (
            "Missing ICAO",
            Aircraft {
                icao_code: String::new(),
                date_flown: Some("2024-12-10".to_string()),
                ..base_aircraft.clone()
            },
            "id: 1, Boeing 737-800, range: 3000, category: A, cruise speed: 450 knots, takeoff distance: 2000 m",
        ),
        (
            "Missing Takeoff Distance",
            Aircraft {
                takeoff_distance: None,
                date_flown: Some("2024-12-10".to_string()),
                ..base_aircraft
            },
            "id: 1, Boeing 737-800 (B738), range: 3000, category: A, cruise speed: 450 knots, takeoff distance: unknown",
        ),
    ];

    for (description, aircraft, expected) in test_cases {
        let formatted = format_aircraft(&aircraft);
        assert_eq!(formatted, expected, "Failed case: {}", description);
    }
}

#[test]
fn test_add_aircraft() {
    let mut database_connections = setup_test_db();
    let new_aircraft = NewAircraft {
        variant: "787-9".to_string(),
        icao_code: "B789".to_string(),
        aircraft_range: 7000,
        cruise_speed: 500,
        takeoff_distance: Some(3000),
        ..common::create_test_new_aircraft()
    };

    let record = database_connections.add_aircraft(&new_aircraft).unwrap();
    assert_eq!(record.manufacturer, "Boeing");
    assert_eq!(record.variant, "787-9");
    assert_eq!(record.icao_code, "B789");
    assert_eq!(record.flown, 0);
    assert_eq!(record.aircraft_range, 7000);
    assert_eq!(record.category, "A");
    assert_eq!(record.cruise_speed, 500);
    assert_eq!(record.date_flown, None);
    assert_eq!(record.takeoff_distance, Some(3000));
}

#[test]
fn test_import_aircraft_from_csv_trims_whitespace() {
    use flight_planner::schema::aircraft::dsl::aircraft;
    use std::fs::File;
    use std::io::Write;

    let tmp_dir = common::TempDir::new("aircraft_import_test");
    let csv_path = tmp_dir.path.join("test_aircraft_basic.csv");

    let mut file = File::create(&csv_path).unwrap();
    writeln!(
        file,
        "manufacturer,variant,icao_code,flown,aircraft_range,category,cruise_speed,date_flown,takeoff_distance"
    )
    .unwrap();
    writeln!(
        file,
        "  Boeing  ,  777-200ER  ,B772,0,6000,Wide-body,482,,3000"
    )
    .unwrap();
    drop(file);

    let mut conn = SqliteConnection::establish(":memory:").unwrap();
    common::create_aircraft_schema(&mut conn);

    let result = import_aircraft_from_csv_if_empty(&mut conn, &csv_path);
    assert!(result.unwrap());

    let imported_aircraft: Vec<Aircraft> = aircraft.load(&mut conn).unwrap();
    assert_eq!(imported_aircraft.len(), 1);

    let record = &imported_aircraft[0];
    assert_eq!(record.manufacturer, "Boeing");
    assert_eq!(record.variant, "777-200ER");
}

#[test]
fn test_import_aircraft_skips_when_not_empty() {
    let tmp_dir = common::TempDir::new("aircraft_import_skip_test");
    let csv_path = tmp_dir.path.join("dummy.csv");
    std::fs::File::create(&csv_path).unwrap();

    let mut conn = SqliteConnection::establish(":memory:").unwrap();
    common::create_aircraft_schema(&mut conn);

    // Insert a dummy record to make it not empty
    conn.batch_execute("
        INSERT INTO aircraft (id, manufacturer, variant, icao_code, flown, aircraft_range, category, cruise_speed, date_flown, takeoff_distance)
        VALUES (1, 'Test', 'T', 'T', 0, 1000, 'A', 100, NULL, 100);
    ").unwrap();

    let result = import_aircraft_from_csv_if_empty(&mut conn, &csv_path);
    // Should return Ok(false) because table is not empty
    assert!(matches!(result, Ok(false)));
}

#[test]
fn test_import_aircraft_skips_malformed_rows() {
    use flight_planner::schema::aircraft::dsl::aircraft;
    use std::fs::File;
    use std::io::Write;

    let tmp_dir = common::TempDir::new("aircraft_import_malformed_test");
    let csv_path = tmp_dir.path.join("test_aircraft_malformed.csv");

    let mut file = File::create(&csv_path).unwrap();
    writeln!(
        file,
        "manufacturer,variant,icao_code,flown,aircraft_range,category,cruise_speed,date_flown,takeoff_distance"
    )
    .unwrap();
    // Valid row
    writeln!(file, "Boeing,737,B737,0,3000,A,450,,2000").unwrap();
    // Malformed row (missing columns)
    writeln!(file, "Airbus,A320,A320,0").unwrap();
    drop(file);

    let mut conn = SqliteConnection::establish(":memory:").unwrap();
    common::create_aircraft_schema(&mut conn);

    let result = import_aircraft_from_csv_if_empty(&mut conn, &csv_path);
    assert!(result.unwrap()); // Should return true as at least one row was imported

    let imported_aircraft: Vec<Aircraft> = aircraft.load(&mut conn).unwrap();
    assert_eq!(imported_aircraft.len(), 1);
    assert_eq!(imported_aircraft[0].manufacturer, "Boeing");
}

#[test]
fn test_import_aircraft_from_csv_file_not_found() {
    let tmp_dir = common::TempDir::new("aircraft_import_fail_test");
    let csv_path = tmp_dir.path.join("non_existent.csv");

    let mut conn = SqliteConnection::establish(":memory:").unwrap();
    common::create_aircraft_schema(&mut conn);

    let result = import_aircraft_from_csv_if_empty(&mut conn, &csv_path);

    assert!(result.is_err());
}
