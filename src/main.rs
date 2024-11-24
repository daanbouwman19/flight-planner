pub mod modules {
    pub mod aircraft;
    pub mod airport;
    pub mod history;
    pub mod runway;
}
mod errors;
mod models;
mod schema;

#[cfg(test)]
mod test;

use std::path;

use errors::ValidationError;
use modules::aircraft::*;
use modules::airport::*;
use modules::history::*;
use modules::runway::*;

use crate::models::Aircraft;
use diesel::prelude::*;
use diesel::result::Error;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

define_sql_function! {fn random() -> Text }

const AIRCRAFT_DB_FILENAME: &str = "data.db";
const AIRPORT_DB_FILENAME: &str = "airports.db3";

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

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
    let connection_aircraft = &mut establish_database_connection(AIRCRAFT_DB_FILENAME);
    let connection_airport = &mut establish_database_connection(AIRPORT_DB_FILENAME);

    connection_aircraft
        .run_pending_migrations(MIGRATIONS)
        .expect("Failed to run migrations");

    let terminal = console::Term::stdout();
    terminal.clear_screen().unwrap();

    loop {
        let unflown_aircraft_count = get_unflown_aircraft_count(connection_aircraft)?;

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
            '1' => show_random_airport(connection_airport)?,
            '2' => show_random_unflown_aircraft(connection_aircraft)?,
            '3' => {
                show_random_aircraft_with_random_airport(connection_aircraft, connection_airport)?
            }
            '4' => show_random_unflown_aircraft_and_route(connection_aircraft, connection_airport)?,
            '5' => show_random_aircraft_and_route(connection_aircraft, connection_airport)?,
            's' => {
                show_random_route_for_selected_aircraft(connection_aircraft, connection_airport)?
            }
            'l' => show_all_aircraft(connection_aircraft)?,
            'h' => show_history(connection_aircraft)?,
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

fn establish_database_connection(database_name: &str) -> SqliteConnection {
    SqliteConnection::establish(database_name).unwrap_or_else(|_| {
        panic!("Error connecting to {}", database_name);
    })
}

fn show_random_airport(connection: &mut SqliteConnection) -> Result<(), Error> {
    let airport = get_random_airport(connection)?;
    println!("{}", format_airport(&airport));

    let runways = get_runways_for_airport(connection, &airport)?;
    for runway in runways {
        println!("{}", format_runway(&runway));
    }

    Ok(())
}

fn show_random_unflown_aircraft(connection: &mut SqliteConnection) -> Result<(), Error> {
    let aircraft = random_unflown_aircraft(connection)?;
    println!("{}", format_aircraft(&aircraft));

    Ok(())
}

fn show_random_aircraft_with_random_airport(
    aircraft_connection: &mut SqliteConnection,
    airport_connection: &mut SqliteConnection,
) -> Result<(), Error> {
    let aircraft = random_unflown_aircraft(aircraft_connection)?;
    let airport = get_random_airport_for_aircraft(airport_connection, &aircraft)?;

    println!("Aircraft: {}", format_aircraft(&aircraft));
    println!("Airport: {}", format_airport(&airport));

    for runway in get_runways_for_airport(airport_connection, &airport)? {
        println!("{}", format_runway(&runway));
    }

    Ok(())
}

fn show_random_aircraft_and_route(
    aircraft_connection: &mut SqliteConnection,
    airport_connection: &mut SqliteConnection,
) -> Result<(), Error> {
    let aircraft = random_aircraft(aircraft_connection)?;
    let departure = get_random_airport_for_aircraft(airport_connection, &aircraft)?;
    let destination = get_destination_airport(airport_connection, &aircraft, &departure)?;
    let distance = haversine_distance_nm(&departure, &destination);

    println!("Aircraft: {}", format_aircraft(&aircraft));
    println!("Departure: {}", format_airport(&departure));
    println!("Destination: {}", format_airport(&destination));
    println!("Distance: {:.2}nm", distance);

    println!("\nDeparture runways:");
    for runway in get_runways_for_airport(airport_connection, &departure)? {
        println!("{}", format_runway(&runway));
    }

    println!("\nDestination runways:");
    for runway in get_runways_for_airport(airport_connection, &destination)? {
        println!("{}", format_runway(&runway));
    }

    Ok(())
}

fn show_all_aircraft(aircraft_connection: &mut SqliteConnection) -> Result<(), Error> {
    let aircrafts = get_all_aircraft(aircraft_connection)?;
    for aircraft in aircrafts {
        println!("{}", format_aircraft(&aircraft));
    }
    Ok(())
}

fn show_random_unflown_aircraft_and_route(
    aircraft_connection: &mut SqliteConnection,
    airport_connection: &mut SqliteConnection,
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

    random_unflown_aircraft_and_route(aircraft_connection, airport_connection, ask_char_fn)
}

fn ask_mark_flown<F>(
    aircraft_connection: &mut SqliteConnection,
    aircraft: &mut Aircraft,
    ask_char_fn: F,
) -> Result<(), Error>
where
    F: Fn() -> Result<char, std::io::Error>,
{
    match ask_char_fn() {
        Ok('y') => {
            aircraft.date_flown = Some(chrono::Local::now().format("%Y-%m-%d").to_string());
            aircraft.flown = 1;
            update_aircraft(aircraft_connection, aircraft)?;
        }
        _ => {}
    }
    Ok(())
}

fn random_unflown_aircraft_and_route<F>(
    aircraft_connection: &mut SqliteConnection,
    airport_connection: &mut SqliteConnection,
    ask_char_fn: F,
) -> Result<(), Error>
where
    F: Fn() -> Result<char, std::io::Error>,
{
    let mut aircraft = random_unflown_aircraft(aircraft_connection)?;
    let departure = get_random_airport_for_aircraft(airport_connection, &aircraft)?;
    let destination = get_destination_airport(airport_connection, &aircraft, &departure)?;
    let distance = haversine_distance_nm(&departure, &destination);

    println!("Aircraft: {}", format_aircraft(&aircraft));
    println!("Departure: {}", format_airport(&departure));
    println!("Destination: {}", format_airport(&destination));
    println!("Distance: {:.2}nm", distance);

    println!("\nDeparture runways:");
    for runway in get_runways_for_airport(airport_connection, &departure)? {
        println!("{}", format_runway(&runway));
    }

    println!("\nDestination runways:");
    for runway in get_runways_for_airport(airport_connection, &destination)? {
        println!("{}", format_runway(&runway));
    }

    ask_mark_flown(aircraft_connection, &mut aircraft, ask_char_fn)?;
    add_to_history(aircraft_connection, &departure, &destination, &aircraft)?;

    Ok(())
}

fn show_history(connection: &mut SqliteConnection) -> Result<(), Error> {
    let history = get_history(connection)?;
    let aircrafts = get_all_aircraft(connection)?;

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

fn show_random_route_for_selected_aircraft(
    connection_aircraft: &mut SqliteConnection,
    connection_airport: &mut SqliteConnection,
) -> Result<(), Error> {
    let terminal = console::Term::stdout();
    let ask_input_id = || -> Result<String, std::io::Error> {
        terminal.write_str("Enter aircraft id: ")?;
        terminal.read_line()
    };

    random_route_for_selected_aircraft(connection_aircraft, connection_airport, ask_input_id)
}

fn random_route_for_selected_aircraft<F>(
    aircraft_connection: &mut SqliteConnection,
    airport_connection: &mut SqliteConnection,
    aircraft_id_fn: F,
) -> Result<(), Error>
where
    F: Fn() -> Result<String, std::io::Error>,
{
    let aircraft_id = match read_id(aircraft_id_fn) {
        Ok(id) => id,
        Err(e) => {
            log::warn!("Invalid id: {}", e);
            return Ok(());
        }
    };

    let aircraft = get_aircraft_by_id(aircraft_connection, aircraft_id)?;
    let departure = get_random_airport_for_aircraft(airport_connection, &aircraft)?;
    let destination = get_destination_airport(airport_connection, &aircraft, &departure)?;
    let distance = haversine_distance_nm(&departure, &destination);

    println!("Aircraft: {}", format_aircraft(&aircraft));
    println!("Departure: {}", format_airport(&departure));
    println!("Destination: {}", format_airport(&destination));
    println!("Distance: {:.2}nm", distance);

    println!("\nDeparture runways:");
    for runway in get_runways_for_airport(airport_connection, &departure)? {
        println!("{}", format_runway(&runway));
    }

    println!("\nDestination runways:");
    for runway in get_runways_for_airport(airport_connection, &destination)? {
        println!("{}", format_runway(&runway));
    }

    Ok(())
}

fn read_id<F>(read_input: F) -> Result<i32, ValidationError>
where
    F: Fn() -> Result<String, std::io::Error>,
{
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
