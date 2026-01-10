use crate::console_utils::{ask_mark_flown, read_id, read_yn};
use crate::errors::Error;
use crate::modules::aircraft::format_aircraft;
use crate::modules::airport::format_airport;
use crate::modules::runway::format_runway;
use crate::traits::{AircraftOperations, AirportOperations, DatabaseOperations, HistoryOperations};
use crate::util::calculate_haversine_distance_nm;

/// Trait to abstract console interactions for testing purposes.
pub trait Interaction {
    fn clear_screen(&self) -> Result<(), Error>;
    fn write_str(&self, s: &str) -> Result<(), Error>;
    fn read_char(&self) -> Result<char, Error>;
    fn read_line(&self) -> Result<String, Error>;
}

/// Real implementation using the `console` crate.
pub struct ConsoleInteraction {
    term: console::Term,
}

impl ConsoleInteraction {
    pub fn new() -> Self {
        Self {
            term: console::Term::stdout(),
        }
    }
}

impl Default for ConsoleInteraction {
    fn default() -> Self {
        Self::new()
    }
}

impl Interaction for ConsoleInteraction {
    fn clear_screen(&self) -> Result<(), Error> {
        self.term.clear_screen()?;
        Ok(())
    }

    fn write_str(&self, s: &str) -> Result<(), Error> {
        self.term.write_str(s)?;
        Ok(())
    }

    fn read_char(&self) -> Result<char, Error> {
        self.term.read_char().map_err(Error::from)
    }

    fn read_line(&self) -> Result<String, Error> {
        self.term.read_line().map_err(Error::from)
    }
}

/// The main function for the command-line interface (CLI).
///
/// This function initializes the console, displays a menu of options, and enters
/// a loop to process user input. It handles various operations such as
/// displaying random airports and aircraft, generating routes, and managing
/// flight history.
///
/// # Arguments
///
/// * `database_connections` - A mutable instance of a type that implements
///   `DatabaseOperations`, providing access to the application's data.
/// * `interaction` - An implementation of the `Interaction` trait for handling I/O.
///
/// # Returns
///
/// A `Result` indicating success or an `Error` if a critical failure occurs.
pub fn console_main<T: DatabaseOperations, I: Interaction>(
    mut database_connections: T,
    interaction: I,
) -> Result<(), Error> {
    interaction.clear_screen()?;

    loop {
        let not_flown_aircraft_count = database_connections.get_not_flown_count()?;

        interaction.write_str(&format!(
            "\nWelcome to the flight planner\n\
             --------------------------------------------------\n\
             Number of not flown aircraft: {not_flown_aircraft_count}\n\
             What do you want to do?\n\
             1. Get random airport\n\
             2. Get random aircraft\n\
             3. Random aircraft from random airport\n\
             4. Random not flown aircraft, airport and destination\n\
             5. random aircraft and route\n\
             s, Random route for selected aircraft\n\
             l. List all aircraft\n\
             m. Mark all aircraft as not flown
             h. History\n\
             q. Quit\n"
        ))?;

        interaction.write_str("Enter your choice: ")?;
        let input = match interaction.read_char() {
            Ok(c) => c,
            Err(e) => {
                log::error!("Failed to read input: {e}");
                continue;
            }
        };
        interaction.clear_screen()?;

        match input {
            '1' => show_random_airport(&mut database_connections, &interaction)?,
            '2' => show_random_not_flown_aircraft(&mut database_connections, &interaction)?,
            '3' => {
                show_random_aircraft_with_random_airport(&mut database_connections, &interaction)?
            }
            '4' => {
                show_random_not_flown_aircraft_and_route(&mut database_connections, &interaction)?
            }
            '5' => show_random_aircraft_and_route(&mut database_connections, &interaction)?,
            's' => {
                show_random_route_for_selected_aircraft(&mut database_connections, &interaction)?
            }
            'l' => show_all_aircraft(&mut database_connections, &interaction)?,
            'm' => show_mark_all_not_flown(&mut database_connections, &interaction)?,
            'h' => show_history(&mut database_connections, &interaction)?,
            'q' => {
                log::info!("Quitting");
                return Ok(());
            }
            _ => {
                interaction.write_str("Invalid input\n")?;
            }
        }
    }
}

fn show_mark_all_not_flown<T: DatabaseOperations, I: Interaction>(
    database_connections: &mut T,
    interaction: &I,
) -> Result<(), Error> {
    let ask_confirm = || -> std::io::Result<char> {
        interaction
            .write_str("Do you want to mark all aircraft as not flown? (y/n)\n")
            .map_err(std::io::Error::other)?;
        interaction.read_char().map_err(std::io::Error::other)
    };

    mark_all_not_flown(database_connections, ask_confirm)
}

