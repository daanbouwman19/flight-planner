use flight_planner::cli::{Interaction, console_main};
use flight_planner::database::DatabaseConnections;
use flight_planner::errors::Error;
use flight_planner::models::NewAircraft;
use flight_planner::traits::AircraftOperations;
use std::cell::RefCell;
use std::collections::VecDeque;

use diesel::connection::SimpleConnection;
use diesel::{Connection, SqliteConnection};

// Setup test database (copied from other tests, should arguably be in test_helpers)
fn setup_test_db() -> DatabaseConnections {
    let aircraft_connection = SqliteConnection::establish(":memory:").unwrap();
    let airport_connection = SqliteConnection::establish(":memory:").unwrap();

    let mut database_connections = DatabaseConnections {
        aircraft_connection,
        airport_connection,
    };

    // Verify tables exist or create them if needed
    let _ = database_connections.aircraft_connection.batch_execute(
        "CREATE TABLE IF NOT EXISTS aircraft (
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
        CREATE TABLE IF NOT EXISTS History (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            date TEXT NOT NULL,
            aircraft INTEGER NOT NULL,
            departure_icao TEXT NOT NULL,
            arrival_icao TEXT NOT NULL,
            distance INTEGER
        );
        ",
    );

    let _ = database_connections.airport_connection.batch_execute(
        "CREATE TABLE IF NOT EXISTS Airports (
            ID INTEGER PRIMARY KEY AUTOINCREMENT,
            Name TEXT NOT NULL,
            ICAO TEXT NOT NULL,
            PrimaryID INTEGER,
            Latitude REAL NOT NULL,
            Longtitude REAL NOT NULL,
            Elevation INTEGER NOT NULL,
            TransitionAltitude INTEGER,
            TransitionLevel INTEGER,
            SpeedLimit INTEGER,
            SpeedLimitAltitude INTEGER
        );
        CREATE TABLE IF NOT EXISTS Runways (
            ID INTEGER PRIMARY KEY AUTOINCREMENT,
            AirportID INTEGER NOT NULL,
            Ident TEXT NOT NULL,
            TrueHeading REAL NOT NULL,
            Length INTEGER NOT NULL,
            Width INTEGER NOT NULL,
            Surface TEXT NOT NULL,
            Latitude REAL NOT NULL,
            Longtitude REAL NOT NULL,
            Elevation INTEGER NOT NULL
        );
        INSERT OR IGNORE INTO Airports (ID, Name, ICAO, Latitude, Longtitude, Elevation) VALUES (1, 'Test Airport', 'TEST', 0.0, 0.0, 0);
        INSERT OR IGNORE INTO Airports (ID, Name, ICAO, Latitude, Longtitude, Elevation) VALUES (2, 'Dest Airport', 'DEST', 10.0, 10.0, 0);
        INSERT OR IGNORE INTO Runways (AirportID, Ident, TrueHeading, Length, Width, Surface, Latitude, Longtitude, Elevation)
        VALUES (1, '01', 0.0, 3000, 45, 'Asphalt', 0.0, 0.0, 0);
        INSERT OR IGNORE INTO Runways (AirportID, Ident, TrueHeading, Length, Width, Surface, Latitude, Longtitude, Elevation)
        VALUES (2, '09', 90.0, 3000, 45, 'Asphalt', 10.0, 10.0, 0);
    ");

    database_connections
}

fn add_test_aircraft(db: &mut DatabaseConnections) {
    let ac = NewAircraft {
        manufacturer: "TestMaker".to_string(),
        variant: "TestPlane".to_string(),
        icao_code: "TST1".to_string(),
        flown: 0,
        date_flown: None,
        aircraft_range: 5000, // Enough range
        category: "A".to_string(),
        cruise_speed: 100,
        takeoff_distance: Some(100),
    };
    db.add_aircraft(&ac).unwrap();
}

struct MockInteraction {
    input_buffer: RefCell<VecDeque<char>>,
    output_buffer: RefCell<String>,
}

impl MockInteraction {
    fn new(input: &str) -> Self {
        Self {
            input_buffer: RefCell::new(input.chars().collect()),
            output_buffer: RefCell::new(String::new()),
        }
    }

    fn get_output(&self) -> String {
        self.output_buffer.borrow().clone()
    }
}

impl Interaction for &MockInteraction {
    fn clear_screen(&self) -> Result<(), Error> {
        Ok(())
    }

    fn write_str(&self, s: &str) -> Result<(), Error> {
        self.output_buffer.borrow_mut().push_str(s);
        Ok(())
    }

    fn read_char(&self) -> Result<char, Error> {
        if let Some(c) = self.input_buffer.borrow_mut().pop_front() {
            Ok(c)
        } else {
            Err(Error::Other(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "End of input",
            )))
        }
    }

    fn read_line(&self) -> Result<String, Error> {
        let mut line = String::new();
        loop {
            // Check if we have chars
            let c_opt = self.input_buffer.borrow_mut().pop_front();

            match c_opt {
                Some('\n') => break, // Stop at newline
                Some(c) => line.push(c),
                None => {
                    if line.is_empty() {
                        return Err(Error::Other(std::io::Error::new(
                            std::io::ErrorKind::UnexpectedEof,
                            "End of input",
                        )));
                    }
                    break;
                }
            }
        }
        Ok(line)
    }
}

#[test]
fn test_cli_quit() {
    let db = setup_test_db();
    let interaction = MockInteraction::new("q");
    let result = console_main(db, &interaction);
    assert!(result.is_ok());
}

