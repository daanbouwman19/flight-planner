use log;
use std::fs;

//TODO airport data (runway length, runway type)
//TODO select airport by suitable runways
//TODO select destination by suitable runway

#[cfg(test)]
mod test;

fn main() {
    env_logger::init();

    if !fs::metadata("data.db").is_ok() {
        log::info!("Aircraft database file does not exist. Creating and initializing...");
        let aircraft_db_connection = sqlite::open("data.db").unwrap();
        initialize_aircraft_db(&aircraft_db_connection);
    } else {
        log::info!("Aircraft database file exists.");
    }

    if !fs::metadata("airports.db3").is_ok() {
        log::info!("Airport database file does not exist. Creating and initializing...");
        let airport_db_connection = sqlite::open("airports.db3").unwrap();
        initialize_airport_db(&airport_db_connection);
    } else {
        log::info!("Airport database file exists.");
    }

    let aircraft_db_connection = sqlite::open("data.db").unwrap();
    let airport_db_connection = sqlite::open("airports.db3").unwrap();

    let airport_database = AirportDatabase::new(airport_db_connection);
    let aircraft_database = AircraftDatabase::new(aircraft_db_connection);
    let unflown_aircraft_count = aircraft_database.get_unflown_aircraft_count().unwrap();

    let terminal = console::Term::stdout();
    terminal.clear_screen().unwrap();

    loop {
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

        let char = terminal.read_char().unwrap();
        terminal.clear_screen().unwrap();

        match char {
            '1' => {
                let airport = airport_database.get_random_airport().unwrap();
                println!(
                    "{} ({}), altitude: {}",
                    airport.name, airport.icao_code, airport.elevation
                );
            }
            '2' => {
                let aircraft = aircraft_database.random_unflown_aircraft().unwrap();
                println!(
                    "{} {}{}, range: {}",
                    aircraft.manufacturer,
                    aircraft.variant,
                    if aircraft.icao_code.is_empty() {
                        "".to_string()
                    } else {
                        format!(" ({})", aircraft.icao_code)
                    },
                    aircraft.aircraft_range
                );
            }
            '3' => {
                let aircraft = aircraft_database.random_unflown_aircraft().unwrap();
                let airport = airport_database.get_random_airport().unwrap();

                println!(
                    "Aircraft: {} {} ({}), range: {}\nAirport: {} ({}), altitude: {}",
                    aircraft.manufacturer,
                    aircraft.variant,
                    aircraft.icao_code,
                    aircraft.aircraft_range,
                    airport.name,
                    airport.icao_code,
                    airport.elevation
                );
            }
            '4' => {
                get_random_aircraft_and_route(&aircraft_database, &airport_database);
            }
            'l' => {
                list_all_aircraft(&aircraft_database);
            }
            'q' => {
                log::info!("Quitting");
                break;
            }
            'h' => show_history(&aircraft_database),
            _ => {
                println!("Invalid input");
            }
        }
    }
}
fn initialize_airport_db(connection: &sqlite::Connection) {
    let query = "
        CREATE TABLE `Airports` (
            ID INTEGER PRIMARY KEY,
            Name TEXT NOT NULL,
            ICAO TEXT NOT NULL,
            Latitude FLOAT NOT NULL,
            Longtitude FLOAT NOT NULL,
            Elevation INTEGER NOT NULL
        );
        CREATE TABLE Runways (
            ID INTEGER PRIMARY KEY,
            AirportID INTEGER NOT NULL,
            Length INTEGER NOT NULL,
            Surface TEXT NOT NULL,
            FOREIGN KEY (AirportID) REFERENCES airport(ID)
        );
    ";
    connection.execute(query).unwrap();
}

fn initialize_aircraft_db(connection: &sqlite::Connection) {
    let query = "
        CREATE TABLE aircraft (
            id INTEGER PRIMARY KEY,
            manufacturer TEXT NOT NULL,
            variant TEXT NOT NULL,
            icao_code TEXT NOT NULL,
            flown INTEGER NOT NULL,
            aircraft_range INTEGER NOT NULL,
            category TEXT NOT NULL,
            cruise_speed INTEGER NOT NULL,
            date_flown TEXT
        );
        CREATE TABLE history (
            id INTEGER PRIMARY KEY,
            departure_icao TEXT NOT NULL,
            arrival_icao TEXT NOT NULL,
            aircraft INTEGER NOT NULL,
            date TEXT NOT NULL
        );
    ";
    connection.execute(query).unwrap();
}

