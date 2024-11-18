pub mod models;
pub mod schema;
#[cfg(test)]
mod test;

use self::models::*;
use diesel::prelude::*;
use std::io::Result;

//TODO airport data (runway length, runway type)
//TODO select airport by suitable runways
//TODO select destination by suitable runway

define_sql_function! {fn random() -> Text }

const AIRCRAFT_DB_FILENAME: &str = "data.db";
const AIRPORT_DB_FILENAME: &str = "airports.db3";
const EARTH_RADIUS_KM: f64 = 6371.0;
const KM_TO_NM: f64 = 0.53995680345572;

fn main() -> Result<()> {
    env_logger::init();

    let connection_aircraft = &mut establish_database_connection(AIRCRAFT_DB_FILENAME);
    let connection_airport = &mut establish_database_connection(AIRPORT_DB_FILENAME);

    let terminal = console::Term::stdout();
    terminal.clear_screen()?;

    loop {
        let unflown_aircraft_count = get_unflown_aircraft_count(connection_aircraft).unwrap();

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

        let input = terminal.read_char()?;
        terminal.clear_screen()?;

        match input {
            // '1' => show_random_airport(connection_airport),
            '2' => show_random_unflown_aircraft(connection_aircraft),
            // '3' => show_random_aircraft_with_random_airport(&aircraft_database, &airport_database)?,
            // '4' => show_random_aircraft_and_route(&aircraft_database, &airport_database)?,
            // 'l' => show_all_aircraft(&aircraft_database)?,
            // 'h' => show_history(&aircraft_database)?,
            'q' => {
                log::info!("Quitting");
                break;
            }
            _ => {
                println!("Invalid input");
            }
        }
    }
    Ok(())
}

fn establish_database_connection(database_name: &str) -> SqliteConnection {
    SqliteConnection::establish(database_name).unwrap_or_else(|_| {
        panic!("Error connecting to {}", database_name);
    })
}

// fn show_random_airport(airport_database: &mut SqliteConnection,) {
//     match airport_database.get_random_airport() {
//         Ok(airport) => {
//             println!("{}", format_airport(&airport));

//             for runway in &airport.runways {
//                 println!("{}", format_runway(runway));
//             }
//         }
//         Err(e) => {
//             log::error!("Error: {}", e);
//         }
//     }
//     Ok(())
// }

fn show_random_unflown_aircraft(connection: &mut SqliteConnection) {
    let aircraft = random_unflown_aircraft(connection).unwrap();
    println!("{}", format_aircraft(&aircraft));
}

// fn show_random_aircraft_with_random_airport(
//     aircraft_database: &AircraftDatabase,
//     airport_database: &AirportDatabase,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let aircraft = aircraft_database.random_unflown_aircraft();
//     let airport = match airport_database.get_random_airport_for_aircraft(&aircraft) {
//         Ok(airport) => airport,
//         Err(e) => {
//             log::error!("Failed to get random airport for aircraft: {}", e);
//             return Ok(());
//         }
//     };

//     println!(
//         "Aircraft: {} {}{}, range: {}\nAirport: {} ({}), altitude: {}",
//         aircraft.manufacturer,
//         aircraft.variant,
//         if aircraft.icao_code.is_empty() {
//             "".to_string()
//         } else {
//             format!(" ({})", aircraft.icao_code)
//         },
//         aircraft.aircraft_range,
//         airport.name,
//         airport.icao_code,
//         airport.elevation
//     );

//     for runway in &airport.runways {
//         println!("{}", format_runway(runway));
//     }
//     Ok(())
// }

// fn show_all_aircraft(
//     aircraft_database: &AircraftDatabase,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     match aircraft_database.get_all_aircraft() {
//         Ok(aircrafts) => {
//             if aircrafts.is_empty() {
//                 println!("No aircraft found");
//                 return Ok(());
//             }
//             for aircraft in aircrafts {
//                 println!(
//                     "{} {}{}, range: {}, flown: {}, date flown: {}",
//                     aircraft.manufacturer,
//                     aircraft.variant,
//                     if aircraft.icao_code.is_empty() {
//                         "".to_string()
//                     } else {
//                         format!(" ({})", aircraft.icao_code)
//                     },
//                     aircraft.aircraft_range,
//                     aircraft.flown,
//                     match &aircraft.date_flown {
//                         Some(date) => date.as_str(),
//                         None => "never",
//                     }
//                 );
//             }
//         }
//         Err(e) => {
//             log::error!("Error: {}", e);
//         }
//     }
//     Ok(())
// }

