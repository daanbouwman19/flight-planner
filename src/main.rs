use std::path;

use diesel::prelude::*;
use diesel::result::Error;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

mod errors;
mod models;
mod schema;
mod traits;
pub mod modules {
    pub mod aircraft;
    pub mod airport;
    pub mod history;
    pub mod runway;
}
mod gui;

#[cfg(test)]
mod test {
    mod database_connection_stub;
    mod test;
}

use eframe::AppCreator;
use gui::Gui;

use crate::models::Aircraft;
use errors::ValidationError;
use modules::aircraft::*;
use modules::airport::*;
use modules::runway::*;
use traits::*;

define_sql_function! {fn random() -> Text }

const AIRCRAFT_DB_FILENAME: &str = "data.db";
const AIRPORT_DB_FILENAME: &str = "airports.db3";

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub struct DatabaseConnections {
    aircraft_connection: SqliteConnection,
    airport_connection: SqliteConnection,
}

impl Default for DatabaseConnections {
    fn default() -> Self {
        Self::new()
    }
}

impl DatabaseOperations for DatabaseConnections {}

impl DatabaseConnections {
    pub fn new() -> Self {
        fn establish_database_connection(database_name: &str) -> SqliteConnection {
            SqliteConnection::establish(database_name).unwrap_or_else(|_| {
                panic!("Error connecting to {}", database_name);
            })
        }

        let aircraft_connection = establish_database_connection(AIRCRAFT_DB_FILENAME);
        let airport_connection = establish_database_connection(AIRPORT_DB_FILENAME);

        DatabaseConnections {
            aircraft_connection,
            airport_connection,
        }
    }
}

fn main() {
    env_logger::init();

    if !path::Path::new(AIRPORT_DB_FILENAME).exists() {
        log::error!("Airports database not found at {}", AIRPORT_DB_FILENAME);
        return;
    }

    if let Err(e) = run() {
        log::error!("Application error: {}", e);
    }
}

fn run() -> Result<(), Error> {
    let mut database_connections = DatabaseConnections::new();

    database_connections
        .aircraft_connection
        .run_pending_migrations(MIGRATIONS)
        .expect("Failed to run migrations");

    let native_options = eframe::NativeOptions::default();
    let app_creator: AppCreator<'_> = Box::new(|cc| {
        Ok(Box::new(Gui::new(cc, &mut database_connections)))
    });
    _ = eframe::run_native("Flight planner", native_options, app_creator);

    
    // console_main(&mut database_connections)?;
    Ok(())
}

#[allow(dead_code)]
fn console_main<T: DatabaseOperations>(database_connections: &mut T) -> Result<(), Error> {
    let terminal = console::Term::stdout();
    terminal.clear_screen().unwrap();

    loop {
        let unflown_aircraft_count = database_connections.get_unflown_aircraft_count()?;

        println!(
            "\nWelcome to the flight planner\n\
             --------------------------------------------------\n\
             Number of unflown aircraft: {}\n\
             What do you want to do?\n\
             1. Get random airport\n\
             2. Get random aircraft\n\
             3. Random aircraft from random airport\n\
             4. Random unflown aircraft, airport and destination\n\
             5. random aircraft and route\n\
             s, Random route for selected aircraft\n\
             l. List all aircraft\n\
             h. History\n\
             q. Quit\n",
            unflown_aircraft_count
        );

        terminal.write_str("Enter your choice: ").unwrap();
        let input = match terminal.read_char() {
            Ok(c) => c,
            Err(e) => {
                log::error!("Failed to read input: {}", e);
                continue;
            }
        };
        terminal.clear_screen().unwrap();

        match input {
            '1' => show_random_airport(database_connections)?,
            '2' => show_random_unflown_aircraft(database_connections)?,
            '3' => show_random_aircraft_with_random_airport(database_connections)?,
            '4' => show_random_unflown_aircraft_and_route(database_connections)?,
            '5' => show_random_aircraft_and_route(database_connections)?,
            's' => show_random_route_for_selected_aircraft(database_connections)?,
            'l' => show_all_aircraft(database_connections)?,
            'h' => show_history(database_connections)?,
            'q' => {
                log::info!("Quitting");
                return Ok(());
            }
            _ => {
                println!("Invalid input");
            }
        }
    }
}

