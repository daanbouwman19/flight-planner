use flight_planner::models::NewAircraft;
use flight_planner::console_utils::{read_id, read_yn, ask_mark_flown};
use flight_planner::traits::AircraftOperations;
use flight_planner::database::DatabaseConnections;
use diesel::connection::SimpleConnection;
use diesel::{Connection, SqliteConnection};

fn setup_test_db() -> DatabaseConnections {
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
                flown INTEGER NOT NULL DEFAULT 0,
                date_flown TEXT,
                aircraft_range INTEGER NOT NULL,
                category TEXT NOT NULL,
                cruise_speed INTEGER NOT NULL,
                takeoff_distance INTEGER
            );
            ",
        )
        .unwrap();

    database_connections
}

#[test]
fn test_read_id() {
    // Test with valid input
    let input = "123\n";
    let result = read_id(|| -> Result<String, std::io::Error> { Ok(input.to_string()) });
    assert_eq!(result, Ok(123));

    // Test with invalid input (not a number)
    let input = "abc\n";
    let result = read_id(|| -> Result<String, std::io::Error> { Ok(input.to_string()) });
    assert_eq!(
        result.unwrap_err().to_string(),
        "Invalid data: Invalid id".to_string()
    );

    // Test with invalid input (negative number)
    let input = "-5\n";
    let result = read_id(|| -> Result<String, std::io::Error> { Ok(input.to_string()) });
    assert_eq!(
        result.unwrap_err().to_string(),
        "Invalid ID: -5".to_string()
    );
}

#[test]
fn test_read_yn() {
    // Test with valid input "y"
    let result = read_yn(|| -> Result<char, std::io::Error> { Ok('y') });
    assert_eq!(result, Ok(true));

    // Test with valid input "n"
    let result = read_yn(|| -> Result<char, std::io::Error> { Ok('n') });
    assert_eq!(result, Ok(false));

    // Test with invalid input
    let result = read_yn(|| -> Result<char, std::io::Error> { Ok('x') });
    assert_eq!(
        result.unwrap_err().to_string(),
        "Invalid data: Invalid input".to_string()
    );
}

#[test]
fn test_ask_mark_flown() {
    // Setup test database using a helper function from aircraft module tests
    let mut db = setup_test_db();
    let aircraft = NewAircraft {
        manufacturer: "Boeing".to_string(),
        variant: "747-400".to_string(),
        icao_code: "B744".to_string(),
        flown: 0,
        date_flown: None,
        aircraft_range: 7260,
        category: "Heavy".to_string(),
        cruise_speed: 490,
        takeoff_distance: Some(9000),
    };
    db.add_aircraft(&aircraft).unwrap();
    let mut aircraft = db.get_all_aircraft().unwrap().pop().unwrap();

    // Test with user confirming
    let result: Result<(), flight_planner::errors::Error> = ask_mark_flown(&mut db, &mut aircraft, || Ok('y'));
    assert!(result.is_ok());
    assert_eq!(aircraft.flown, 1);
    assert!(aircraft.date_flown.is_some());

    // Reset the aircraft
    aircraft.flown = 0;
    aircraft.date_flown = None;

    // Test with user declining
    let result = ask_mark_flown(&mut db, &mut aircraft, || Ok('n'));
    assert!(result.is_ok());
    assert_eq!(aircraft.flown, 0);
    assert!(aircraft.date_flown.is_none());
}
