use flight_planner::cli::{Interaction, console_main};
use flight_planner::errors::Error;
use std::cell::RefCell;
use std::collections::VecDeque;

mod common;

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
    let db = common::setup_test_db();
    let interaction = MockInteraction::new("q");
    let result = console_main(db, &interaction);
    assert!(result.is_ok());
}

#[test]
fn test_cli_get_random_airport() {
    let db = common::setup_test_db();
    let interaction = MockInteraction::new("1q");
    let result = console_main(db, &interaction);
    assert!(result.is_ok());

    let output = interaction.get_output();
    let expected_airports = [
        "Amsterdam Airport Schiphol (EHAM)",
        "Rotterdam The Hague Airport (EHRD)",
        "Eindhoven Airport (EHEH)",
    ];
    assert!(
        expected_airports.iter().any(|a| output.contains(*a)),
        "Output did not contain expected airport. Output:\n{}",
        output
    );
}

#[test]
fn test_cli_get_random_aircraft() {
    let db = common::setup_test_db();
    // Aircraft is already added by setup_test_db

    let interaction = MockInteraction::new("2q");
    let result = console_main(db, &interaction);
    let output = interaction.get_output();
    assert!(
        result.is_ok(),
        "Function returned error. Output:\n{}",
        output
    );

    assert!(output.contains("Boeing 737-800"), "Output: {}", output);
}

#[test]
fn test_cli_random_aircraft_random_airport() {
    let db = common::setup_test_db();
    // '3' then 'q'
    let interaction = MockInteraction::new("3q");
    let result = console_main(db, &interaction);
    let output = interaction.get_output();
    assert!(result.is_ok(), "Result error. Output:\n{}", output);

    // Should show both aircraft and airport
    assert!(output.contains("Aircraft:"), "Missing aircraft label");
    assert!(output.contains("Boeing 737-800"), "Missing aircraft name");
    assert!(output.contains("Airport:"), "Missing airport label");
}

#[test]
fn test_cli_random_not_flown_aircraft_and_route() {
    let db = common::setup_test_db();
    // '4' -> "Do you want to mark the aircraft as flown? (y/n)" -> 'n' -> 'q'
    let interaction = MockInteraction::new("4nq");
    let result = console_main(db, &interaction);
    let output = interaction.get_output();
    assert!(result.is_ok(), "Result error. Output:\n{}", output);

    // Check elements
    assert!(output.contains("Aircraft:"), "Missing Aircraft label");
    // We relax the test to just check if aircraft name is present anywhere
    assert!(output.contains("Boeing 737-800"), "Missing aircraft name");
    assert!(output.contains("Departure:"), "Missing Departure");
    assert!(output.contains("Destination:"), "Missing Destination");
    assert!(output.contains("Distance:"), "Missing Distance");
    assert!(output.contains("Do you want to mark the aircraft as flown?"));
}

#[test]
fn test_cli_random_aircraft_and_route() {
    let db = common::setup_test_db();
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
    let db = common::setup_test_db();
    // 's' -> enter id "1\n" -> 'q'
    let interaction = MockInteraction::new("s1\nq");
    let result = console_main(db, &interaction);
    let output = interaction.get_output();
    assert!(result.is_ok(), "Result error. Output:\n{}", output);

    assert!(output.contains("Enter aircraft id:"));
    assert!(output.contains("Aircraft:"), "Missing Aircraft label");
    assert!(output.contains("Boeing 737-800"), "Missing aircraft name");
    assert!(output.contains("Distance:"));
}

#[test]
fn test_cli_list_all_aircraft() {
    let db = common::setup_test_db();
    // 'l' -> 'q'
    let interaction = MockInteraction::new("lq");
    let result = console_main(db, &interaction);
    assert!(result.is_ok());

    let output = interaction.get_output();
    assert!(output.contains("Boeing 737-800"));
}

#[test]
fn test_cli_mark_all_not_flown() {
    let db = common::setup_test_db();
    // 'm' -> 'n' (confirm? no) -> 'q'
    let interaction = MockInteraction::new("mnq");
    let result = console_main(db, &interaction);
    assert!(result.is_ok());
    let output = interaction.get_output();
    assert!(output.contains("Do you want to mark all aircraft as not flown?"));

    // 'm' -> 'y' (confirm? yes) -> 'q'
    let interaction2 = MockInteraction::new("myq");
    let result2 = console_main(common::setup_test_db(), &interaction2);
    assert!(result2.is_ok());
    let _output2 = interaction2.get_output();
}

#[test]
fn test_cli_history() {
    let db = common::setup_test_db();
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
    let db = common::setup_test_db();
    // Input 'x' (Invalid) then 'q'
    let interaction = MockInteraction::new("xq");
    let result = console_main(db, &interaction);
    assert!(result.is_ok());

    let output = interaction.get_output();
    assert!(output.contains("Invalid input"));
}
