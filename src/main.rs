pub mod models;
pub mod schema;

#[cfg(test)]
mod test;

use self::models::*;
use diesel::prelude::*;
use diesel::result::Error;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

//TODO airport data (runway length, runway type)
//TODO select airport by suitable runways
//TODO select destination by suitable runway

define_sql_function! {fn random() -> Text }

const AIRCRAFT_DB_FILENAME: &str = "data.db";
const AIRPORT_DB_FILENAME: &str = "airports.db3";
const EARTH_RADIUS_KM: f64 = 6371.0;
const KM_TO_NM: f64 = 0.53995680345572;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

fn main() {
    env_logger::init();

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
             4. Random aircraft, airport and destination\n\
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
            '4' => show_random_aircraft_and_route(connection_aircraft, connection_airport)?,
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
    let airport = get_random_airport(airport_connection)?;

    println!(
        "Aircraft: {} {}{}, range: {}\nAirport: {} ({}), altitude: {}",
        aircraft.manufacturer,
        aircraft.variant,
        if aircraft.icao_code.is_empty() {
            "".to_string()
        } else {
            format!(" ({})", aircraft.icao_code)
        },
        aircraft.aircraft_range,
        airport.Name,
        airport.ICAO,
        airport.Elevation
    );

    for runway in get_runways_for_airport(airport_connection, &airport)? {
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

fn show_random_aircraft_and_route(
    aircraft_connection: &mut SqliteConnection,
    airport_connection: &mut SqliteConnection,
) -> Result<(), Error> {
    let mut aircraft = random_unflown_aircraft(aircraft_connection)?;
    let departure = get_random_airport(airport_connection)?;
    let destination = get_destination_airport(airport_connection, &aircraft, &departure)?;

    let distance = haversine_distance_nm(&departure, &destination);

    println!(
        "Aircraft: {} {}{}, range: {}\nDeparture: {} ({}), altitude: {}\nDestination: {} ({}), altitude: {}\nDistance: {} nm",
        aircraft.manufacturer,
        aircraft.variant,
        if aircraft.icao_code.is_empty() { "".to_string() } else { format!(" ({})", aircraft.icao_code) },
        aircraft.aircraft_range,
        departure.Name,
        departure.ICAO,
        departure.Elevation,
        destination.Name,
        destination.ICAO,
        destination.Elevation,
        distance
    );

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

fn format_aircraft(aircraft: &Aircraft) -> String {
    format!(
        "{} {} ({}), range: {} nm, category: {}, cruise speed: {} knots",
        aircraft.manufacturer,
        aircraft.variant,
        aircraft.icao_code,
        aircraft.aircraft_range,
        aircraft.category,
        aircraft.cruise_speed
    )
}

fn format_airport(airport: &Airport) -> String {
    format!(
        "{} ({}), altitude: {}",
        airport.Name, airport.ICAO, airport.Elevation
    )
}

fn format_runway(runway: &Runway) -> String {
    format!(
        "Runway: {}, heading: {:.2}, length: {}, width: {}, surface: {}, elevation: {}ft",
        runway.Ident,
        runway.TrueHeading,
        runway.Length,
        runway.Width,
        runway.Surface,
        runway.Elevation
    )
}

// pub fn get_random_airport_for_aircraft(
//     &self,
//     _aircraft: &Aircaft,
// ) -> Result<Airport, sqlite::Error> {
//     let query = "SELECT * FROM `Airports` ORDER BY RANDOM() LIMIT 1";

//     let mut stmt = self.connection.prepare(query)?;

//     let mut cursor = stmt.iter();

//     if let Some(result) = cursor.next() {
//         let row = result?;
//         let airport = Airport {
//             id: row.read::<i64, _>("ID"),
//             name: row.read::<&str, _>("Name").to_string(),
//             icao_code: row.read::<&str, _>("ICAO").to_string(),
//             latitude: row.read::<f64, _>("Latitude"),
//             longtitude: row.read::<f64, _>("Longtitude"),
//             elevation: row.read::<i64, _>("Elevation"),
//             runways: self.create_runway_vec(row.read::<i64, _>("ID")),
//         };

//         return Ok(airport);
//     }

//     Err(sqlite::Error {
//         code: Some(sqlite::ffi::SQLITE_ERROR as isize),
//         message: Some("No rows returned".to_string()),
//     })
// }

pub fn insert_airport(
    connection: &mut SqliteConnection,
    record: &Airport,
) -> Result<(), diesel::result::Error> {
    use self::schema::Airports::dsl::*;

    diesel::insert_into(Airports)
        .values(record)
        .execute(connection)?;

    Ok(())
}

pub fn insert_runway(
    connection: &mut SqliteConnection,
    record: &Runway,
) -> Result<(), diesel::result::Error> {
    use self::schema::Runways::dsl::*;
    diesel::insert_into(Runways)
        .values(record)
        .execute(connection)?;
    Ok(())
}

pub fn get_random_airport(
    connection: &mut SqliteConnection,
) -> Result<Airport, diesel::result::Error> {
    use self::schema::Airports::dsl::*;

    let airport: Airport = Airports.order(random()).limit(1).get_result(connection)?;
    Ok(airport)
}

pub fn get_destination_airport(
    connection: &mut SqliteConnection,
    aircraft: &Aircraft,
    departure: &Airport,
) -> Result<Airport, diesel::result::Error> {
    use self::schema::Airports::dsl::*;

    let max_aircraft_range_nm = aircraft.aircraft_range;
    let origin_lat = departure.Latitude;
    let origin_lon = departure.Longtitude;

    let max_difference_degrees = (max_aircraft_range_nm as f64) / 60.0;
    let min_lat = origin_lat - max_difference_degrees;
    let max_lat = origin_lat + max_difference_degrees;
    let min_lon = origin_lon - max_difference_degrees;
    let max_lon = origin_lon + max_difference_degrees;

    let airport: Airport = Airports
        .filter(Latitude.between(min_lat, max_lat))
        .filter(Longtitude.between(min_lon, max_lon))
        .filter(ID.ne(departure.ID))
        .order(random())
        .get_result(connection)?;

    let distance = haversine_distance_nm(departure, &airport);
    if distance > aircraft.aircraft_range {
        return get_destination_airport(connection, aircraft, departure);
    }

    Ok(airport)
}

pub fn haversine_distance_nm(airport1: &Airport, airport2: &Airport) -> i32 {
    let lat1 = airport1.Latitude.to_radians();
    let lon1 = airport1.Longtitude.to_radians();
    let lat2 = airport2.Latitude.to_radians();
    let lon2 = airport2.Longtitude.to_radians();

    let dlat = lat2 - lat1;
    let dlon = lon2 - lon1;

    let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    let distance_km = EARTH_RADIUS_KM * c;

    f64::round(distance_km * KM_TO_NM) as i32
}

pub fn update_aircraft(
    connection: &mut SqliteConnection,
    record: &Aircraft,
) -> Result<(), diesel::result::Error> {
    use self::schema::aircraft::dsl::*;

    diesel::update(aircraft.find(record.id))
        .set(record)
        .execute(connection)?;

    Ok(())
}

pub fn get_unflown_aircraft_count(
    connection: &mut SqliteConnection,
) -> Result<i32, diesel::result::Error> {
    use self::schema::aircraft::dsl::*;

    let count: i64 = aircraft
        .filter(flown.eq(0))
        .count()
        .get_result(connection)?;

    Ok(count as i32)
}

pub fn mark_all_aircraft_unflown(
    connection: &mut SqliteConnection,
) -> Result<(), diesel::result::Error> {
    use self::schema::aircraft::dsl::*;

    diesel::update(aircraft)
        .set(flown.eq(0))
        .execute(connection)?;

    Ok(())
}

pub fn random_unflown_aircraft(
    connection: &mut SqliteConnection,
) -> Result<Aircraft, diesel::result::Error> {
    use self::schema::aircraft::dsl::*;

    let record: Aircraft = aircraft
        .filter(flown.eq(0))
        .order(random())
        .limit(1)
        .get_result(connection)?;

    Ok(record)
}

pub fn get_all_aircraft(
    connection: &mut SqliteConnection,
) -> Result<Vec<Aircraft>, diesel::result::Error> {
    use self::schema::aircraft::dsl::*;
    let records: Vec<Aircraft> = aircraft.load(connection)?;
    Ok(records)
}

fn add_to_history(
    connection: &mut SqliteConnection,
    departure: &Airport,
    arrival: &Airport,
    aircraft_record: &Aircraft,
) -> Result<(), diesel::result::Error> {
    use self::schema::history::dsl::*;

    let date_string = chrono::Local::now().format("%Y-%m-%d").to_string();
    diesel::insert_into(history)
        .values((
            departure_icao.eq(&departure.ICAO),
            arrival_icao.eq(&arrival.ICAO),
            aircraft.eq(&aircraft_record.id),
            date.eq(&date_string),
        ))
        .execute(connection)?;
    Ok(())
}

fn get_history(connection: &mut SqliteConnection) -> Result<Vec<History>, diesel::result::Error> {
    use self::schema::history::dsl::*;

    let records: Vec<History> = history.load(connection)?;
    Ok(records)
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::aircraft)]
struct AircraftForm<'a> {
    manufacturer: &'a str,
    variant: &'a str,
    icao_code: &'a str,
    flown: i32,
    aircraft_range: i32,
    category: &'a str,
    cruise_speed: i32,
    date_flown: Option<&'a str>,
}

pub fn insert_aircraft(
    connection: &mut SqliteConnection,
    record: &Aircraft,
) -> Result<(), diesel::result::Error> {
    use self::schema::aircraft::dsl::*;

    let new_aircraft = AircraftForm {
        manufacturer: &record.manufacturer,
        variant: &record.variant,
        icao_code: &record.icao_code,
        flown: record.flown,
        aircraft_range: record.aircraft_range,
        category: &record.category,
        cruise_speed: record.cruise_speed,
        date_flown: record.date_flown.as_deref(),
    };

    diesel::insert_into(aircraft)
        .values(&new_aircraft)
        .execute(connection)?;
    Ok(())
}

pub fn get_runways_for_airport(
    connection: &mut SqliteConnection,
    airport: &Airport,
) -> Result<Vec<Runway>, diesel::result::Error> {
    let runways = Runway::belonging_to(airport).load(connection)?;
    Ok(runways)
}
