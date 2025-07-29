use std::sync::Arc;
use crate::models::{Aircraft, Airport};

/// A structure representing a flight route.
#[derive(Clone)]
pub struct ListItemRoute {
    /// The departure airport.
    pub departure: Arc<Airport>,
    /// The destination airport.
    pub destination: Arc<Airport>,
    /// The aircraft used for the route.
    pub aircraft: Arc<Aircraft>,
    /// The departure runways.
    pub departure_runway_length: String,
    /// The destination runways.
    pub destination_runway_length: String,
    /// route length
    pub route_length: String,
}

/// A structure representing a flight history item.
#[derive(Clone)]
pub struct ListItemHistory {
    /// The ID of the history item.
    pub id: String,
    /// The departure ICAO code.
    pub departure_icao: String,
    /// The arrival ICAO code.
    pub arrival_icao: String,
    /// The aircraft ID.
    pub aircraft_name: String,
    /// The date of the flight.
    pub date: String,
}

/// A structure representing an aircraft list item.
#[derive(Clone)]
pub struct ListItemAircraft {
    /// The ID of the aircraft.
    id: String,
    /// The variant of the aircraft.
    variant: String,
    /// The manufacturer of the aircraft.
    manufacturer: String,
    /// The number of times the aircraft has been flown.
    flown: String,
}

impl ListItemAircraft {
    /// Creates a new `ListItemAircraft` from an `Aircraft`.
    ///
    /// # Arguments
    ///
    /// * `aircraft` - The aircraft to convert.
    pub fn from_aircraft(aircraft: &Aircraft) -> Self {
        Self {
            id: aircraft.id.to_string(),
            variant: aircraft.variant.clone(),
            manufacturer: aircraft.manufacturer.clone(),
            flown: if aircraft.flown > 0 { "true".to_string() } else { "false".to_string() },
        }
    }

    /// Gets the aircraft ID.
    pub fn get_id(&self) -> &str {
        &self.id
    }

    /// Gets the aircraft variant.
    pub fn get_variant(&self) -> &str {
        &self.variant
    }

    /// Gets the aircraft manufacturer.
    pub fn get_manufacturer(&self) -> &str {
        &self.manufacturer
    }

    /// Gets whether the aircraft has been flown.
    pub fn get_flown(&self) -> &str {
        &self.flown
    }
}

/// A structure representing an airport list item.
#[derive(Clone)]
pub struct ListItemAirport {
    /// The ID of the airport.
    pub id: String,
    /// The name of the airport.
    pub name: String,
    /// The ICAO code of the airport.
    pub icao: String,
}

impl ListItemAirport {
    /// Creates a new airport list item.
    pub const fn new(id: String, name: String, icao: String) -> Self {
        Self { id, name, icao }
    }
}
