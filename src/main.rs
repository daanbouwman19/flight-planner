pub mod modules {
    pub mod aircraft;
    pub mod airport;
    pub mod history;
    pub mod runway;
}
mod models;
mod schema;

#[cfg(test)]
mod test;

use std::path;

use modules::aircraft::*;
use modules::airport::*;
use modules::history::*;
use modules::runway::*;

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
        log::error!("Airports database not found");
        return;
    }

    if let Err(e) = run() {
        log::error!("Error: {}", e);
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
             q. Quit\n\n",
            unflown_aircraft_count
        );

        let input = terminal.read_char().unwrap();
        terminal.clear_screen().unwrap();

        match input {
            '1' => show_random_airport(connection_airport)?,
            '2' => show_random_unflown_aircraft(connection_aircraft)?,
            '3' => {
                show_random_aircraft_with_random_airport(connection_aircraft, connection_airport)?
            }
            '4' => show_random_unflown_aircraft_and_route(connection_aircraft, connection_airport)?,
            '5' => show_random_aircraft_and_route(connection_aircraft, connection_airport)?,
            's' => random_route_for_selected_aircraft(connection_aircraft, connection_airport)?,
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

    let term = console::Term::stdout();
    term.write_str("Do you want to mark the aircraft as flown? (y/n)\n")
        .unwrap();
    let char = term.read_char().unwrap();
    if char == 'y' {
        let now = chrono::Local::now();
        aircraft.date_flown = Some(now.format("%Y-%m-%d").to_string());
        aircraft.flown = 1;
        if let Err(e) = update_aircraft(aircraft_connection, &aircraft) {
            log::error!("Failed to update aircraft: {}", e);
        }
    }

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
        let aircraft = aircrafts
            .iter()
            .find(|a| a.id == record.aircraft)
            .expect("Aircraft not found");

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

fn random_route_for_selected_aircraft(
    aircraft_connection: &mut SqliteConnection,
    airport_connection: &mut SqliteConnection,
) -> Result<(), Error> {
    let term = console::Term::stdout();
    term.write_str("Enter aircraft id: ").unwrap();
    let aircraft_id = term.read_line().unwrap();
    let aircraft_id = aircraft_id.trim().parse::<i32>().unwrap();

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