fn list_all_aircraft(aircraft_picker: &AircraftDatabase) {
    let aircrafts = aircraft_picker.get_all_aircraft().unwrap();
    if aircrafts.is_empty() {
        println!("No aircraft found");
        return;
    }

    for aircraft in aircrafts {
        println!(
            "{} {}{}, range: {}, flown: {}, date flown: {}",
            aircraft.manufacturer,
            aircraft.variant,
            if aircraft.icao_code.is_empty() {
                "".to_string()
            } else {
                format!(" ({})", aircraft.icao_code)
            },
            aircraft.aircraft_range,
            aircraft.flown,
            match &aircraft.date_flown {
                Some(date) => date.as_str(),
                None => "never",
            }
        );
    }
}

fn get_random_aircraft_and_route(
    aircraft_picker: &AircraftDatabase,
    airport_picker: &AirportDatabase,
) {
    let mut aircraft = aircraft_picker.random_unflown_aircraft().unwrap();

    let departure = airport_picker
        .get_random_airport_for_aircraft(&aircraft)
        .unwrap();

    let destination = airport_picker
        .get_destination_airport(&aircraft, &departure)
        .unwrap();

    let distance = airport_picker.haversine_distance_nm(&departure, &destination);

    println!(
        "Aircraft: {} {}{}, range: {}\nDeparture: {} ({}), altitude: {}\nDestination: {} ({}), altitude: {}\nDistance: {} nm",
        aircraft.manufacturer,
        aircraft.variant,
        if aircraft.icao_code.is_empty() { "".to_string() } else { format!(" ({})", aircraft.icao_code) },
        aircraft.aircraft_range,
        departure.name,
        departure.icao_code,
        departure.elevation,
        destination.name,
        destination.icao_code,
        destination.elevation,
        distance
    );

    let term = console::Term::stdout();
    term.write_str("Do you want to mark the aircraft as flown? (y/n)\n")
        .unwrap();
    let char = term.read_char().unwrap();
    if char == 'y' {
        let now = chrono::Local::now();
        let date = now.format("%Y-%m-%d").to_string();
        aircraft.date_flown = Some(date);
        aircraft.flown = true;
        aircraft_picker.update_aircraft(&aircraft).unwrap();
    }

    aircraft_picker
        .add_to_history(&departure, &destination, &aircraft)
        .unwrap();
}

fn show_history(aircraft_picker: &AircraftDatabase) {
    let history = aircraft_picker.get_history().unwrap();
    let aircrafts = aircraft_picker.get_all_aircraft().unwrap();

    if history.is_empty() {
        println!("No history found");
        return;
    }

    for entry in history {
        let aircraft = aircrafts
            .iter()
            .find(|a| a.id == entry.aircraft_id)
            .unwrap();
        println!(
            "Flight: {} -> {} with the {} on {}",
            entry.departure_icao, entry.arrival_icao, aircraft.variant, entry.date
        );
    }
}

#[derive(PartialEq)]
pub struct Aircraft {
    pub id: i64,
    pub manufacturer: String,
    pub variant: String,
    pub icao_code: String,
    pub flown: bool,
    pub aircraft_range: i64,
    pub category: String,
    pub cruise_speed: i64,
    pub date_flown: Option<String>,
}

pub struct History {
    pub id: i64,
    departure_icao: String,
    arrival_icao: String,
    aircraft_id: i64,
    date: String,
}

pub struct AircraftDatabase {
    pub connection: sqlite::Connection,
}

pub struct AirportDatabase {
    pub connection: sqlite::Connection,
}

#[derive(PartialEq)]
pub struct Airport {
    pub id: i64,
    pub name: String,
    pub icao_code: String,
    pub latitude: f64,
    pub longtitude: f64,
    pub elevation: i64,
}