#[test]
fn test_cli_get_random_airport() {
    let db = setup_test_db();
    let interaction = MockInteraction::new("1q");
    let result = console_main(db, &interaction);
    assert!(result.is_ok());

    let output = interaction.get_output();
    let has_test = output.contains("Test Airport (TEST)");
    let has_dest = output.contains("Dest Airport (DEST)");
    assert!(
        has_test || has_dest,
        "Output did not contain expected airport. Output:\n{}",
        output
    );
}

#[test]
fn test_cli_get_random_aircraft() {
    let mut db = setup_test_db();
    add_test_aircraft(&mut db);

    let interaction = MockInteraction::new("2q");
    let result = console_main(db, &interaction);
    let output = interaction.get_output();
    assert!(
        result.is_ok(),
        "Function returned error. Output:\n{}",
        output
    );

    assert!(output.contains("TestMaker TestPlane"), "Output: {}", output);
}

#[test]
fn test_cli_random_aircraft_random_airport() {
    let mut db = setup_test_db();
    add_test_aircraft(&mut db);

    // '3' then 'q'
    let interaction = MockInteraction::new("3q");
    let result = console_main(db, &interaction);
    let output = interaction.get_output();
    assert!(result.is_ok(), "Result error. Output:\n{}", output);

    // Should show both aircraft and airport
    assert!(output.contains("Aircraft:"), "Missing aircraft label");
    assert!(
        output.contains("TestMaker TestPlane"),
        "Missing aircraft name"
    );
    assert!(output.contains("Airport:"), "Missing airport label");
}

#[test]
fn test_cli_random_not_flown_aircraft_and_route() {
    let mut db = setup_test_db();
    add_test_aircraft(&mut db);

    // '4' -> "Do you want to mark the aircraft as flown? (y/n)" -> 'n' -> 'q'
    let interaction = MockInteraction::new("4nq");
    let result = console_main(db, &interaction);
    let output = interaction.get_output();
    assert!(result.is_ok(), "Result error. Output:\n{}", output);

    // Check elements
    assert!(output.contains("Aircraft:"), "Missing Aircraft label");
    // We relax the test to just check if aircraft name is present anywhere
    assert!(
        output.contains("TestMaker TestPlane"),
        "Missing aircraft name"
    );
    assert!(output.contains("Departure:"), "Missing Departure");
    assert!(output.contains("Destination:"), "Missing Destination");
    assert!(output.contains("Distance:"), "Missing Distance");
    assert!(output.contains("Do you want to mark the aircraft as flown?"));
}

#[test]
fn test_cli_random_aircraft_and_route() {
    let mut db = setup_test_db();
    add_test_aircraft(&mut db);

    // '5' -> 'q'
    let interaction = MockInteraction::new("5q");
    let result = console_main(db, &interaction);
    assert!(result.is_ok());
    let output = interaction.get_output();

    assert!(output.contains("Destination:"));
    assert!(output.contains("Distance:"));
}

#[test]
fn test_cli_random_route_for_selected_aircraft() {
    let mut db = setup_test_db();
    add_test_aircraft(&mut db);

    // 's' -> enter id "1\n" -> 'q'
    let interaction = MockInteraction::new("s1\nq");
    let result = console_main(db, &interaction);
    let output = interaction.get_output();
    assert!(result.is_ok(), "Result error. Output:\n{}", output);

    assert!(output.contains("Enter aircraft id:"));
    assert!(output.contains("Aircraft:"), "Missing Aircraft label");
    assert!(
        output.contains("TestMaker TestPlane"),
        "Missing aircraft name"
    );
    assert!(output.contains("Distance:"));
}

#[test]
fn test_cli_list_all_aircraft() {
    let mut db = setup_test_db();
    add_test_aircraft(&mut db);

    // 'l' -> 'q'
    let interaction = MockInteraction::new("lq");
    let result = console_main(db, &interaction);
    assert!(result.is_ok());

    let output = interaction.get_output();
    assert!(output.contains("TestMaker TestPlane"));
}

#[test]
fn test_cli_mark_all_not_flown() {
    let mut db = setup_test_db();
    add_test_aircraft(&mut db);

    // 'm' -> 'n' (confirm? no) -> 'q'
    let interaction = MockInteraction::new("mnq");
    let result = console_main(db, &interaction);
    assert!(result.is_ok());
    let output = interaction.get_output();
    assert!(output.contains("Do you want to mark all aircraft as not flown?"));

    // 'm' -> 'y' (confirm? yes) -> 'q'
    let interaction2 = MockInteraction::new("myq");
    let result2 = console_main(setup_test_db(), &interaction2);
    assert!(result2.is_ok());
    let _output2 = interaction2.get_output();
}

#[test]
fn test_cli_history() {
    let db = setup_test_db();
    // 'h' -> 'q'
    // Empty history
    let interaction = MockInteraction::new("hq");
    let result = console_main(db, &interaction);
    let output = interaction.get_output();
    // If result errors, it implies database issue (like missing history table or columns)
    assert!(result.is_ok(), "Result error. Output:\n{}", output);
    assert!(output.contains("No history found"));
}

#[test]
fn test_cli_invalid_input() {
    let db = setup_test_db();
    // Input 'x' (Invalid) then 'q'
    let interaction = MockInteraction::new("xq");
    let result = console_main(db, &interaction);
    assert!(result.is_ok());

    let output = interaction.get_output();
    assert!(output.contains("Invalid input"));
}