// fn show_random_aircraft_and_route(
//     aircraft_database: &AircraftDatabase,
//     airport_database: &AirportDatabase,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let mut aircraft = aircraft_database.random_unflown_aircraft();

//     let departure = match airport_database.get_random_airport_for_aircraft(&aircraft) {
//         Ok(airport) => airport,
//         Err(e) => {
//             log::error!("Failed to get random airport for aircraft: {}", e);
//             return Ok(());
//         }
//     };

//     let destination = match airport_database.get_destination_airport(&aircraft, &departure) {
//         Ok(airport) => airport,
//         Err(e) => {
//             log::error!("Failed to get destination airport: {}", e);
//             return Ok(());
//         }
//     };

//     let distance = airport_database.haversine_distance_nm(&departure, &destination);

//     println!(
//         "Aircraft: {} {}{}, range: {}\nDeparture: {} ({}), altitude: {}\nDestination: {} ({}), altitude: {}\nDistance: {} nm",
//         aircraft.manufacturer,
//         aircraft.variant,
//         if aircraft.icao_code.is_empty() { "".to_string() } else { format!(" ({})", aircraft.icao_code) },
//         aircraft.aircraft_range,
//         departure.name,
//         departure.icao_code,
//         departure.elevation,
//         destination.name,
//         destination.icao_code,
//         destination.elevation,
//         distance
//     );

//     println!("\nDeparture runways:");
//     for runway in &departure.runways {
//         println!("{}", format_runway(runway));
//     }

//     println!("\nDestination runways:");
//     for runway in &destination.runways {
//         println!("{}", format_runway(runway));
//     }

//     let term = console::Term::stdout();
//     term.write_str("Do you want to mark the aircraft as flown? (y/n)\n")?;
//     let char = term.read_char()?;
//     if char == 'y' {
//         let now = chrono::Local::now();
//         let date = now.format("%Y-%m-%d").to_string();
//         aircraft.date_flown = date;
//         aircraft.flown = 1;
//         if let Err(e) = aircraft_database.update_aircraft(&aircraft) {
//             log::error!("Failed to update aircraft: {}", e);
//             return Ok(());
//         }
//     }

//     if let Err(e) = aircraft_database.add_to_history(&departure, &destination, &aircraft) {
//         log::error!("Failed to add to history: {}", e);
//     }
//     Ok(())
// }

// fn show_history(aircraft_database: &AircraftDatabase) -> Result<(), Box<dyn std::error::Error>> {
//     let history = match aircraft_database.get_history() {
//         Ok(history) => history,
//         Err(e) => {
//             log::error!("Failed to get history: {}", e);
//             return Ok(());
//         }
//     };
//     let aircrafts = match aircraft_database.get_all_aircraft() {
//         Ok(aircrafts) => aircrafts,
//         Err(e) => {
//             log::error!("Failed to get aircrafts: {}", e);
//             return Ok(());
//         }
//     };

//     if history.is_empty() {
//         println!("No history found");
//         return Ok(());
//     }

//     for entry in history {
//         match aircrafts.iter().find(|a| a.id == entry.aircraft_id) {
//             Some(aircraft) => {
//                 println!(
//                     "Flight: {} -> {} with the {} on {}",
//                     entry.departure_icao, entry.arrival_icao, aircraft.variant, entry.date
//                 );
//             }
//             None => {
//                 log::error!("Aircraft not found for history entry: {:?}", entry);
//             }
//         }
//     }
//     Ok(())
// }

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

// fn format_airport(airport: &Airport) -> String {
//     format!(
//         "{} ({}), altitude: {}",
//         airport.name, airport.icao_code, airport.elevation
//     )
// }

