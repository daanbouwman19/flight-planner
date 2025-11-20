use diesel::prelude::*;
use diesel::result::Error;

use crate::database::{DatabaseConnections, DatabasePool};
use crate::models::Aircraft;
use crate::schema::aircraft::dsl::{aircraft, date_flown, flown};
use crate::traits::AircraftOperations;
use crate::util::random;

use crate::errors::Error as AppError;
use crate::models::NewAircraft;
use serde::de;
use std::fmt;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

/// Serde deserialization decorator to trim whitespace from strings.
fn trim_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: de::Deserializer<'de>,
{
    struct StringVisitor;

    impl<'de> de::Visitor<'de> for StringVisitor {
        type Value = String;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a string")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(v.trim().to_string())
        }
    }

    deserializer.deserialize_string(StringVisitor)
}

#[derive(Debug, serde::Deserialize)]
struct CsvAircraftRecord {
    #[serde(deserialize_with = "trim_string")]
    manufacturer: String,
    #[serde(deserialize_with = "trim_string")]
    variant: String,
    icao_code: String,
    flown: i32,
    aircraft_range: i32,
    category: String,
    cruise_speed: i32,
    #[serde(default)]
    date_flown: Option<String>,
    #[serde(default)]
    takeoff_distance: Option<i32>,
}

impl From<CsvAircraftRecord> for NewAircraft {
    fn from(r: CsvAircraftRecord) -> Self {
        NewAircraft {
            manufacturer: r.manufacturer,
            variant: r.variant,
            icao_code: r.icao_code,
            flown: r.flown,
            aircraft_range: r.aircraft_range,
            category: r.category,
            cruise_speed: r.cruise_speed,
            date_flown: r.date_flown,
            takeoff_distance: r.takeoff_distance,
        }
    }
}

/// Imports aircraft from a CSV file into the database if the aircraft table is empty.
///
/// This function reads aircraft data from the given CSV file and inserts it into
/// the database. The import is performed within a transaction to ensure atomicity.
/// If the aircraft table already contains data, the import is skipped.
///
/// Malformed rows in the CSV file are skipped, and a warning is logged.
///
/// # Arguments
///
/// * `conn` - A mutable reference to an `SqliteConnection`.
/// * `csv_path` - The path to the CSV file to import.
///
/// # Returns
///
/// * `Ok(true)` if the import was successfully performed.
/// * `Ok(false)` if the import was not needed (e.g., the table was not empty or the CSV had no data).
/// * `Err(AppError)` if an error occurs during file I/O or database operations.
pub fn import_aircraft_from_csv_if_empty(
    conn: &mut SqliteConnection,
    csv_path: &Path,
) -> Result<bool, AppError> {
    use crate::schema::aircraft::dsl::aircraft as aircraft_table;

    // Check if table is empty BEFORE doing any file I/O
    let count: i64 = aircraft_table.count().get_result(conn)?;
    if count > 0 {
        return Ok(false);
    }

    // Read CSV (I/O outside transaction)
    let file = File::open(csv_path)?;
    let reader = BufReader::new(file);
    let mut rdr = csv::Reader::from_reader(reader);

    // Collect inserts (skip malformed rows with a warning)
    let new_records: Vec<NewAircraft> = rdr
        .deserialize::<CsvAircraftRecord>()
        .filter_map(|res| match res {
            Ok(rec) => Some(rec.into()),
            Err(err) => {
                log::warn!("Skipping malformed CSV row: {err}");
                None
            }
        })
        .collect();

    if new_records.is_empty() {
        return Ok(false);
    }

    // Perform insert atomically
    let imported = conn.transaction::<bool, diesel::result::Error, _>(|tx| {
        // Double-check count inside transaction to be safe against races
        let count: i64 = aircraft_table.count().get_result(tx)?;
        if count > 0 {
            return Ok(false);
        }

        diesel::insert_into(aircraft_table)
            .values(&new_records)
            .execute(tx)?;

        Ok(true)
    })?;

    if imported {
        log::info!(
            "Imported {} aircraft from CSV at {}",
            new_records.len(),
            csv_path.display()
        );
    }

    Ok(imported)
}

impl AircraftOperations for DatabaseConnections {
    fn get_not_flown_count(&mut self) -> Result<i64, Error> {
        let count: i64 = aircraft
            .filter(flown.eq(0))
            .count()
            .get_result(&mut self.aircraft_connection)?;

        Ok(count)
    }

    fn random_not_flown_aircraft(&mut self) -> Result<Aircraft, Error> {
        let record: Aircraft = aircraft
            .filter(flown.eq(0))
            .order(random())
            .limit(1)
            .get_result(&mut self.aircraft_connection)?;

        Ok(record)
    }

    fn get_all_aircraft(&mut self) -> Result<Vec<Aircraft>, Error> {
        let records: Vec<Aircraft> = aircraft.load(&mut self.aircraft_connection)?;

        Ok(records)
    }

    fn update_aircraft(&mut self, record: &Aircraft) -> Result<(), Error> {
        diesel::update(aircraft.find(record.id))
            .set(record)
            .execute(&mut self.aircraft_connection)?;

        Ok(())
    }

