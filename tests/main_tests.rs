use diesel::connection::SimpleConnection;
use diesel::{Connection, SqliteConnection};
use flight_planner::console_utils::{ask_mark_flown, read_id, read_yn};
use flight_planner::database::DatabaseConnections;
use flight_planner::errors::Error;
use flight_planner::models::{Aircraft, NewAircraft};
use flight_planner::traits::AircraftOperations;

// Custom error for mocking database failures
#[derive(Debug)]
struct MockDbError(String);

impl std::fmt::Display for MockDbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for MockDbError {}

// Mock database for testing failure scenarios
#[derive(Default)]
struct MockAircraftDb {
    update_should_fail: bool,
}

impl AircraftOperations for MockAircraftDb {
    fn get_not_flown_count(&mut self) -> Result<i64, diesel::result::Error> {
        unimplemented!()
    }

    fn random_not_flown_aircraft(&mut self) -> Result<Aircraft, diesel::result::Error> {
        unimplemented!()
    }

    fn get_all_aircraft(&mut self) -> Result<Vec<Aircraft>, diesel::result::Error> {
        unimplemented!()
    }

    fn update_aircraft(&mut self, _record: &Aircraft) -> Result<(), diesel::result::Error> {
        if self.update_should_fail {
            Err(diesel::result::Error::QueryBuilderError(Box::new(
                MockDbError("Simulated database error".to_string()),
            )))
        } else {
            Ok(())
        }
    }

    fn random_aircraft(&mut self) -> Result<Aircraft, diesel::result::Error> {
        unimplemented!()
    }

    fn get_aircraft_by_id(&mut self, _aircraft_id: i32) -> Result<Aircraft, diesel::result::Error> {
        unimplemented!()
    }

    fn mark_all_aircraft_not_flown(&mut self) -> Result<(), diesel::result::Error> {
        unimplemented!()
    }

    #[allow(dead_code)]
    fn add_aircraft(&mut self, _record: &NewAircraft) -> Result<Aircraft, diesel::result::Error> {
        unimplemented!()
    }
}

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

    // Test with zero, which is an invalid ID
    let input = "0\n";
    let result = read_id(|| -> Result<String, std::io::Error> { Ok(input.to_string()) });
    assert_eq!(result.unwrap_err().to_string(), "Invalid ID: 0".to_string());

    // Test with extra whitespace
    let input = "  42  \n";
    let result = read_id(|| -> Result<String, std::io::Error> { Ok(input.to_string()) });
    assert_eq!(result, Ok(42));

    // Test with an I/O error
    let result =
        read_id(|| -> Result<String, std::io::Error> { Err(std::io::Error::other("test error")) });
    assert_eq!(
        result.unwrap_err().to_string(),
        "Invalid data: test error".to_string()
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

    // Test with an I/O error
    let result =
        read_yn(|| -> Result<char, std::io::Error> { Err(std::io::Error::other("test error")) });
    assert_eq!(
        result.unwrap_err().to_string(),
        "Invalid data: test error".to_string()
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
    let result: Result<(), flight_planner::errors::Error> =
        ask_mark_flown(&mut db, &mut aircraft, || Ok('y'));
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

    // Test with invalid input from user
    let result = ask_mark_flown(&mut db, &mut aircraft, || Ok('x'));
    assert!(result.is_ok());
    assert_eq!(aircraft.flown, 0); // Should not change
    assert!(aircraft.date_flown.is_none()); // Should not change

    // Test with a database error
    let mut mock_db = MockAircraftDb {
        update_should_fail: true,
    };
    let result = ask_mark_flown(&mut mock_db, &mut aircraft, || Ok('y'));
    let err = result.unwrap_err();
    assert_eq!(
        err.to_string(),
        "Database error: Simulated database error"
    );
    assert!(matches!(
        err,
        Error::Diesel(diesel::result::Error::QueryBuilderError(_))
    ));
}
