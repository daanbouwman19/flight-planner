use super::list_items::{ListItemAircraft, ListItemAirport, ListItemHistory, ListItemRoute};
use std::borrow::Cow;

/// An enum representing the items that can be displayed in the table.
pub enum TableItem {
    /// Represents an airport item.
    Airport(ListItemAirport),
    /// Represents an aircraft item.
    Aircraft(ListItemAircraft),
    /// Represents a route item.
    Route(ListItemRoute),
    /// Represents a history item.
    History(ListItemHistory),
}

impl TableItem {
    /// Returns the column headers for the table item.
    pub fn get_columns(&self) -> Vec<&'static str> {
        match self {
            Self::Airport(_) => vec!["Name", "ICAO", "Longest Runway"],
            Self::Aircraft(_) => vec!["ID", "Model", "Registration", "Flown"],
            Self::Route(_) => vec![
                "Departure",
                "ICAO",
                "Runway length",
                "Destination",
                "ICAO",
                "Runway length",
                "Manufacturer",
                "Aircraft",
                "Distance",
            ],
            Self::History(_) => vec!["ID", "Departure", "Arrival", "Aircraft", "Date"],
        }
    }

    /// Returns the data for the table item.
    pub fn get_data(&self) -> Vec<Cow<'_, str>> {
        match self {
            Self::Airport(airport) => vec![
                Cow::Borrowed(&airport.name),
                Cow::Borrowed(&airport.icao),
                Cow::Borrowed(&airport.longest_runway_length),
            ],
            Self::Aircraft(aircraft) => vec![
                Cow::Borrowed(aircraft.get_id()),
                Cow::Borrowed(aircraft.get_manufacturer()),
                Cow::Borrowed(aircraft.get_variant()),
                Cow::Borrowed(aircraft.get_flown()),
            ],
            Self::Route(route) => {
                vec![
                    Cow::Borrowed(&route.departure.Name),
                    Cow::Borrowed(&route.departure.ICAO),
                    Cow::Borrowed(&route.departure_runway_length),
                    Cow::Borrowed(&route.destination.Name),
                    Cow::Borrowed(&route.destination.ICAO),
                    Cow::Borrowed(&route.destination_runway_length),
                    Cow::Borrowed(&route.aircraft.manufacturer),
                    Cow::Borrowed(&route.aircraft.variant),
                    Cow::Borrowed(&route.route_length),
                ]
            }
            Self::History(history) => {
                vec![
                    Cow::Borrowed(&history.id),
                    Cow::Borrowed(&history.departure_icao),
                    Cow::Borrowed(&history.arrival_icao),
                    Cow::Borrowed(&history.aircraft_name),
                    Cow::Borrowed(&history.date),
                ]
            }
        }
    }

    /// Checks if the item matches the search query.
    ///
    /// # Arguments
    ///
    /// * `query` - The search query string.
    pub fn matches_query(&self, query: &str) -> bool {
        let query = query.to_lowercase();
        match self {
            Self::Airport(airport) => {
                airport.name.to_lowercase().contains(&query)
                    || airport.icao.to_lowercase().contains(&query)
                    || airport
                        .longest_runway_length
                        .to_lowercase()
                        .contains(&query)
            }
            Self::Aircraft(aircraft) => {
                aircraft.get_variant().to_lowercase().contains(&query)
                    || aircraft.get_manufacturer().to_lowercase().contains(&query)
                    || aircraft.get_id().to_string().contains(&query)
            }
            Self::Route(route) => {
                route.departure.Name.to_lowercase().contains(&query)
                    || route.departure.ICAO.to_lowercase().contains(&query)
                    || route.destination.Name.to_lowercase().contains(&query)
                    || route.destination.ICAO.to_lowercase().contains(&query)
                    || route.aircraft.manufacturer.to_lowercase().contains(&query)
                    || route.aircraft.variant.to_lowercase().contains(&query)
            }
            Self::History(_) => false,
        }
    }
}
