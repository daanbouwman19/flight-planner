use diesel::RunQueryDsl;
use diesel::connection::SimpleConnection;
use diesel::{Connection, SqliteConnection};
use rand::prelude::*;
mod common;

use common::create_test_aircraft;
use flight_planner::database::DatabaseConnections;
use flight_planner::models::{Aircraft, NewAircraft};
use flight_planner::modules::aircraft::*;
use flight_planner::traits::AircraftOperations;

const AIRCRAFT_CSV_HEADER: &str = "manufacturer,variant,icao_code,flown,aircraft_range,category,cruise_speed,date_flown,takeoff_distance";

fn establish_optimized_connection(name: &str) -> SqliteConnection {
    let url = format!("file:{}?mode=memory&cache=shared", name);
    let mut conn = SqliteConnection::establish(&url).expect("Failed to create in-memory database");
    conn.batch_execute(
        "
        PRAGMA journal_mode = WAL;
        PRAGMA busy_timeout = 5000;
        PRAGMA synchronous = NORMAL;
    ",
    )
    .unwrap();
    conn
}

pub fn setup_test_db() -> DatabaseConnections {
    let mut rng = rand::rng();
    let aircraft_name = format!("memdb_aircraft_{}", rng.random::<u64>());
    let airport_name = format!("memdb_airport_{}", rng.random::<u64>());

    let mut aircraft_connection = establish_optimized_connection(&aircraft_name);
    let airport_connection = establish_optimized_connection(&airport_name);

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

fn setup_import_db() -> SqliteConnection {
    let mut rng = rand::rng();
    let name = format!("memdb_import_{}", rng.random::<u64>());
    let mut conn = establish_optimized_connection(&name);
    common::create_aircraft_schema(&mut conn);
    conn
}

fn create_csv(path: &std::path::Path, lines: &[&str]) {
    use std::fs::File;
    use std::io::Write;
    let mut file = File::create(path).expect("Failed to create CSV file");
    for line in lines {
        writeln!(file, "{}", line).expect("Failed to write to CSV file");
    }
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

    struct TestCase {
        id: i32,
        expected_manufacturer: Option<&'static str>,
        description: &'static str,
    }

    let test_cases = vec![
        TestCase {
            id: 1,
            expected_manufacturer: Some("Boeing"),
            description: "Valid ID",
        },
        TestCase {
            id: 999,
            expected_manufacturer: None,
            description: "Non-existent ID",
        },
        TestCase {
            id: 0,
            expected_manufacturer: None,
            description: "Invalid ID (0)",
        },
        TestCase {
            id: -1,
            expected_manufacturer: None,
            description: "Invalid ID (-1)",
        },
    ];

    for case in test_cases {
        let result = database_connections.get_aircraft_by_id(case.id);

        match case.expected_manufacturer {
            Some(manufacturer) => {
                assert!(result.is_ok(), "Failed case: {}", case.description);
                let record = result.unwrap();
                assert_eq!(
                    record.id, case.id,
                    "ID mismatch for case: {}",
                    case.description
                );
                assert_eq!(
                    record.manufacturer, manufacturer,
                    "Manufacturer mismatch for case: {}",
                    case.description
                );
            }
            None => {
                assert!(result.is_err(), "Failed case: {}", case.description);
                let err = result.unwrap_err();
                assert!(
                    matches!(
                        err,
                        flight_planner::errors::Error::Diesel(diesel::result::Error::NotFound)
                    ),
                    "Expected NotFound error for case '{}', got: {:?}",
                    case.description,
                    err
                );
            }
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
                ..base_aircraft.clone()
            },
            "id: 1, Boeing 737-800 (B738), range: 3000, category: A, cruise speed: 450 knots, takeoff distance: unknown",
        ),
        (
            "UTF-8 Manufacturer",
            Aircraft {
                manufacturer: "Möller".to_string(),
                ..base_aircraft.clone()
            },
            "id: 1, Möller 737-800 (B738), range: 3000, category: A, cruise speed: 450 knots, takeoff distance: 2000 m",
        ),
        (
            "Zero Takeoff Distance",
            Aircraft {
                takeoff_distance: Some(0),
                ..base_aircraft.clone()
            },
            "id: 1, Boeing 737-800 (B738), range: 3000, category: A, cruise speed: 450 knots, takeoff distance: 0 m",
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

    let tmp_dir = common::TempDir::new("aircraft_import_test");
    let csv_path = tmp_dir.path.join("test_aircraft_basic.csv");

    create_csv(
        &csv_path,
        &[
            AIRCRAFT_CSV_HEADER,
            "  Boeing  ,  777-200ER  ,  B772  ,0,6000,Wide-body,482,,3000",
        ],
    );

    let mut conn = setup_import_db();

    let result = import_aircraft_from_csv_if_empty(&mut conn, &csv_path);
    assert!(result.unwrap());

    let imported_aircraft: Vec<Aircraft> = aircraft.load(&mut conn).unwrap();
    assert_eq!(imported_aircraft.len(), 1);

    let record = &imported_aircraft[0];
    assert_eq!(record.manufacturer, "Boeing");
    assert_eq!(record.variant, "777-200ER");
    assert_eq!(record.icao_code, "B772");
}

#[test]
fn test_import_aircraft_skips_when_not_empty() {
    let tmp_dir = common::TempDir::new("aircraft_import_skip_test");
    let csv_path = tmp_dir.path.join("dummy.csv");
    create_csv(&csv_path, &[]);

    let mut conn = setup_import_db();

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

    let tmp_dir = common::TempDir::new("aircraft_import_malformed_test");
    let csv_path = tmp_dir.path.join("test_aircraft_malformed.csv");

    create_csv(
        &csv_path,
        &[
            AIRCRAFT_CSV_HEADER,
            "Boeing,737,B737,0,3000,A,450,,2000",
            "Airbus,A320,A320,0",
        ],
    );

    let mut conn = setup_import_db();

    let result = import_aircraft_from_csv_if_empty(&mut conn, &csv_path);
    assert!(result.unwrap()); // Should return true as at least one row was imported

    let imported_aircraft: Vec<Aircraft> = aircraft.load(&mut conn).unwrap();
    assert_eq!(imported_aircraft.len(), 1);
    assert_eq!(imported_aircraft[0].manufacturer, "Boeing");
}

#[test]
fn test_import_aircraft_empty_csv_returns_false() {
    let tmp_dir = common::TempDir::new("aircraft_import_empty_csv_test");
    let csv_path = tmp_dir.path.join("empty.csv");

    // CSV with only header
    create_csv(&csv_path, &[AIRCRAFT_CSV_HEADER]);

    let mut conn = setup_import_db();

    let result = import_aircraft_from_csv_if_empty(&mut conn, &csv_path);
    // Should return Ok(false) because no records found in CSV
    assert!(matches!(result, Ok(false)));
}

#[test]
fn test_import_aircraft_from_csv_file_not_found() {
    let tmp_dir = common::TempDir::new("aircraft_import_fail_test");
    let csv_path = tmp_dir.path.join("non_existent.csv");

    let mut conn = setup_import_db();

    let result = import_aircraft_from_csv_if_empty(&mut conn, &csv_path);

    assert!(result.is_err());
}

#[test]
fn test_import_aircraft_skips_invalid_data_types() {
    use flight_planner::schema::aircraft::dsl::aircraft;

    let tmp_dir = common::TempDir::new("aircraft_import_invalid_types_test");
    let csv_path = tmp_dir.path.join("test_aircraft_invalid_types.csv");

    // CSV header is already defined as AIRCRAFT_CSV_HEADER
    // manufacturer,variant,icao_code,flown,aircraft_range,category,cruise_speed,date_flown,takeoff_distance

    create_csv(
        &csv_path,
        &[
            AIRCRAFT_CSV_HEADER,
            "Boeing,737,B737,0,3000,A,450,,2000", // Valid row
            "Airbus,A320,A320,NOT_A_NUMBER,2500,A,430,,1800", // Invalid: 'flown' is string, expected int
            "Boeing,777,B777,0,6000,Wide-body,500,,2500",     // Valid row
        ],
    );

    let mut conn = setup_import_db();

    // The import should succeed (return true) because at least one valid row was imported
    // and invalid rows are skipped with a warning.
    let result = import_aircraft_from_csv_if_empty(&mut conn, &csv_path);
    assert!(
        result.unwrap(),
        "Import should succeed even with some invalid rows"
    );

    let imported_aircraft: Vec<Aircraft> = aircraft.load(&mut conn).unwrap();

    // Should have 2 records (the Airbus row should be skipped)
    assert_eq!(
        imported_aircraft.len(),
        2,
        "Should import exactly 2 valid aircraft"
    );

    // Verify specific records presence
    let manufacturers: Vec<String> = imported_aircraft
        .iter()
        .map(|a| a.manufacturer.clone())
        .collect();
    assert!(
        manufacturers.contains(&"Boeing".to_string()),
        "Should contain Boeing"
    );
    assert!(
        !manufacturers.contains(&"Airbus".to_string()),
        "Should NOT contain Airbus"
    );
}