// fn format_runway(runway: &Runway) -> String {
//     format!(
//         "Runway: {}, heading: {:.2}, length: {}, width: {}, surface: {}, elevation: {}ft",
//         runway.ident,
//         runway.true_heading,
//         runway.length,
//         runway.width,
//         runway.surface,
//         runway.elevation
//     )
// }

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

// pub fn insert_airport(&self, airport: &Airport) -> Result<(), sqlite::Error> {
//     let query = "INSERT INTO `Airports` (`Name`, `ICAO`, `Latitude`, `Longtitude`, `Elevation`) VALUES (?, ?, ?, ?, ?)";

//     let mut stmt = self.connection.prepare(query)?;
//     stmt.bind((1, airport.name.as_str()))?;
//     stmt.bind((2, airport.icao_code.as_str()))?;
//     stmt.bind((3, airport.latitude))?;
//     stmt.bind((4, airport.longtitude))?;
//     stmt.bind((5, airport.elevation))?;
//     stmt.next()?;

//     Ok(())
// }

// pub fn get_runways_for_airport(&self, airport_id: i64) -> Result<Vec<Runway>, sqlite::Error> {
//     let query = "SELECT * FROM `Runways` WHERE `AirportID` = ?";

//     let mut stmt = self.connection.prepare(query)?;
//     stmt.bind((1, airport_id))?;

//     let cursor = stmt.iter();
//     let mut runways = Vec::new();

//     for result in cursor {
//         let row = result?;
//         let runway = Runway {
//             id: row.read::<i64, _>("ID"),
//             airport_id: row.read::<i64, _>("AirportID"),
//             ident: row.read::<&str, _>("Ident").to_string(),
//             true_heading: row.read::<f64, _>("TrueHeading"),
//             length: row.read::<i64, _>("Length"),
//             width: row.read::<i64, _>("Width"),
//             surface: row.read::<&str, _>("Surface").to_string(),
//             latitude: row.read::<f64, _>("Latitude"),
//             longtitude: row.read::<f64, _>("Longtitude"),
//             elevation: row.read::<i64, _>("Elevation"),
//         };
//         runways.push(runway);
//     }
//     Ok(runways)
// }

// pub fn insert_runway(&self, runway: &Runway) -> Result<(), sqlite::Error> {
//     let query = "INSERT INTO `Runways` (`AirportID`, `Ident`, `TrueHeading`, `Length`, `Width`, `Surface`, `Latitude`, `Longtitude`, `Elevation`) VALUES (:airport_id, :ident, :true_heading, :length, :width, :surface, :latitude, :longtitude, :elevation)";

//     let mut stmt = self.connection.prepare(query)?;
//     stmt.bind((":airport_id", runway.airport_id))?;
//     stmt.bind((":ident", runway.ident.as_str()))?;
//     stmt.bind((":true_heading", runway.true_heading))?;
//     stmt.bind((":length", runway.length))?;
//     stmt.bind((":width", runway.width))?;
//     stmt.bind((":surface", runway.surface.as_str()))?;
//     stmt.bind((":latitude", runway.latitude))?;
//     stmt.bind((":longtitude", runway.longtitude))?;
//     stmt.bind((":elevation", runway.elevation))?;
//     stmt.next()?;

//     Ok(())
// }

// pub fn get_random_airport(&self) -> Result<Airport, sqlite::Error> {
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

//         Ok(airport)
//     } else {
//         Err(sqlite::Error {
//             code: Some(sqlite::ffi::SQLITE_ERROR as isize),
//             message: Some("No rows returned".to_string()),
//         })
//     }
// }

// pub fn create_runway_vec(&self, airport_id: i64) -> Vec<Runway> {
//     match self.get_runways_for_airport(airport_id) {
//         Ok(runways) => runways,
//         Err(e) => {
//             log::error!("Failed to get runways: {}", e);
//             Vec::new()
//         }
//     }
// }

// pub fn get_destination_airport(
//     &self,
//     aircraft: &Aircraft,
//     departure: &Airport,
// ) -> Result<Airport, sqlite::Error> {
//     let max_aircraft_range_nm = aircraft.aircraft_range;
//     let origin_lat = departure.latitude;
//     let origin_lon = departure.longtitude;

