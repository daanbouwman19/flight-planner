use super::list_items::{ListItemAircraft, ListItemAirport, ListItemHistory, ListItemRoute};
use std::borrow::Cow;

/// An enum representing the items that can be displayed in the table.
#[derive(Clone, Debug, PartialEq)]
pub enum TableItem {
    /// Represents an airport item.
    Airport(ListItemAirport),
    /// Represents a route item.
    Route(ListItemRoute),
    /// Represents a history item.
    History(ListItemHistory),
    /// Represents an aircraft item.
    Aircraft(ListItemAircraft),
}

impl TableItem {
    /// Returns the column headers for the table item.
    pub fn get_columns(&self) -> Vec<&'static str> {
        match self {
            Self::Airport(_) => vec!["Name", "ICAO", "Longest Runway"],
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
                "Actions",
            ],
            Self::History(_) => vec!["ID", "Departure", "Arrival", "Aircraft", "Date"],
            Self::Aircraft(_) => vec![
                "Manufacturer",
                "Variant",
                "ICAO",
                "Range",
                "Category",
                "Cruise Speed",
                "Date Flown",
                "Action",
            ],
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
                    // Actions column is handled separately in the table component
                    Cow::Borrowed(""),
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
            Self::Aircraft(aircraft) => {
                vec![
                    Cow::Borrowed(&aircraft.manufacturer),
                    Cow::Borrowed(&aircraft.variant),
                    Cow::Borrowed(&aircraft.icao_code),
                    Cow::Borrowed(&aircraft.range),
                    Cow::Borrowed(&aircraft.category),
                    Cow::Borrowed(&aircraft.cruise_speed),
                    Cow::Borrowed(&aircraft.date_flown),
                    // Action column is handled separately in the table component
                    Cow::Borrowed(""),
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
        let query_lower = query.to_lowercase();
        self.matches_query_lower(&query_lower)
    }

    /// Optimized version that takes a pre-lowercased query to avoid repeated allocations.
    pub fn matches_query_lower(&self, query_lower: &str) -> bool {
        // Use Unicode-aware case-insensitive search with pre-lowercased query for optimization
        match self {
            Self::Airport(airport) => {
                airport.name.to_lowercase().contains(query_lower)
                    || airport.icao.to_lowercase().contains(query_lower)
                    || airport
                        .longest_runway_length
                        .to_lowercase()
                        .contains(query_lower)
            }
            Self::Route(route) => {
                route.departure.Name.to_lowercase().contains(query_lower)
                    || route.departure.ICAO.to_lowercase().contains(query_lower)
                    || route.destination.Name.to_lowercase().contains(query_lower)
                    || route.destination.ICAO.to_lowercase().contains(query_lower)
                    || route
                        .aircraft
                        .manufacturer
                        .to_lowercase()
                        .contains(query_lower)
                    || route.aircraft.variant.to_lowercase().contains(query_lower)
            }
            Self::History(history) => {
                history.departure_icao.to_lowercase().contains(query_lower)
                    || history.arrival_icao.to_lowercase().contains(query_lower)
                    || history.aircraft_name.to_lowercase().contains(query_lower)
                    || history.date.to_lowercase().contains(query_lower)
            }
            Self::Aircraft(aircraft) => {
                aircraft.manufacturer.to_lowercase().contains(query_lower)
                    || aircraft.variant.to_lowercase().contains(query_lower)
                    || aircraft.icao_code.to_lowercase().contains(query_lower)
                    || aircraft.category.to_lowercase().contains(query_lower)
                    || aircraft.date_flown.to_lowercase().contains(query_lower)
            }
        }
    }
}