fn show_random_airport<T: AirportOperations>(database_connections: &mut T) -> Result<(), Error> {
    let airport = database_connections.get_random_airport()?;
    println!("{}", format_airport(&airport));

    let runways = database_connections.get_runways_for_airport(&airport)?;
    for runway in runways {
        println!("{}", format_runway(&runway));
    }

    Ok(())
}

fn show_random_unflown_aircraft<T: AircraftOperations>(
    database_connections: &mut T,
) -> Result<(), Error> {
    let aircraft = database_connections.random_unflown_aircraft()?;
    println!("{}", format_aircraft(&aircraft));

    Ok(())
}

fn show_random_aircraft_with_random_airport<T: DatabaseOperations>(
    database_connections: &mut T,
) -> Result<(), Error> {
    let aircraft = database_connections.random_unflown_aircraft()?;
    let airport = database_connections.get_random_airport_for_aircraft(&aircraft)?;

    println!("Aircraft: {}", format_aircraft(&aircraft));
    println!("Airport: {}", format_airport(&airport));

    for runway in database_connections.get_runways_for_airport(&airport)? {
        println!("{}", format_runway(&runway));
    }

    Ok(())
}

fn show_random_aircraft_and_route<T: DatabaseOperations>(
    database_connections: &mut T,
) -> Result<(), Error> {
    let aircraft = database_connections.random_aircraft()?;
    let departure = database_connections.get_random_airport_for_aircraft(&aircraft)?;
    let destination = database_connections.get_destination_airport(&aircraft, &departure)?;
    let distance = haversine_distance_nm(&departure, &destination);

    println!("Aircraft: {}", format_aircraft(&aircraft));
    println!("Departure: {}", format_airport(&departure));
    println!("Destination: {}", format_airport(&destination));
    println!("Distance: {:.2}nm", distance);

    println!("\nDeparture runways:");
    for runway in database_connections.get_runways_for_airport(&departure)? {
        println!("{}", format_runway(&runway));
    }

    println!("\nDestination runways:");
    for runway in database_connections.get_runways_for_airport(&destination)? {
        println!("{}", format_runway(&runway));
    }

    Ok(())
}

fn show_all_aircraft<T: AircraftOperations>(database_connections: &mut T) -> Result<(), Error> {
    let aircrafts = database_connections.get_all_aircraft()?;
    for aircraft in aircrafts {
        println!("{}", format_aircraft(&aircraft));
    }

    Ok(())
}

fn show_random_unflown_aircraft_and_route<T: DatabaseOperations>(
    database_connections: &mut T,
) -> Result<(), Error> {
    let ask_char_fn = || -> Result<char, std::io::Error> {
        let term = console::Term::stdout();
        term.write_str("Do you want to mark the aircraft as flown? (y/n)\n")
            .unwrap();
        match term.read_char() {
            Ok(c) => Ok(c),
            Err(e) => Err(e),
        }
    };

    random_unflown_aircraft_and_route(database_connections, ask_char_fn)
}

fn ask_mark_flown<T: AircraftOperations, F: Fn() -> Result<char, std::io::Error>>(
    database_connections: &mut T,
    aircraft: &mut Aircraft,
    ask_char_fn: F,
) -> Result<(), Error> {
    if let Ok('y') = ask_char_fn() {
        aircraft.date_flown = Some(chrono::Local::now().format("%Y-%m-%d").to_string());
        aircraft.flown = 1;
        database_connections.update_aircraft(aircraft)?;
    }

    Ok(())
}

fn random_unflown_aircraft_and_route<
    T: DatabaseOperations,
    F: Fn() -> Result<char, std::io::Error>,
