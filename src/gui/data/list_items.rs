use crate::models::{Aircraft, Airport};
use std::sync::Arc;

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

/// A structure representing an airport list item.
#[derive(Clone)]
pub struct ListItemAirport {
    /// The name of the airport.
    pub name: String,
    /// The ICAO code of the airport.
    pub icao: String,
    /// The longest runway length in feet.
    pub longest_runway_length: String,
}

impl ListItemAirport {
    /// Creates a new airport list item.
    pub const fn new(name: String, icao: String, longest_runway_length: String) -> Self {
        Self {
            name,
            icao,
            longest_runway_length,
        }
    }
}