//     let max_difference_degrees = (max_aircraft_range_nm as f64) / 60.0;
//     let min_lat = origin_lat - max_difference_degrees;
//     let max_lat = origin_lat + max_difference_degrees;
//     let min_lon = origin_lon - max_difference_degrees;
//     let max_lon = origin_lon + max_difference_degrees;

//     let query = "SELECT * FROM `Airports` WHERE `ID` != :airport_id AND `ICAO` != :airport_icao AND `Latitude` BETWEEN :min_lat AND :max_lat AND `Longtitude` BETWEEN :min_long AND :max_long ORDER BY RANDOM()";
//     let mut stmt = self.connection.prepare(query)?;
//     stmt.bind((":airport_id", departure.id))?;
//     stmt.bind((":airport_icao", departure.icao_code.as_str()))?;
//     stmt.bind((":min_lat", min_lat))?;
//     stmt.bind((":max_lat", max_lat))?;
//     stmt.bind((":min_long", min_lon))?;
//     stmt.bind((":max_long", max_lon))?;

//     let cursor = stmt.iter();
//     for result in cursor {
//         let row = result.unwrap();
//         let destination = Airport {
//             id: row.read::<i64, _>("ID"),
//             name: row.read::<&str, _>("Name").to_string(),
//             icao_code: row.read::<&str, _>("ICAO").to_string(),
//             latitude: row.read::<f64, _>("Latitude"),
//             longtitude: row.read::<f64, _>("Longtitude"),
//             elevation: row.read::<i64, _>("Elevation"),
//             runways: self.create_runway_vec(row.read::<i64, _>("ID")),
//         };

//         if destination.icao_code == departure.icao_code {
//             continue;
//         }

//         let distance = self.haversine_distance_nm(departure, &destination);

//         if distance <= max_aircraft_range_nm {
//             let ruwnays = self.get_runways_for_airport(destination.id).unwrap();
//             for runway in ruwnays {
//                 log::info!("Runway: {:?}", runway);
//             }
//             return Ok(destination);
//         }
//     }
//     Err(sqlite::Error {
//         code: Some(sqlite::ffi::SQLITE_ERROR as isize),
//         message: Some("No suitable destination found".to_string()),
//     })
// }

// pub fn haversine_distance_nm(&self, airport1: &Airport, airport2: &Airport) -> i64 {
//     let lat1 = airport1.latitude.to_radians();
//     let lon1 = airport1.longtitude.to_radians();
//     let lat2 = airport2.latitude.to_radians();
//     let lon2 = airport2.longtitude.to_radians();

//     let dlat = lat2 - lat1;
//     let dlon = lon2 - lon1;

//     let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
//     let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
//     let distance_km = EARTH_RADIUS_KM * c;

//     f64::round(distance_km * KM_TO_NM) as i64
// }

// pub fn update_aircraft(&self, aircraft: &Aircraft) -> Result<(), sqlite::Error> {
//     let query = "UPDATE aircraft SET manufacturer = :manufacturer, variant = :variant, icao_code = :icao_code, flown = :flown, aircraft_range = :aircraft_range, category = :category, cruise_speed = :cruise_speed, date_flown = :date_flown WHERE id = :id";

//     let date_flown = match &aircraft.date_flown {
//         Some(date) => date.as_str(),
//         None => "",
//     };

//     let mut stmt = self.connection.prepare(query)?;
//     stmt.bind((":manufacturer", aircraft.manufacturer.as_str()))?;
//     stmt.bind((":variant", aircraft.variant.as_str()))?;
//     stmt.bind((":icao_code", aircraft.icao_code.as_str()))?;
//     stmt.bind((":flown", if aircraft.flown { 1 } else { 0 }))?;
//     stmt.bind((":aircraft_range", aircraft.aircraft_range))?;
//     stmt.bind((":category", aircraft.category.as_str()))?;
//     stmt.bind((":cruise_speed", aircraft.cruise_speed))?;
//     stmt.bind((":date_flown", date_flown))?;
//     stmt.bind((":id", aircraft.id))?;
//     stmt.next()?;