fn mark_all_not_flown<T: AircraftOperations, F: Fn() -> Result<char, std::io::Error>>(
    database_connections: &mut T,
    confirm_fn: F,
) -> Result<(), Error> {
    match read_yn(confirm_fn) {
        Ok(true) => {
            database_connections.mark_all_aircraft_not_flown()?;
        }
        Ok(false) => {
            log::info!("Mark all aircraft as not flown action cancelled.");
        }
        Err(e) => {
            log::error!("Failed to read input: {e}");
        }
    }

    Ok(())
}

fn show_random_airport<T: AirportOperations, I: Interaction>(
    database_connections: &mut T,
    interaction: &I,
) -> Result<(), Error> {
    let airport = database_connections.get_random_airport()?;
    interaction.write_str(&format!("{}\n", format_airport(&airport)))?;

    let runways = database_connections.get_runways_for_airport(&airport)?;
    for runway in runways {
        interaction.write_str(&format!("{}\n", format_runway(&runway)))?;
    }

    Ok(())
}

fn show_random_not_flown_aircraft<T: AircraftOperations, I: Interaction>(
    database_connections: &mut T,
    interaction: &I,
) -> Result<(), Error> {
    match database_connections.random_not_flown_aircraft() {
        Ok(aircraft) => {
            interaction.write_str(&format!("{}\n", format_aircraft(&aircraft)))?;
            Ok(())
        }
        Err(e) => {
            log::error!("Failed to get random not flown aircraft: {e}");
            Ok(())
        }
    }
}

fn show_random_aircraft_with_random_airport<T: DatabaseOperations, I: Interaction>(
    database_connections: &mut T,
    interaction: &I,
) -> Result<(), Error> {
    let aircraft = database_connections.random_not_flown_aircraft()?;
    let airport = database_connections.get_random_airport_for_aircraft(&aircraft)?;

    interaction.write_str(&format!("Aircraft: {}\n", format_aircraft(&aircraft)))?;
    interaction.write_str(&format!("Airport: {}\n", format_airport(&airport)))?;

    for runway in database_connections.get_runways_for_airport(&airport)? {
        interaction.write_str(&format!("{}\n", format_runway(&runway)))?;
    }

    Ok(())
}

fn show_random_aircraft_and_route<T: DatabaseOperations, I: Interaction>(
    database_connections: &mut T,
    interaction: &I,
) -> Result<(), Error> {
    let aircraft = database_connections.random_aircraft()?;
    let departure = database_connections.get_random_airport_for_aircraft(&aircraft)?;
    let destination = database_connections.get_destination_airport(&aircraft, &departure)?;
    let distance = calculate_haversine_distance_nm(&departure, &destination);

    interaction.write_str(&format!("Aircraft: {}\n", format_aircraft(&aircraft)))?;
    interaction.write_str(&format!("Departure: {}\n", format_airport(&departure)))?;
    interaction.write_str(&format!("Destination: {}\n", format_airport(&destination)))?;
    interaction.write_str(&format!("Distance: {distance:.2}nm\n"))?;

    interaction.write_str("\nDeparture runways:\n")?;
    for runway in database_connections.get_runways_for_airport(&departure)? {
        interaction.write_str(&format!("{}\n", format_runway(&runway)))?;
    }

    interaction.write_str("\nDestination runways:\n")?;
    for runway in database_connections.get_runways_for_airport(&destination)? {
        interaction.write_str(&format!("{}\n", format_runway(&runway)))?;
    }

    Ok(())
}

fn show_all_aircraft<T: AircraftOperations, I: Interaction>(
    database_connections: &mut T,
    interaction: &I,
) -> Result<(), Error> {
    let all_aircraft = database_connections.get_all_aircraft()?;
    for aircraft in all_aircraft {
        interaction.write_str(&format!("{}\n", format_aircraft(&aircraft)))?;
    }

    Ok(())
}

fn show_random_not_flown_aircraft_and_route<T: DatabaseOperations, I: Interaction>(
    database_connections: &mut T,
    interaction: &I,
) -> Result<(), Error> {
    let ask_char_fn = || -> Result<char, std::io::Error> {
        interaction
            .write_str("Do you want to mark the aircraft as flown? (y/n)\n")
            .map_err(std::io::Error::other)?;
        interaction.read_char().map_err(std::io::Error::other)
    };

    random_not_flown_aircraft_and_route(database_connections, ask_char_fn, interaction)
}