pub struct Runway {
    pub id: i64,
    pub airport_id: i64,
    pub ident: String,
    pub true_heading: f64,
    pub length: i64,
    pub width: i64,
    pub surface: String,
    pub latitude: f64,
    pub longtitude: f64,
    pub elevation: i64,
}

impl std::fmt::Debug for History {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("History")
            .field("id", &self.id)
            .field("departure_icao", &self.departure_icao)
            .field("arrival_icao", &self.arrival_icao)
            .field("aircraft_id", &self.aircraft_id)
            .field("date", &self.date)
            .finish()
    }
}

impl std::fmt::Debug for Aircraft {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Aircraft")
            .field("id", &self.id)
            .field("manufacturer", &self.manufacturer)
            .field("variant", &self.variant)
            .field("icao_code", &self.icao_code)
            .field("flown", &self.flown)
            .field("aircraft_range", &self.aircraft_range)
            .field("category", &self.category)
            .field("cruise_speed", &self.cruise_speed)
            .field("date_flown", &self.date_flown)
            .finish()
    }
}

impl std::fmt::Debug for Airport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Airport")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("icao_code", &self.icao_code)
            .field("latitude", &self.latitude)
            .field("longtitude", &self.longtitude)
            .field("elevation", &self.elevation)
            .finish()
    }
}

impl std::fmt::Debug for Runway {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Runway")
            .field("id", &self.id)
            .field("airport_id", &self.airport_id)
            .field("ident", &self.ident)
            .field("true_heading", &self.true_heading)
            .field("length", &self.length)
            .field("width", &self.width)
            .field("surface", &self.surface)
            .field("latitude", &self.latitude)
            .field("longtitude", &self.longtitude)
            .field("elevation", &self.elevation)
            .finish()
    }
}

impl AirportDatabase {
    pub fn new(connection: sqlite::Connection) -> Self {
        AirportDatabase { connection }
    }

    pub fn get_random_airport_for_aircraft(
        &self,
        _aircraft: &Aircraft,
    ) -> Result<Airport, sqlite::Error> {
        let query = "SELECT * FROM `Airports` ORDER BY RANDOM() LIMIT 1";
        log::debug!("Query: {}", query);

        let mut stmt = self.connection.prepare(query)?;

        let mut cursor = stmt.iter();
        if let Some(result) = cursor.next() {
            let row = result?;
            let airport = Airport {
                id: row.read::<i64, _>("ID"),
                name: row.read::<&str, _>("Name").to_string(),
                icao_code: row.read::<&str, _>("ICAO").to_string(),
                latitude: row.read::<f64, _>("Latitude"),
                longtitude: row.read::<f64, _>("Longtitude"),
                elevation: row.read::<i64, _>("Elevation"),
            };

            let runways = self.get_runways_for_airport(airport.id).unwrap();
            for runway in runways {
                log::info!("Runway: {:?}", runway);
            }
            Ok(airport)
        } else {
            Err(sqlite::Error {
                code: Some(sqlite::ffi::SQLITE_ERROR as isize),
                message: Some("No rows returned".to_string()),
            })
        }
    }

    pub fn insert_airport(&self, airport: &Airport) -> Result<(), sqlite::Error> {
        let query = "INSERT INTO `Airports` (`Name`, `ICAO`, `Latitude`, `Longtitude`, `Elevation`) VALUES (?, ?, ?, ?, ?)";
        log::debug!("Query: {}", query);

        let mut stmt = self.connection.prepare(query)?;
        stmt.bind((1, airport.name.as_str()))?;
        stmt.bind((2, airport.icao_code.as_str()))?;
        stmt.bind((3, airport.latitude))?;
        stmt.bind((4, airport.longtitude))?;
        stmt.bind((5, airport.elevation))?;
        stmt.next()?;

        Ok(())
    }

