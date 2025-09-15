use crate::date_utils;
use crate::models::{Aircraft, Airport};
use std::sync::Arc;

pub trait ListItem {
    fn get_headers(&self) -> Vec<&'static str>;
    fn get_values(&self) -> Vec<String>;
}

/// A structure representing a flight route.
#[derive(Clone, Debug)]
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

impl ListItem for ListItemRoute {
    fn get_headers(&self) -> Vec<&'static str> {
        vec!["Departure", "Arrival", "Aircraft", "Distance"]
    }

    fn get_values(&self) -> Vec<String> {
        vec![
            self.departure.ICAO.clone(),
            self.destination.ICAO.clone(),
            self.aircraft.variant.clone(),
            self.route_length.clone(),
        ]
    }
}

/// A structure representing a flight history item.
#[derive(Clone, Debug, PartialEq)]
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

impl ListItem for ListItemHistory {
    fn get_headers(&self) -> Vec<&'static str> {
        vec!["Departure", "Arrival", "Aircraft", "Date"]
    }

    fn get_values(&self) -> Vec<String> {
        vec![
            self.departure_icao.clone(),
            self.arrival_icao.clone(),
            self.aircraft_name.clone(),
            self.date.clone(),
        ]
    }
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

impl ListItem for ListItemAirport {
    fn get_headers(&self) -> Vec<&'static str> {
        vec!["Name", "ICAO", "Longest Runway"]
    }

    fn get_values(&self) -> Vec<String> {
        vec![
            self.name.clone(),
            self.icao.clone(),
            self.longest_runway_length.clone(),
        ]
    }
}

/// A structure representing an aircraft list item.
#[derive(Clone)]
pub struct ListItemAircraft {
    /// The aircraft ID.
    pub id: i32,
    /// The manufacturer name.
    pub manufacturer: String,
    /// The variant name.
    pub variant: String,
    /// The ICAO code.
    pub icao_code: String,
    /// Whether the aircraft has been flown.
    pub flown: i32,
    /// The aircraft range.
    pub range: String,
    /// The aircraft category.
    pub category: String,
    /// The cruise speed.
    pub cruise_speed: String,
    /// The date flown (if any).
    pub date_flown: String,
}

impl ListItemAircraft {
    /// Creates a new aircraft list item.
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

impl ListItem for ListItemAircraft {
    fn get_headers(&self) -> Vec<&'static str> {
        vec![
            "Manufacturer",
            "Variant",
            "ICAO",
            "Range",
            "Category",
            "Cruise Speed",
            "Flown",
            "Date Flown",
        ]
    }

    fn get_values(&self) -> Vec<String> {
        vec![
            self.manufacturer.clone(),
            self.variant.clone(),
            self.icao_code.clone(),
            self.range.clone(),
            self.category.clone(),
            self.cruise_speed.clone(),
            if self.flown == 1 { "Yes" } else { "No" }.to_string(),
            self.date_flown.clone(),
        ]
    }
}
