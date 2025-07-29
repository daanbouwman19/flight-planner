use diesel::connection::SimpleConnection;
use diesel::{Connection, SqliteConnection};
use flight_planner::database::DatabaseConnections;
use flight_planner::models::{Aircraft, NewAircraft};
use flight_planner::modules::aircraft::*;
use flight_planner::traits::AircraftOperations;

pub fn setup_test_db() -> DatabaseConnections {
    let aircraft_connection = SqliteConnection::establish(":memory:").unwrap();
    let airport_connection = SqliteConnection::establish(":memory:").unwrap();

    let mut database_connections = DatabaseConnections {
        aircraft_connection,
        airport_connection,
    };

    database_connections
        .aircraft_connection
        .batch_execute(
            "
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
            INSERT INTO aircraft (manufacturer, variant, icao_code, flown, aircraft_range, category, cruise_speed, date_flown, takeoff_distance)
            VALUES ('Boeing', '737-800', 'B738', 0, 3000, 'A', 450, '2024-12-10', 2000),
                   ('Airbus', 'A320', 'A320', 1, 2500, 'A', 430, NULL, 1800),
                   ('Boeing', '777-300ER', 'B77W', 0, 6000, 'A', 500, NULL, 2500);
            ",
        )
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
        id: 1,
        manufacturer: "Boeing".to_string(),
        variant: "737-800".to_string(),
        icao_code: "B738".to_string(),
        flown: 0,
        aircraft_range: 3000,
        category: "A".to_string(),
        cruise_speed: 450,
        date_flown: Some("2024-12-10".to_string()),
        takeoff_distance: Some(2000),
    };

    let formatted = format_aircraft(&record);
    assert_eq!(
        formatted,
        "id: 1, Boeing 737-800 (B738), range: 3000, category: A, cruise speed: 450 knots, takeoff distance: 2000 m"
    );

    let aircraft_without_icao = Aircraft {
        id: 1,
        manufacturer: "Boeing".to_string(),
        variant: "737-800".to_string(),
        icao_code: String::new(),
        flown: 0,
        aircraft_range: 3000,
        category: "A".to_string(),
        cruise_speed: 450,
        date_flown: Some("2024-12-10".to_string()),
        takeoff_distance: Some(2000),
    };

    let formatted = format_aircraft(&aircraft_without_icao);
    assert_eq!(
        formatted,
        "id: 1, Boeing 737-800, range: 3000, category: A, cruise speed: 450 knots, takeoff distance: 2000 m"
    );

    let aircraft_without_takeoff_distance = Aircraft {
        id: 1,
        manufacturer: "Boeing".to_string(),
        variant: "737-800".to_string(),
        icao_code: "B738".to_string(),
        flown: 0,
        aircraft_range: 3000,
        category: "A".to_string(),
        cruise_speed: 450,
        date_flown: Some("2024-12-10".to_string()),
        takeoff_distance: None,
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
