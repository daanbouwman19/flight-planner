use crate::console_utils::{ask_mark_flown, read_id, read_yn};
use crate::errors::Error;
use crate::modules::aircraft::format_aircraft;
use crate::modules::airport::format_airport;
use crate::modules::runway::format_runway;
use crate::traits::{AircraftOperations, AirportOperations, DatabaseOperations, HistoryOperations};
use crate::util::calculate_haversine_distance_nm;

pub fn console_main<T: DatabaseOperations>(mut database_connections: T) -> Result<(), Error> {
    let terminal = console::Term::stdout();
    terminal.clear_screen()?;

    loop {
        let not_flown_aircraft_count = database_connections.get_not_flown_count()?;

        println!(
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
             m. Mark aircraft as flown\n\
             h. History\n\
             q. Quit\n"
        );

        terminal.write_str("Enter your choice: ")?;
        let input = match terminal.read_char() {
            Ok(c) => c,
            Err(e) => {
                log::error!("Failed to read input: {e}");
                continue;
            }
        };
        terminal.clear_screen()?;

        match input {
            '1' => show_random_airport(&mut database_connections)?,
            '2' => show_random_not_flown_aircraft(&mut database_connections),
            '3' => show_random_aircraft_with_random_airport(&mut database_connections)?,
            '4' => show_random_not_flown_aircraft_and_route(&mut database_connections)?,
            '5' => show_random_aircraft_and_route(&mut database_connections)?,
            's' => show_random_route_for_selected_aircraft(&mut database_connections)?,
            'l' => show_all_aircraft(&mut database_connections)?,
            'm' => show_mark_all_not_flown(&mut database_connections)?,
            'h' => show_history(&mut database_connections)?,
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

fn show_mark_all_not_flown<T: DatabaseOperations>(
    database_connections: &mut T,
) -> Result<(), Error> {
    let terminal = console::Term::stdout();
    let ask_confirm = || -> std::io::Result<char> {
        terminal.write_str("Do you want to mark all aircraft as flown? (y/n)\n")?;
        terminal.read_char()
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
            log::info!("Not marking all aircraft as flown");
        }
        Err(e) => {
            log::error!("Failed to read input: {e}");
        }
    }

    Ok(())
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

fn show_random_not_flown_aircraft<T: AircraftOperations>(database_connections: &mut T) {
    match database_connections.random_not_flown_aircraft() {
        Ok(aircraft) => {
            println!("{}", format_aircraft(&aircraft));
        }
        Err(e) => {
            log::error!("Failed to get random not flown aircraft: {e}");
        }
    }
}

fn show_random_aircraft_with_random_airport<T: DatabaseOperations>(
    database_connections: &mut T,
) -> Result<(), Error> {
    let aircraft = database_connections.random_not_flown_aircraft()?;
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
    let distance = calculate_haversine_distance_nm(&departure, &destination);

    println!("Aircraft: {}", format_aircraft(&aircraft));
    println!("Departure: {}", format_airport(&departure));
    println!("Destination: {}", format_airport(&destination));
    println!("Distance: {distance:.2}nm");

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
    let all_aircraft = database_connections.get_all_aircraft()?;
    for aircraft in all_aircraft {
        println!("{}", format_aircraft(&aircraft));
    }

    Ok(())
}

fn show_random_not_flown_aircraft_and_route<T: DatabaseOperations>(
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

    random_not_flown_aircraft_and_route(database_connections, ask_char_fn)
}

fn random_not_flown_aircraft_and_route<
    T: DatabaseOperations,
    F: Fn() -> Result<char, std::io::Error>,
>(
    database_connections: &mut T,
    ask_char_fn: F,
) -> Result<(), Error> {
    let mut aircraft = database_connections.random_not_flown_aircraft()?;
    let departure = database_connections.get_random_airport_for_aircraft(&aircraft)?;
    let destination = database_connections.get_destination_airport(&aircraft, &departure)?;
    let distance = calculate_haversine_distance_nm(&departure, &destination);

    println!("Aircraft: {}", format_aircraft(&aircraft));
    println!("Departure: {}", format_airport(&departure));
    println!("Destination: {}", format_airport(&destination));
    println!("Distance: {distance:.2}nm");

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
    let history_data = database_connections.get_history()?;
    let aircraft_data = database_connections.get_all_aircraft()?;

    if history_data.is_empty() {
        println!("No history found");
        return Ok(());
    }

    for record in history_data {
        let Some(aircraft) = aircraft_data.iter().find(|a| a.id == record.aircraft) else {
            log::warn!("Aircraft not found for id: {}", record.aircraft);
            return Err(Error::Diesel(diesel::result::Error::NotFound));
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
            log::warn!("Invalid id: {e}");
            return Ok(());
        }
    };

    let aircraft = database_connections.get_aircraft_by_id(aircraft_id)?;
    let departure = database_connections.get_random_airport_for_aircraft(&aircraft)?;
    let destination = database_connections.get_destination_airport(&aircraft, &departure)?;
    let distance = calculate_haversine_distance_nm(&departure, &destination);

    println!("Aircraft: {}", format_aircraft(&aircraft));
    println!("Departure: {}", format_airport(&departure));
    println!("Destination: {}", format_airport(&destination));
    println!("Distance: {distance:.2}nm");

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
