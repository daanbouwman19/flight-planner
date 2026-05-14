#[cfg(not(target_arch = "wasm32"))]
use crate::schema::history;
#[cfg(not(target_arch = "wasm32"))]
use diesel::prelude::*;

/// Represents a flight history record from the database.
///
/// This struct corresponds to the `history` table and is used for querying
/// past flight records.
#[cfg_attr(not(target_arch = "wasm32"), derive(Queryable, Identifiable))]
#[cfg_attr(not(target_arch = "wasm32"), diesel(table_name = history))]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
#[cfg_attr(not(target_arch = "wasm32"), derive(Insertable))]
#[cfg_attr(not(target_arch = "wasm32"), diesel(table_name = history))]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
