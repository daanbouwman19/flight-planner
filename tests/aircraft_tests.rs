use diesel::RunQueryDsl;
use diesel::connection::SimpleConnection;
use diesel::{Connection, SqliteConnection};
mod common;

use common::create_test_aircraft;
use flight_planner::database::DatabaseConnections;
use flight_planner::models::{Aircraft, NewAircraft};
use flight_planner::modules::aircraft::*;
use flight_planner::traits::AircraftOperations;

const AIRCRAFT_TABLE_SQL: &str = "
    CREATE TABLE aircraft (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        manufacturer TEXT NOT NULL,
        variant TEXT NOT NULL,
        icao_code TEXT NOT NULL,
        flown INTEGER NOT NULL,
        aircraft_range INTEGER NOT NULL,
        category TEXT NOT NULL,
        cruise_speed INTEGER NOT NULL,
        date_flown TEXT,
        takeoff_distance INTEGER
    );
";

pub fn setup_test_db() -> DatabaseConnections {
    let aircraft_connection = SqliteConnection::establish(":memory:").unwrap();
    let airport_connection = SqliteConnection::establish(":memory:").unwrap();

    let mut database_connections = DatabaseConnections {
        aircraft_connection,
        airport_connection,
    };

    database_connections
        .aircraft_connection
        .batch_execute(&format!(
            "
            {AIRCRAFT_TABLE_SQL}
            INSERT INTO aircraft (manufacturer, variant, icao_code, flown, aircraft_range, category, cruise_speed, date_flown, takeoff_distance)
            VALUES ('Boeing', '737-800', 'B738', 0, 3000, 'A', 450, '2024-12-10', 2000),
                   ('Airbus', 'A320', 'A320', 1, 2500, 'A', 430, NULL, 1800),
                   ('Boeing', '777-300ER', 'B77W', 0, 6000, 'A', 500, NULL, 2500);
            "
        ))
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
fn test_get_aircraft_by_id() {
    let mut database_connections = setup_test_db();
    let record = database_connections.get_aircraft_by_id(1).unwrap();
    assert_eq!(record.manufacturer, "Boeing");
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
    let record = Aircraft {
        date_flown: Some("2024-12-10".to_string()),
        ..create_test_aircraft(1, "Boeing", "737-800", "B738")
    };

    let formatted = format_aircraft(&record);
    assert_eq!(
        formatted,
        "id: 1, Boeing 737-800 (B738), range: 3000, category: A, cruise speed: 450 knots, takeoff distance: 2000 m"
    );

    let aircraft_without_icao = Aircraft {
        icao_code: String::new(),
        date_flown: Some("2024-12-10".to_string()),
        ..create_test_aircraft(1, "Boeing", "737-800", "B738")
    };

    let formatted = format_aircraft(&aircraft_without_icao);
    assert_eq!(
        formatted,
        "id: 1, Boeing 737-800, range: 3000, category: A, cruise speed: 450 knots, takeoff distance: 2000 m"
    );

    let aircraft_without_takeoff_distance = Aircraft {
        takeoff_distance: None,
        date_flown: Some("2024-12-10".to_string()),
        ..create_test_aircraft(1, "Boeing", "737-800", "B738")
    };

    let formatted = format_aircraft(&aircraft_without_takeoff_distance);
    assert_eq!(
        formatted,
        "id: 1, Boeing 737-800 (B738), range: 3000, category: A, cruise speed: 450 knots, takeoff distance: unknown"
    );
}

#[test]
fn test_add_aircraft() {
    let mut database_connections = setup_test_db();
    let new_aircraft = NewAircraft {
        manufacturer: "Boeing".to_string(),
        variant: "787-9".to_string(),
        icao_code: "B789".to_string(),
        flown: 0,
        aircraft_range: 7000,
        category: "A".to_string(),
        cruise_speed: 500,
        date_flown: None,
        takeoff_distance: Some(3000),
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
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::Path;

    let csv_path = Path::new("test_aircraft_basic.csv");

    struct FileGuard<'a>(&'a Path);
    impl<'a> Drop for FileGuard<'a> {
        fn drop(&mut self) {
            let _ = fs::remove_file(self.0);
        }
    }
    let _guard = FileGuard(csv_path);

    let mut file = File::create(csv_path).unwrap();
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
    conn.batch_execute(AIRCRAFT_TABLE_SQL).unwrap();

    let result = import_aircraft_from_csv_if_empty(&mut conn, csv_path);
    assert!(result.unwrap());

    let imported_aircraft: Vec<Aircraft> = aircraft.load(&mut conn).unwrap();
    assert_eq!(imported_aircraft.len(), 1);

    let record = &imported_aircraft[0];
    assert_eq!(record.manufacturer, "Boeing");
    assert_eq!(record.variant, "777-200ER");
}

#[test]
fn test_import_aircraft_skips_when_not_empty() {
    use std::path::Path;
    let csv_path = Path::new("dummy.csv");
    // We don't even create the file because logic should skip before file open
    // BUT checking logic reads file open first in some implementations?
    // Wait, implementation check `count > 0` FIRST. Line 100 in aircraft.rs.

    let mut conn = SqliteConnection::establish(":memory:").unwrap();
    conn.batch_execute("
        CREATE TABLE aircraft (
            id INTEGER PRIMARY KEY,
            manufacturer TEXT,
            variant TEXT,
            icao_code TEXT,
            flown INTEGER,
            aircraft_range INTEGER,
            category TEXT,
            cruise_speed INTEGER,
            date_flown TEXT,
            takeoff_distance INTEGER
        );
        INSERT INTO aircraft (id, manufacturer, variant, icao_code, flown, aircraft_range, category, cruise_speed, date_flown, takeoff_distance)
        VALUES (1, 'Test', 'T', 'T', 0, 1000, 'A', 100, NULL, 100);
    ").unwrap();

    let result = import_aircraft_from_csv_if_empty(&mut conn, csv_path);
    // Should return Ok(false) because table is not empty
    assert!(matches!(result, Ok(false)));
}

#[test]
fn test_import_aircraft_skips_malformed_rows() {
    use flight_planner::schema::aircraft::dsl::aircraft;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::Path;

    let csv_path = Path::new("test_aircraft_malformed.csv");

    struct FileGuard<'a>(&'a Path);
    impl<'a> Drop for FileGuard<'a> {
        fn drop(&mut self) {
            let _ = fs::remove_file(self.0);
        }
    }
    let _guard = FileGuard(csv_path);

    let mut file = File::create(csv_path).unwrap();
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
    conn.batch_execute(AIRCRAFT_TABLE_SQL).unwrap();

    let result = import_aircraft_from_csv_if_empty(&mut conn, csv_path);
    assert!(result.unwrap()); // Should return true as at least one row was imported

    let imported_aircraft: Vec<Aircraft> = aircraft.load(&mut conn).unwrap();
    assert_eq!(imported_aircraft.len(), 1);
    assert_eq!(imported_aircraft[0].manufacturer, "Boeing");
}
