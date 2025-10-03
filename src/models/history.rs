use crate::schema::history;
use diesel::prelude::*;

/// Represents a flight history record from the database.
///
/// This struct corresponds to the `history` table and is used for querying
/// past flight records.
#[derive(Queryable, Identifiable, Debug, Clone)]
#[diesel(table_name = history)]
pub struct History {
    /// The unique identifier for the history record.
    pub id: i32,
    /// The ICAO code of the departure airport.
    pub departure_icao: String,
    /// The ICAO code of the arrival airport.
    pub arrival_icao: String,
    /// The ID of the aircraft used for the flight, corresponding to the `aircraft` table.
    pub aircraft: i32,
    /// The date of the flight in `YYYY-MM-DD` format.
    pub date: String,
    /// The distance of the flight in nautical miles.
    pub distance: Option<i32>,
}

/// Represents a new flight history record to be inserted into the database.
///
/// This struct is used with Diesel's `insert_into` to create new history records.
#[derive(Insertable)]
#[diesel(table_name = history)]
pub struct NewHistory {
    /// The ICAO code of the departure airport.
    pub departure_icao: String,
    /// The ICAO code of the arrival airport.
    pub arrival_icao: String,
    /// The ID of the aircraft used for the flight.
    pub aircraft: i32,
    /// The date of the flight in `YYYY-MM-DD` format.
    pub date: String,
    /// The distance of the flight in nautical miles.
    pub distance: Option<i32>,
}