//     Ok(())
// }

pub fn get_unflown_aircraft_count(connection: &mut SqliteConnection) -> Result<i32> {
    use self::schema::aircraft::dsl::*;

    let count: i64 = aircraft
        .filter(flown.eq(0))
        .count()
        .get_result(connection)
        .expect("Error counting unflown aircraft");

    Ok(count as i32)
}

// pub fn mark_all_aircraft_unflown(&self) -> Result<(), sqlite::Error> {
//     let query = "UPDATE aircraft SET flown = 0, date_flown = NULL";x
//     let mut stmt = self.connection.prepare(query)?;
//     stmt.next()?;

//     Ok(())
// }

pub fn random_unflown_aircraft(connection: &mut SqliteConnection) -> Result<Aircraft> {
    use self::schema::aircraft::dsl::*;

    let results: Vec<Aircraft> = aircraft
        .filter(flown.eq(0))
        .order(random())
        .limit(1)
        .load(connection)
        .expect("Error loading aircraft");

    Ok(results[0].clone())
}

// pub fn get_all_aircraft(&self) -> Result<Vec<Aircraft>, sqlite::Error> {
//     let mut aircrafts = Vec::new();
//     let query = "SELECT * FROM aircraft";

//     let mut stmt = self.connection.prepare(query).unwrap();

//     let cursor = stmt.iter();
//     for result in cursor {
//         let row = result.unwrap();
//         let aircraft = Aircraft {
//             id: row.read::<i64, _>("id"),
//             manufacturer: row.read::<&str, _>("manufacturer").to_string(),
//             variant: row.read::<&str, _>("variant").to_string(),
//             icao_code: row.read::<&str, _>("icao_code").to_string(),
//             flown: row.read::<i64, _>("flown") == 1,
//             aircraft_range: row.read::<i64, _>("aircraft_range"),
//             category: row.read::<&str, _>("category").to_string(),
//             cruise_speed: row.read::<i64, _>("cruise_speed"),
//             date_flown: row
//                 .read::<Option<&str>, _>("date_flown")
//                 .map(|s| s.to_string()),
//         };
//         aircrafts.push(aircraft);
//     }
//     Ok(aircrafts)
// }

// fn add_to_history(
//     &self,
//     departure: &Airport,
//     destination: &Airport,
//     aircraft: &Aircraft,
// ) -> Result<(), sqlite::Error> {
//     let query = "INSERT INTO history (departure_icao, arrival_icao, aircraft, date) VALUES (:departure_icao, :arrival_icao, :aircraft, :date)";

//     let now = chrono::Local::now();
//     let date = now.format("%Y-%m-%d").to_string();

//     let mut stmt = self.connection.prepare(query)?;
//     stmt.bind((":departure_icao", departure.icao_code.as_str()))?;
//     stmt.bind((":arrival_icao", destination.icao_code.as_str()))?;
//     stmt.bind((":aircraft", aircraft.id))?;
//     stmt.bind((":date", date.as_str()))?;
//     stmt.next()?;
//     Ok(())
// }

// fn get_history(&self) -> Result<Vec<History>, sqlite::Error> {
//     let mut history = Vec::new();
//     let query = "SELECT * FROM history";

//     let mut stmt = self.connection.prepare(query)?;

//     let cursor = stmt.iter();
//     for result in cursor {
//         let row = result.unwrap();
//         let entry = History {
//             id: row.read::<i64, _>("id"),
//             departure_icao: row.read::<&str, _>("departure_icao").to_string(),
//             arrival_icao: row.read::<&str, _>("arrival_icao").to_string(),
//             aircraft_id: row.read::<i64, _>("aircraft"),
//             date: row.read::<&str, _>("date").to_string(),
//         };
//         history.push(entry);
//     }
//     Ok(history)
// }

pub fn insert_aircraft(connection: &mut SqliteConnection, record: &Aircraft) {
    use self::schema::aircraft::dsl::*;

    diesel::insert_into(aircraft)
        .values(record)
        .execute(connection)
        .expect("Error inserting aircraft");
}