fn random_not_flown_aircraft_and_route<
    T: DatabaseOperations,
    F: Fn() -> Result<char, std::io::Error>,
    I: Interaction,
>(
    database_connections: &mut T,
    ask_char_fn: F,
    interaction: &I,
) -> Result<(), Error> {
    let mut aircraft = database_connections.random_not_flown_aircraft()?;
    let departure = database_connections.get_random_airport_for_aircraft(&aircraft)?;
    let destination = database_connections.get_destination_airport(&aircraft, &departure)?;
    let distance = calculate_haversine_distance_nm(&departure, &destination);

    interaction.write_str(&format!("Aircraft: {}\n", format_aircraft(&aircraft)))?;
    interaction.write_str(&format!("Departure: {}\n", format_airport(&departure)))?;
    interaction.write_str(&format!("Destination: {}\n", format_airport(&destination)))?;
    interaction.write_str(&format!("Distance: {distance:.2}nm\n"))?;

    interaction.write_str("\nDeparture runways:\n")?;
    for runway in database_connections.get_runways_for_airport(&departure)? {
        interaction.write_str(&format!("{}\n", format_runway(&runway)))?;
    }

    interaction.write_str("\nDestination runways:\n")?;
    for runway in database_connections.get_runways_for_airport(&destination)? {
        interaction.write_str(&format!("{}\n", format_runway(&runway)))?;
    }

    ask_mark_flown(database_connections, &mut aircraft, ask_char_fn)?;
    database_connections.add_to_history(&departure, &destination, &aircraft)?;

    Ok(())
}

fn show_history<T: HistoryOperations + AircraftOperations, I: Interaction>(
    database_connections: &mut T,
    interaction: &I,
) -> Result<(), Error> {
    let history_data = database_connections.get_history()?;
    let aircraft_data = database_connections.get_all_aircraft()?;

    if history_data.is_empty() {
        interaction.write_str("No history found\n")?;
        return Ok(());
    }

    for record in history_data {
        let Some(aircraft) = aircraft_data.iter().find(|a| a.id == record.aircraft) else {
            log::warn!("Aircraft not found for id: {}", record.aircraft);
            return Err(Error::Diesel(diesel::result::Error::NotFound));
        };

        interaction.write_str(&format!(
            "Date: {}\nDeparture: {}\nDestination: {}\nAircraft: {} {} ({})\n\n",
            record.date,
            record.departure_icao,
            record.arrival_icao,
            aircraft.manufacturer,
            aircraft.variant,
            aircraft.icao_code
        ))?;
    }

    Ok(())
}

fn show_random_route_for_selected_aircraft<T: DatabaseOperations, I: Interaction>(
    database_connections: &mut T,
    interaction: &I,
) -> Result<(), Error> {
    let ask_input_id = || -> Result<String, std::io::Error> {
        interaction
            .write_str("Enter aircraft id: ")
            .map_err(std::io::Error::other)?;
        interaction.read_line().map_err(std::io::Error::other)
    };

    random_route_for_selected_aircraft(database_connections, ask_input_id, interaction)
}

fn random_route_for_selected_aircraft<
    T: DatabaseOperations,
    F: Fn() -> Result<String, std::io::Error>,
    I: Interaction,
>(
    database_connections: &mut T,
    aircraft_id_fn: F,
    interaction: &I,
) -> Result<(), Error> {
    let aircraft_id = match read_id(aircraft_id_fn) {
        Ok(id) => id,
        Err(e) => {
            log::warn!("Invalid id: {e}");
            return Ok(());
        }
    };

    let aircraft = database_connections.get_aircraft_by_id(aircraft_id)?;
    let departure = database_connections.get_random_airport_for_aircraft(&aircraft)?;
    let destination = database_connections.get_destination_airport(&aircraft, &departure)?;
    let distance = calculate_haversine_distance_nm(&departure, &destination);

    interaction.write_str(&format!("Aircraft: {}\n", format_aircraft(&aircraft)))?;
    interaction.write_str(&format!("Departure: {}\n", format_airport(&departure)))?;
    interaction.write_str(&format!("Destination: {}\n", format_airport(&destination)))?;
    interaction.write_str(&format!("Distance: {distance:.2}nm\n"))?;

    interaction.write_str("\nDeparture runways:\n")?;
    for runway in database_connections.get_runways_for_airport(&departure)? {
        interaction.write_str(&format!("{}\n", format_runway(&runway)))?;
    }

    interaction.write_str("\nDestination runways:\n")?;
    for runway in database_connections.get_runways_for_airport(&destination)? {
        interaction.write_str(&format!("{}\n", format_runway(&runway)))?;
    }

    Ok(())
}