    fn random_aircraft(&mut self) -> Result<Aircraft, Error> {
        let record: Aircraft = aircraft
            .order(random())
            .limit(1)
            .get_result(&mut self.aircraft_connection)?;

        Ok(record)
    }

    fn get_aircraft_by_id(&mut self, aircraft_id: i32) -> Result<Aircraft, Error> {
        if aircraft_id < 1 {
            return Err(Error::NotFound);
        }

        let record: Aircraft = aircraft
            .find(aircraft_id)
            .get_result(&mut self.aircraft_connection)?;
        Ok(record)
    }

    fn mark_all_aircraft_not_flown(&mut self) -> Result<(), Error> {
        mark_all_aircraft_not_flown(&mut self.aircraft_connection)
    }

    fn add_aircraft(&mut self, record: &NewAircraft) -> Result<Aircraft, Error> {
        add_aircraft(record, &mut self.aircraft_connection)
    }
}

impl AircraftOperations for DatabasePool {
    fn get_not_flown_count(&mut self) -> Result<i64, Error> {
        let conn = &mut self.aircraft_pool.get().unwrap();
        let count: i64 = aircraft.filter(flown.eq(0)).count().get_result(conn)?;

        Ok(count)
    }

    fn random_not_flown_aircraft(&mut self) -> Result<Aircraft, Error> {
        let conn = &mut self.aircraft_pool.get().unwrap();
        let record: Aircraft = aircraft
            .filter(flown.eq(0))
            .order(random())
            .limit(1)
            .get_result(conn)?;

        Ok(record)
    }

    fn get_all_aircraft(&mut self) -> Result<Vec<Aircraft>, Error> {
        let conn = &mut self.aircraft_pool.get().unwrap();
        let records: Vec<Aircraft> = aircraft.load(conn)?;

        Ok(records)
    }

    fn update_aircraft(&mut self, record: &Aircraft) -> Result<(), Error> {
        let conn = &mut self.aircraft_pool.get().unwrap();

        // Use explicit field updates instead of .set(record) to handle NULL values properly
        diesel::update(aircraft.find(record.id))
            .set((flown.eq(record.flown), date_flown.eq(&record.date_flown)))
            .execute(conn)?;

        Ok(())
    }

    fn random_aircraft(&mut self) -> Result<Aircraft, Error> {
        let conn = &mut self.aircraft_pool.get().unwrap();
        let record: Aircraft = aircraft.order(random()).limit(1).get_result(conn)?;

        Ok(record)
    }

    fn get_aircraft_by_id(&mut self, aircraft_id: i32) -> Result<Aircraft, Error> {
        if aircraft_id < 1 {
            return Err(Error::NotFound);
        }

        let conn = &mut self.aircraft_pool.get().unwrap();
        let record: Aircraft = aircraft.find(aircraft_id).get_result(conn)?;
        Ok(record)
    }

    fn mark_all_aircraft_not_flown(&mut self) -> Result<(), Error> {
        mark_all_aircraft_not_flown(&mut self.aircraft_pool.get().unwrap())
    }

    fn add_aircraft(&mut self, record: &NewAircraft) -> Result<Aircraft, Error> {
        let conn = &mut self.aircraft_pool.get().unwrap();
        add_aircraft(record, conn)
    }
}

fn mark_all_aircraft_not_flown(conn: &mut SqliteConnection) -> Result<(), Error> {
    diesel::update(aircraft)
        .set((flown.eq(0), date_flown.eq(None::<String>)))
        .execute(conn)?;

    Ok(())
}

fn add_aircraft(record: &NewAircraft, conn: &mut SqliteConnection) -> Result<Aircraft, Error> {
    use crate::schema::aircraft::dsl::id;

    diesel::insert_into(aircraft).values(record).execute(conn)?;
    let inserted_aircraft: Aircraft = aircraft.order(id.desc()).first(conn)?;

    Ok(inserted_aircraft)
}

/// Finds the path to the `aircrafts.csv` file.
///
/// This function searches for `aircrafts.csv` in the following locations:
/// 1. The application data directory.
/// 2. The current working directory.
/// 3. The system-wide shared data directory.
///
/// # Returns
///
/// An `Option<PathBuf>` containing the path to the file if found, or `None`.
pub fn find_aircraft_csv_path() -> Option<PathBuf> {
    let candidates = crate::get_aircraft_csv_candidate_paths();
    for path in candidates {
        if path.exists() {
            return Some(path);
        }
    }
    None
}

/// Formats an `Aircraft` struct into a human-readable string.
///
/// # Arguments
///
/// * `ac` - A reference to the `Aircraft` struct to format.
///
/// # Returns
///
/// A `String` containing the formatted aircraft details.
pub fn format_aircraft(ac: &Aircraft) -> String {
    format!(
        "id: {}, {} {}{}, range: {}, category: {}, cruise speed: {} knots, takeoff distance: {}",
        ac.id,
        ac.manufacturer,
        ac.variant,
        if ac.icao_code.is_empty() {
            String::new()
        } else {
            format!(" ({})", ac.icao_code)
        },
        ac.aircraft_range,
        ac.category,
        ac.cruise_speed,
        ac.takeoff_distance
            .map_or("unknown".to_string(), |d| format!("{d} m")),
    )
}
