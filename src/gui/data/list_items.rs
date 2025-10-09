use crate::date_utils;
use crate::models::{Aircraft, Airport};
use crate::modules::weather::Metar;
use std::sync::Arc;

/// Represents a flight route displayed as an item in a list or table.
///
/// This struct aggregates all the necessary information for a single route,
/// including departure and destination details, the assigned aircraft, and
/// relevant runway and distance data.
#[derive(Clone, Debug, PartialEq)]
pub struct ListItemRoute {
    /// A shared pointer to the departure `Airport`.
    pub departure: Arc<Airport>,
    /// A shared pointer to the destination `Airport`.
    pub destination: Arc<Airport>,
    /// A shared pointer to the `Aircraft` assigned to the route.
    pub aircraft: Arc<Aircraft>,
    /// A string representing the length of the longest runway at the departure airport.
    pub departure_runway_length: String,
    /// A string representing the length of the longest runway at the destination airport.
    pub destination_runway_length: String,
    /// The total length of the route in nautical miles, formatted as a string.
    pub route_length: String,
    /// The METAR data for the departure airport, if available.
    pub departure_metar: Option<Metar>,
    /// The METAR data for the destination airport, if available.
    pub destination_metar: Option<Metar>,
}

/// Represents a flight history record formatted for display in the UI.
///
/// This struct contains denormalized data, such as airport and aircraft names,
/// to avoid additional lookups during rendering.
#[derive(Clone, Debug, PartialEq)]
pub struct ListItemHistory {
    /// The unique identifier of the history record, as a string.
    pub id: String,
    /// The ICAO code of the departure airport.
    pub departure_icao: String,
    /// The full name of the departure airport.
    pub departure_airport_name: String,
    /// The ICAO code of the arrival airport.
    pub arrival_icao: String,
    /// The full name of the arrival airport.
    pub arrival_airport_name: String,
    /// The name of the aircraft used for the flight (e.g., "Boeing 737-800").
    pub aircraft_name: String,
    /// The date of the flight, formatted as a string.
    pub date: String,
}

/// Represents an airport formatted for display in a list.
#[derive(Clone, Debug, PartialEq)]
pub struct ListItemAirport {
    /// The full name of the airport.
    pub name: String,
    /// The ICAO code of the airport.
    pub icao: String,
    /// The length of the longest runway in feet, formatted as a string.
    pub longest_runway_length: String,
}

impl ListItemAirport {
    /// Creates a new `ListItemAirport`.
    ///
    /// # Arguments
    ///
    /// * `name` - The full name of the airport.
    /// * `icao` - The ICAO code of the airport.
    /// * `longest_runway_length` - The formatted string for the longest runway length.
    pub const fn new(name: String, icao: String, longest_runway_length: String) -> Self {
        Self {
            name,
            icao,
            longest_runway_length,
        }
    }
}

/// Represents an aircraft formatted for display in a list or table.
///
/// This struct holds formatted strings for various aircraft properties, making
/// it suitable for direct rendering in the UI without additional processing.
#[derive(Clone, Debug, PartialEq)]
pub struct ListItemAircraft {
    /// The unique identifier of the aircraft.
    pub id: i32,
    /// The name of the aircraft's manufacturer.
    pub manufacturer: String,
    /// The specific model or variant of the aircraft.
    pub variant: String,
    /// The ICAO code for the aircraft type.
    pub icao_code: String,
    /// A flag indicating if the aircraft has been flown (1 for true, 0 for false).
    pub flown: i32,
    /// The operational range of the aircraft, formatted as a string (e.g., "3000 NM").
    pub range: String,
    /// The category of the aircraft.
    pub category: String,
    /// The cruise speed of the aircraft, formatted as a string (e.g., "450 knots").
    pub cruise_speed: String,
    /// The date the aircraft was last flown, formatted for display (e.g., "YYYY-MM-DD" or "Never").
    pub date_flown: String,
}

impl ListItemAircraft {
    /// Creates a new `ListItemAircraft` from an `Aircraft` model.
    ///
    /// This function handles the conversion and formatting of `Aircraft` data
    /// into a display-ready format.
    ///
    /// # Arguments
    ///
    /// * `aircraft` - A shared pointer to the `Aircraft` model.
    pub fn new(aircraft: &Arc<Aircraft>) -> Self {
        let date_display = date_utils::format_date_for_display(aircraft.date_flown.as_ref());

        Self {
            id: aircraft.id,
            manufacturer: aircraft.manufacturer.clone(),
            variant: aircraft.variant.clone(),
            icao_code: aircraft.icao_code.clone(),
            flown: aircraft.flown,
            range: format!("{} NM", aircraft.aircraft_range),
            category: aircraft.category.clone(),
            cruise_speed: format!("{} knots", aircraft.cruise_speed),
            date_flown: date_display,
        }
    }
}
