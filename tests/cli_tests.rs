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
fn test_cli_scenarios() {
    struct CliTestCase<'a> {
        name: &'a str,
        input: &'a str,
        expected_contains: Vec<&'a str>,
    }

    let cases = vec![
        CliTestCase {
            name: "Quit",
            input: "q",
            expected_contains: vec![],
        },
        CliTestCase {
            name: "Get random aircraft",
            input: "2q",
            expected_contains: vec!["Boeing 737-800"],
        },
        CliTestCase {
            name: "Random aircraft and random airport",
            input: "3q",
            expected_contains: vec!["Aircraft:", "Boeing 737-800", "Airport:"],
        },
        CliTestCase {
            name: "Random not flown aircraft and route",
            input: "4nq",
            expected_contains: vec![
                "Aircraft:",
                "Boeing 737-800",
                "Departure:",
                "Destination:",
                "Distance:",
                "Do you want to mark the aircraft as flown?",
            ],
        },
        CliTestCase {
            name: "Random aircraft and route",
            input: "5q",
            expected_contains: vec!["Destination:", "Distance:"],
        },
        CliTestCase {
            name: "Random route for selected aircraft",
            input: "s1\nq",
            expected_contains: vec![
                "Enter aircraft id:",
                "Aircraft:",
                "Boeing 737-800",
                "Distance:",
            ],
        },
        CliTestCase {
            name: "List all aircraft",
            input: "lq",
            expected_contains: vec!["Boeing 737-800"],
        },
        CliTestCase {
            name: "Mark all not flown (No)",
            input: "mnq",
            expected_contains: vec!["Do you want to mark all aircraft as not flown?"],
        },
        CliTestCase {
            name: "Mark all not flown (Yes)",
            input: "myq",
            expected_contains: vec!["Do you want to mark all aircraft as not flown?"],
        },
        CliTestCase {
            name: "History",
            input: "hq",
            expected_contains: vec!["No history found"],
        },
        CliTestCase {
            name: "Invalid input",
            input: "xq",
            expected_contains: vec!["Invalid input"],
        },
    ];

    for case in cases {
        let db = common::setup_test_db();
        let interaction = MockInteraction::new(case.input);
        let result = console_main(db, &interaction);
        assert!(result.is_ok(), "Failed case: {}", case.name);

        let output = interaction.get_output();
        for expected in case.expected_contains {
            assert!(
                output.contains(expected),
                "Failed case: {}. Output missing '{}'. Output: {}",
                case.name,
                expected,
                output
            );
        }
    }
}
