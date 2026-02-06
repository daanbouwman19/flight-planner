use flight_planner::console_utils::{ask_mark_flown, read_id, read_yn};
use flight_planner::errors::Error;
use flight_planner::models::{Aircraft, NewAircraft};
use flight_planner::traits::AircraftOperations;

mod common;

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
    fn get_not_flown_count(&mut self) -> Result<i64, Error> {
        unimplemented!()
    }

    fn random_not_flown_aircraft(&mut self) -> Result<Aircraft, Error> {
        unimplemented!()
    }

    fn get_all_aircraft(&mut self) -> Result<Vec<Aircraft>, Error> {
        unimplemented!()
    }

    fn update_aircraft(&mut self, _record: &Aircraft) -> Result<(), Error> {
        if self.update_should_fail {
            Err(
                diesel::result::Error::QueryBuilderError(Box::new(MockDbError(
                    "Simulated database error".to_string(),
                )))
                .into(),
            )
        } else {
            Ok(())
        }
    }

    fn random_aircraft(&mut self) -> Result<Aircraft, Error> {
        unimplemented!()
    }

    fn get_aircraft_by_id(&mut self, _aircraft_id: i32) -> Result<Aircraft, Error> {
        unimplemented!()
    }

    fn mark_all_aircraft_not_flown(&mut self) -> Result<(), Error> {
        unimplemented!()
    }

    #[allow(dead_code)]
    fn add_aircraft(&mut self, _record: &NewAircraft) -> Result<Aircraft, Error> {
        unimplemented!()
    }
}

#[test]
fn test_read_id() {
    let cases = vec![
        ("Valid input", Ok("123\n"), Ok(123)),
        (
            "Invalid input (not a number)",
            Ok("abc\n"),
            Err("Invalid data: Invalid id"),
        ),
        (
            "Invalid input (negative number)",
            Ok("-5\n"),
            Err("Invalid ID: -5"),
        ),
        ("Invalid input (zero)", Ok("0\n"), Err("Invalid ID: 0")),
        ("Extra whitespace", Ok("  42  \n"), Ok(42)),
        (
            "I/O Error",
            Err("test error"),
            Err("Invalid data: test error"),
        ),
    ];

    for (name, input, expected) in cases {
        let input_closure = || -> Result<String, std::io::Error> {
            match input {
                Ok(s) => Ok(s.to_string()),
                Err(e) => Err(std::io::Error::other(e)),
            }
        };

        let result = read_id(input_closure);
        match expected {
            Ok(val) => assert_eq!(result.unwrap(), val, "Failed case: {}", name),
            Err(msg) => assert_eq!(
                result.unwrap_err().to_string(),
                msg,
                "Failed case: {}",
                name
            ),
        }
    }
}

#[test]
fn test_read_yn() {
    let cases = vec![
        ("Valid 'y'", Ok('y'), Ok(true)),
        ("Valid 'n'", Ok('n'), Ok(false)),
        ("Invalid input", Ok('x'), Err("Invalid data: Invalid input")),
        (
            "I/O Error",
            Err("test error"),
            Err("Invalid data: test error"),
        ),
    ];

    for (name, input, expected) in cases {
        let input_closure = || -> Result<char, std::io::Error> {
            match input {
                Ok(c) => Ok(c),
                Err(e) => Err(std::io::Error::other(e)),
            }
        };

        let result = read_yn(input_closure);
        match expected {
            Ok(val) => assert_eq!(result.unwrap(), val, "Failed case: {}", name),
            Err(msg) => assert_eq!(
                result.unwrap_err().to_string(),
                msg,
                "Failed case: {}",
                name
            ),
        }
    }
}

#[test]
fn test_ask_mark_flown() {
    // Setup test database using shared helper
    let mut db = common::setup_test_db();
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
    // add_aircraft returns the inserted record (with ID)
    let mut aircraft = db.add_aircraft(&aircraft).unwrap();

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
    assert_eq!(err.to_string(), "Database error: Simulated database error");
    assert!(matches!(
        err,
        Error::Diesel(diesel::result::Error::QueryBuilderError(_))
    ));
}