    pub fn get_runways_for_airport(&self, airport_id: i64) -> Result<Vec<Runway>, sqlite::Error> {
        let query = "SELECT * FROM `Runways` WHERE `AirportID` = ?";
        log::debug!("Query: {}", query);

        let mut stmt = self.connection.prepare(query)?;
        stmt.bind((1, airport_id))?;

        let mut cursor = stmt.iter();
        let mut runways = Vec::new();
        while let Some(result) = cursor.next() {
            let row = result?;
            let runway = Runway {
                id: row.read::<i64, _>("ID"),
                airport_id: row.read::<i64, _>("AirportID"),
                ident: row.read::<&str, _>("Ident").to_string(),
                true_heading: row.read::<f64, _>("TrueHeading"),
                length: row.read::<i64, _>("Length"),
                width: row.read::<i64, _>("Width"),
                surface: row.read::<&str, _>("Surface").to_string(),
                latitude: row.read::<f64, _>("Latitude"),
                longtitude: row.read::<f64, _>("Longtitude"),
                elevation: row.read::<i64, _>("Elevation"),
            };
            runways.push(runway);
        }
        Ok(runways)
    }

    pub fn get_random_airport(&self) -> Result<Airport, sqlite::Error> {
        let query = "SELECT * FROM `Airports` ORDER BY RANDOM() LIMIT 1";
        log::debug!("Query: {}", query);

        let mut stmt = self.connection.prepare(query)?;

        let mut cursor = stmt.iter();
        if let Some(result) = cursor.next() {
            let row = result?;
            let airport = Airport {
                id: row.read::<i64, _>("ID"),
                name: row.read::<&str, _>("Name").to_string(),
                icao_code: row.read::<&str, _>("ICAO").to_string(),
                latitude: row.read::<f64, _>("Latitude"),
                longtitude: row.read::<f64, _>("Longtitude"),
                elevation: row.read::<i64, _>("Elevation"),
            };
            let runways = self.get_runways_for_airport(airport.id).unwrap();
            for runway in runways {
                log::info!("Runway: {:?}", runway);
            }

            Ok(airport)
        } else {
            Err(sqlite::Error {
                code: Some(sqlite::ffi::SQLITE_ERROR as isize),
                message: Some("No rows returned".to_string()),
            })
        }
    }

    pub fn get_destination_airport(
        &self,
        aircraft: &Aircraft,
        departure: &Airport,
    ) -> Result<Airport, sqlite::Error> {
        let max_aircraft_range_nm = aircraft.aircraft_range; //range in nm
        let origin_lat = departure.latitude;
        let origin_lon = departure.longtitude;

        let max_difference_degress = (max_aircraft_range_nm as f64) / 60.0;
        let min_lat = origin_lat - max_difference_degress;
        let max_lat = origin_lat + max_difference_degress;
        let min_lon = origin_lon - max_difference_degress;
        let max_lon = origin_lon + max_difference_degress;

        let query = "SELECT * FROM `Airports` WHERE `ID` != ? AND `ICAO` != ? AND `Latitude` BETWEEN ? AND ? AND `Longtitude` BETWEEN ? AND ? ORDER BY RANDOM()";
        log::debug!("Query: {}", query);

        let mut stmt = self.connection.prepare(query)?;
        stmt.bind((1, departure.id))?;
        stmt.bind((2, departure.icao_code.as_str()))?;
        stmt.bind((3, min_lat))?;
        stmt.bind((4, max_lat))?;
        stmt.bind((5, min_lon))?;
        stmt.bind((6, max_lon))?;

        let mut cursor = stmt.iter();

        while let Some(result) = cursor.next() {
            let row = result?;
            let destination = Airport {
                id: row.read::<i64, _>("ID"),
                name: row.read::<&str, _>("Name").to_string(),
                icao_code: row.read::<&str, _>("ICAO").to_string(),
                latitude: row.read::<f64, _>("Latitude"),
                longtitude: row.read::<f64, _>("Longtitude"),
                elevation: row.read::<i64, _>("Elevation"),
            };

            let distance = self.haversine_distance_nm(&departure, &destination);

            if distance <= max_aircraft_range_nm {
                let ruwnays = self.get_runways_for_airport(destination.id).unwrap();
                for runway in ruwnays {
                    log::info!("Runway: {:?}", runway);
                }
                return Ok(destination);
            }
        }
        Err(sqlite::Error {
            code: Some(sqlite::ffi::SQLITE_ERROR as isize),
            message: Some("No suitable destination found".to_string()),
        })
    }