>(
    database_connections: &mut T,
    ask_char_fn: F,
) -> Result<(), Error> {
    let mut aircraft = database_connections.random_unflown_aircraft()?;
    let departure = database_connections.get_random_airport_for_aircraft(&aircraft)?;
    let destination = database_connections.get_destination_airport(&aircraft, &departure)?;
    let distance = haversine_distance_nm(&departure, &destination);

    println!("Aircraft: {}", format_aircraft(&aircraft));
    println!("Departure: {}", format_airport(&departure));
    println!("Destination: {}", format_airport(&destination));
    println!("Distance: {:.2}nm", distance);

    println!("\nDeparture runways:");
    for runway in database_connections.get_runways_for_airport(&departure)? {
        println!("{}", format_runway(&runway));
    }

    println!("\nDestination runways:");
    for runway in database_connections.get_runways_for_airport(&destination)? {
        println!("{}", format_runway(&runway));
    }

    ask_mark_flown(database_connections, &mut aircraft, ask_char_fn)?;
    database_connections.add_to_history(&departure, &destination, &aircraft)?;

    Ok(())
}

fn show_history<T: HistoryOperations + AircraftOperations>(
    database_connections: &mut T,
) -> Result<(), Error> {
    let history = database_connections.get_history()?;
    let aircrafts = database_connections.get_all_aircraft()?;

    if history.is_empty() {
        println!("No history found");
        return Ok(());
    }

    for record in history {
        let aircraft = match aircrafts.iter().find(|a| a.id == record.aircraft) {
            Some(a) => a,
            None => {
                log::warn!("Aircraft not found for id: {}", record.aircraft);
                return Err(Error::NotFound);
            }
        };

        println!(
            "Date: {}\nDeparture: {}\nDestination: {}\nAircraft: {} {} ({})\n",
            record.date,
            record.departure_icao,
            record.arrival_icao,
            aircraft.manufacturer,
            aircraft.variant,
            aircraft.icao_code
        );
    }

    Ok(())
}

fn show_random_route_for_selected_aircraft<T: DatabaseOperations>(
    database_connections: &mut T,
) -> Result<(), Error> {
    let terminal = console::Term::stdout();
    let ask_input_id = || -> Result<String, std::io::Error> {
        terminal.write_str("Enter aircraft id: ")?;
        terminal.read_line()
    };

    random_route_for_selected_aircraft(database_connections, ask_input_id)
}

fn random_route_for_selected_aircraft<
    T: DatabaseOperations,
    F: Fn() -> Result<String, std::io::Error>,
>(
    database_connections: &mut T,
    aircraft_id_fn: F,
) -> Result<(), Error> {
    let aircraft_id = match read_id(aircraft_id_fn) {
        Ok(id) => id,
        Err(e) => {
            log::warn!("Invalid id: {}", e);
            return Ok(());
        }
    };

    let aircraft = database_connections.get_aircraft_by_id(aircraft_id)?;
    let departure = database_connections.get_random_airport_for_aircraft(&aircraft)?;
    let destination = database_connections.get_destination_airport(&aircraft, &departure)?;
    let distance = haversine_distance_nm(&departure, &destination);

    println!("Aircraft: {}", format_aircraft(&aircraft));
    println!("Departure: {}", format_airport(&departure));
    println!("Destination: {}", format_airport(&destination));
    println!("Distance: {:.2}nm", distance);

    println!("\nDeparture runways:");
    for runway in database_connections.get_runways_for_airport(&departure)? {
        println!("{}", format_runway(&runway));
    }

    println!("\nDestination runways:");
    for runway in database_connections.get_runways_for_airport(&destination)? {
        println!("{}", format_runway(&runway));
    }

    Ok(())
}

fn read_id<F: Fn() -> Result<String, std::io::Error>>(
    read_input: F,
) -> Result<i32, ValidationError> {
    let input = read_input().map_err(|e| ValidationError::InvalidData(e.to_string()))?;
    let id = match input.trim().parse::<i32>() {
        Ok(id) => id,
        Err(e) => {
            log::error!("Failed to parse id: {}", e);
            return Err(ValidationError::InvalidData("Invalid id".to_string()));
        }
    };

    if id < 1 {
        return Err(ValidationError::InvalidId(id));
    }

    Ok(id)
}
