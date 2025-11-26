use crate::schema::aircraft;
use diesel::prelude::*;

/// Represents an aircraft record from the database.
///
/// This struct corresponds to the `aircraft` table and is used for querying,
/// updating, and identifying aircraft records.
#[derive(Queryable, Debug, PartialEq, Eq, Clone, Identifiable, AsChangeset)]
#[diesel(table_name = aircraft)]
#[diesel(treat_none_as_null = true)]
#[allow(clippy::struct_field_names)]
pub struct Aircraft {
    /// The unique identifier for the aircraft.
    pub id: i32,
    /// The manufacturer of the aircraft (e.g., "Boeing").
    pub manufacturer: String,
    /// The specific model or variant of the aircraft (e.g., "737-800").
    pub variant: String,
    /// The ICAO code for the aircraft type (e.g., "B738").
    pub icao_code: String,
    /// A flag indicating whether the aircraft has been flown (1 for true, 0 for false).
    pub flown: i32,
    /// The operational range of the aircraft in nautical miles.
    pub aircraft_range: i32,
    /// The category of the aircraft (e.g., "A", "B", "C").
    pub category: String,
    /// The typical cruise speed of the aircraft in knots.
    pub cruise_speed: i32,
    /// The date the aircraft was last flown, in `YYYY-MM-DD` format.
    pub date_flown: Option<String>,
    /// The required takeoff distance in meters.
    pub takeoff_distance: Option<i32>,
}

/// Represents a new aircraft record to be inserted into the database.
///
/// This struct is used with Diesel's `insert_into` to create new aircraft records.
#[derive(Insertable, Debug, PartialEq, Eq)]
#[diesel(table_name = aircraft)]
pub struct NewAircraft {
    /// The manufacturer of the aircraft.
    pub manufacturer: String,
    /// The model or variant of the aircraft.
    pub variant: String,
    /// The ICAO code for the aircraft type.
    pub icao_code: String,
    /// The initial flown status (usually 0 for not flown).
    pub flown: i32,
    /// The operational range of the aircraft in nautical miles.
    pub aircraft_range: i32,
    /// The category of the aircraft.
    pub category: String,
    /// The typical cruise speed in knots.
    pub cruise_speed: i32,
    /// The date the aircraft was flown. `None` if not yet flown.
    pub date_flown: Option<String>,
    /// The required takeoff distance in meters.
    pub takeoff_distance: Option<i32>,
}