    pub fn haversine_distance_nm(&self, airport1: &Airport, airport2: &Airport) -> i64 {
        let r = 6371.0; //radius of the earth in km
        let lat1 = airport1.latitude.to_radians();
        let lon1 = airport1.longtitude.to_radians();
        let lat2 = airport2.latitude.to_radians();
        let lon2 = airport2.longtitude.to_radians();

        let dlat = lat2 - lat1;
        let dlon = lon2 - lon1;

        let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        let distance_km = r * c;

        f64::round(distance_km * 0.53995680345572) as i64 //convert to nm
    }
}

impl AircraftDatabase {
    pub fn new(connection: sqlite::Connection) -> Self {
        AircraftDatabase { connection }
    }

    pub fn update_aircraft(&self, aircraft: &Aircraft) -> Result<(), sqlite::Error> {
        let query = "UPDATE aircraft SET manufacturer = ?, variant = ?, icao_code = ?, flown = ?, aircraft_range = ?, category = ?, cruise_speed = ?, date_flown=? WHERE id = ?";
        log::debug!("Query: {}", query);

        let date_flown = match &aircraft.date_flown {
            Some(date) => date.as_str(),
            None => "",
        };

        let mut stmt = self.connection.prepare(query)?;
        stmt.bind((1, aircraft.manufacturer.as_str()))?;
        stmt.bind((2, aircraft.variant.as_str()))?;
        stmt.bind((3, aircraft.icao_code.as_str()))?;
        stmt.bind((4, if aircraft.flown { 1 } else { 0 }))?;
        stmt.bind((5, aircraft.aircraft_range))?;
        stmt.bind((6, aircraft.category.as_str()))?;
        stmt.bind((7, aircraft.cruise_speed))?;
        stmt.bind((8, date_flown))?;
        stmt.bind((9, aircraft.id))?;
        stmt.next()?;

        Ok(())
    }

    pub fn get_unflown_aircraft_count(&self) -> Result<i64, sqlite::Error> {
        let query = "SELECT COUNT(*) FROM aircraft WHERE flown = 0";
        log::debug!("Query: {}", query);

        let mut stmt = self.connection.prepare(query)?;

        let mut cursor = stmt.iter();
        if let Some(result) = cursor.next() {
            let row = result?;
            let count: i64 = row.read(0);
            Ok(count)
        } else {
            Err(sqlite::Error {
                code: Some(sqlite::ffi::SQLITE_ERROR as isize),
                message: Some("No rows returned".to_string()),
            })
        }
    }

    pub fn mark_all_aircraft_unflown(&self) -> Result<(), sqlite::Error> {
        let query = "UPDATE aircraft SET flown = 0, date_flown = NULL";
        log::debug!("Query: {}", query);

        let mut stmt = self.connection.prepare(query)?;
        stmt.next()?;

        Ok(())
    }

    pub fn random_unflown_aircraft(&self) -> Result<Aircraft, sqlite::Error> {
        let query = "SELECT * FROM aircraft WHERE flown = 0 ORDER BY RANDOM() LIMIT 1";
        log::debug!("Query: {}", query);

        let mut stmt = self.connection.prepare(query)?;

        let mut cursor = stmt.iter();
        if let Some(result) = cursor.next() {
            let row = result?;
            let aircraft = Aircraft {
                id: row.read::<i64, _>("id"),
                manufacturer: row.read::<&str, _>("manufacturer").to_string(),
                variant: row.read::<&str, _>("variant").to_string(),
                icao_code: row.read::<&str, _>("icao_code").to_string(),
                flown: row.read::<i64, _>("flown") == 1,
                aircraft_range: row.read::<i64, _>("aircraft_range"),
                category: row.read::<&str, _>("category").to_string(),
                cruise_speed: row.read::<i64, _>("cruise_speed"),
                date_flown: row.read::<Option<&str>, _>("date_flown").map(|s| s.to_string()),
            };
            Ok(aircraft)
        } else {
            Err(sqlite::Error {
                code: Some(sqlite::ffi::SQLITE_ERROR as isize),
                message: Some("No rows returned".to_string()),
            })
        }
    }

