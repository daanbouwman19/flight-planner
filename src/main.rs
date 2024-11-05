use log;

fn main() {
    env_logger::init();

    let connection = match sqlite::open("data.db") {
        Ok(conn) => {
            log::info!("Database opened successfully");
            conn
        }
        Err(e) => {
            log::error!("Error opening database: {}", e);
            return;
        }
    };

    let airport_connection = match sqlite::open("airports.db3") {
        Ok(conn) => {
            log::info!("Airport database opened successfully");
            conn
        }
        Err(e) => {
            log::error!("Error opening airport database: {}", e);
            return;
        }
    };

    let airport_picker = AirportPicker::new(airport_connection);
    let picker = AircraftPicker::new(connection);
    let count = picker.get_unflown_aircraft_count().unwrap();
    log::info!("Unflown aircraft count: {}", count);
    
    if count > 0 {
        let aircraft = picker.random_unflown_aircraft().unwrap();
        log::info!("Random unflown aircraft: {:?}", aircraft);

        //TODO: Implement the following (runway type and length depending on aircraft)
        let airport = airport_picker
            .get_random_airport_for_aircraft(&aircraft)
            .unwrap();
        log::info!("Random airport: {:?}", airport);

        //TODO select destination airport within aircraft range

        // aircraft.flown = true;
        // log::info!("Marking aircraft as flown: {:?}", aircraft);
        // picker.update_aircraft(&aircraft).unwrap();
    } else {
        log::info!("No unflown aircraft found");
        picker.mark_all_aircraft_unflown().unwrap();
    }
}

pub struct Aircraft {
    pub id: i64,
    pub manufacturer: String,
    pub variant: String,
    pub icao_code: String,
    pub flown: bool,
    pub aircraft_range: i64,
    pub category: String,
    pub cruise_speed: i64,
}

pub struct AircraftPicker {
    pub connection: sqlite::Connection,
}

pub struct AirportPicker {
    pub connection: sqlite::Connection,
}

pub struct Aiport {
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
            .finish()
    }
}

impl std::fmt::Debug for Aiport {
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

impl AirportPicker {
    pub fn new(connection: sqlite::Connection) -> Self {
        AirportPicker { connection }
    }

    //TODO Still no check for runway type and length
    pub fn get_random_airport_for_aircraft(
        &self,
        _aircraft: &Aircraft,
    ) -> Result<Aiport, sqlite::Error> {
        let query = "SELECT * FROM `Airports` ORDER BY RANDOM() LIMIT 1";
        log::debug!("Query: {}", query);

        let mut stmt = self.connection.prepare(query)?;

        let mut cursor = stmt.iter();
        if let Some(result) = cursor.next() {
            let row = result?;
            let airport = Aiport {
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

    pub fn get_random_airport(&self) -> Result<Aiport, sqlite::Error> {
        let query = "SELECT * FROM `Airports` ORDER BY RANDOM() LIMIT 1";
        log::debug!("Query: {}", query);

        let mut stmt = self.connection.prepare(query)?;

        let mut cursor = stmt.iter();
        if let Some(result) = cursor.next() {
            let row = result?;
            let airport = Aiport {
                id: row.read::<i64, _>("ID"),
                name: row.read::<&str, _>("Name").to_string(),
                icao_code: row.read::<&str, _>("ICAO").to_string(),
                latitude: row.read::<f64, _>("Latitude"),
                longtitude: row.read::<f64, _>("Longtitude"),
                elevation: row.read::<i64, _>("Elevation"),
            };
            Ok(airport)
        } else {
            Err(sqlite::Error {
                code: Some(sqlite::ffi::SQLITE_ERROR as isize),
                message: Some("No rows returned".to_string()),
            })
        }
    }

    //TODO pub fn get_desition_airport(&self, aircraft: &Aircraft, airport: &Aiport) {
    // }
}

impl AircraftPicker {
    pub fn new(connection: sqlite::Connection) -> Self {
        AircraftPicker { connection }
    }

    pub fn update_aircraft(&self, aircraft: &Aircraft) -> Result<(), sqlite::Error> {
        let query = "UPDATE aircraft SET manufacturer = ?, variant = ?, icao_code = ?, flown = ?, aircraft_range = ?, category = ?, cruise_speed = ? WHERE id = ?";
        log::debug!("Query: {}", query);

        let mut stmt = self.connection.prepare(query)?;
        stmt.bind((1, aircraft.manufacturer.as_str()))?;
        stmt.bind((2, aircraft.variant.as_str()))?;
        stmt.bind((3, aircraft.icao_code.as_str()))?;
        stmt.bind((4, if aircraft.flown { 1 } else { 0 }))?;
        stmt.bind((5, aircraft.aircraft_range))?;
        stmt.bind((6, aircraft.category.as_str()))?;
        stmt.bind((7, aircraft.cruise_speed))?;
        stmt.bind((8, aircraft.id))?;
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
        let query = "UPDATE aircraft SET flown = 0";
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
            };
            Ok(aircraft)
        } else {
            Err(sqlite::Error {
                code: Some(sqlite::ffi::SQLITE_ERROR as isize),
                message: Some("No rows returned".to_string()),
            })
        }
    }
}