    pub fn get_all_aircraft(&self) -> Result<Vec<Aircraft>, sqlite::Error> {
        let mut aircrafts = Vec::new();
        let query = "SELECT * FROM aircraft";
        log::debug!("Query: {}", query);

        let mut stmt = self.connection.prepare(query).unwrap();

        let mut cursor = stmt.iter();
        while let Some(result) = cursor.next() {
            let row = result.unwrap();
            let aircraft = Aircraft {
                id: row.read::<i64, _>("id"),
                manufacturer: row.read::<&str, _>("manufacturer").to_string(),
                variant: row.read::<&str, _>("variant").to_string(),
                icao_code: row.read::<&str, _>("icao_code").to_string(),
                flown: row.read::<i64, _>("flown") == 1,
                aircraft_range: row.read::<i64, _>("aircraft_range"),
                category: row.read::<&str, _>("category").to_string(),
                cruise_speed: row.read::<i64, _>("cruise_speed"),
                date_flown: row
                    .read::<Option<&str>, _>("date_flown")
                    .map(|s| s.to_string()),
            };
            aircrafts.push(aircraft);
        }
        Ok(aircrafts)
    }

    fn add_to_history(
        &self,
        airport: &Airport,
        destination: &Airport,
        aircraft: &Aircraft,
    ) -> Result<(), sqlite::Error> {
        let query = "INSERT INTO history (departure_icao, arrival_icao, aircraft, date) VALUES (?, ?, ?, ?)";
        log::debug!("Query: {}", query);

        let now = chrono::Local::now();
        let date = now.format("%Y-%m-%d").to_string();

        let mut stmt = self.connection.prepare(query)?;
        stmt.bind((1, airport.icao_code.as_str()))?;
        stmt.bind((2, destination.icao_code.as_str()))?;
        stmt.bind((3, aircraft.id))?;
        stmt.bind((4, date.as_str()))?;
        stmt.next()?;
        Ok(())
    }

    fn get_history(&self) -> Result<Vec<History>, sqlite::Error> {
        let mut history = Vec::new();
        let query = "SELECT * FROM history";
        log::debug!("Query: {}", query);

        let mut stmt = self.connection.prepare(query)?;

        let mut cursor = stmt.iter();
        while let Some(result) = cursor.next() {
            let row = result.unwrap();
            let entry = History {
                id: row.read::<i64, _>("id"),
                departure_icao: row.read::<&str, _>("departure_icao").to_string(),
                arrival_icao: row.read::<&str, _>("arrival_icao").to_string(),
                aircraft_id: row.read::<i64, _>("aircraft"),
                date: row.read::<&str, _>("date").to_string(),
            };
            history.push(entry);
        }
        Ok(history)
    }

    fn insert_aircraft(&self, aircraft: &Aircraft) -> Result<(), sqlite::Error> {
        let query = "INSERT INTO aircraft (manufacturer, variant, icao_code, flown, aircraft_range, category, cruise_speed, date_flown) VALUES (?, ?, ?, ?, ?, ?, ?, ?)";
        log::debug!("Query: {}", query);

        let mut stmt = self.connection.prepare(query)?;
        stmt.bind((1, aircraft.manufacturer.as_str()))?;
        stmt.bind((2, aircraft.variant.as_str()))?;
        stmt.bind((3, aircraft.icao_code.as_str()))?;
        stmt.bind((4, if aircraft.flown { 1 } else { 0 }))?;
        stmt.bind((5, aircraft.aircraft_range))?;
        stmt.bind((6, aircraft.category.as_str()))?;
        stmt.bind((7, aircraft.cruise_speed))?;
        if let Some(date) = &aircraft.date_flown {
            stmt.bind((8, date.as_str()))?;
        } else {
            stmt.bind((8, sqlite::Value::Null))?;
        }
        stmt.next()?;

        Ok(())
    }
}
