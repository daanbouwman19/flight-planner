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
        // Use a more efficient case-insensitive search that avoids string allocations
        match self {
            Self::Airport(airport) => {
                contains_ignore_ascii_case(&airport.name, query_lower)
                    || contains_ignore_ascii_case(&airport.icao, query_lower)
                    || contains_ignore_ascii_case(&airport.longest_runway_length, query_lower)
            }
            Self::Route(route) => {
                contains_ignore_ascii_case(&route.departure.Name, query_lower)
                    || contains_ignore_ascii_case(&route.departure.ICAO, query_lower)
                    || contains_ignore_ascii_case(&route.destination.Name, query_lower)
                    || contains_ignore_ascii_case(&route.destination.ICAO, query_lower)
                    || contains_ignore_ascii_case(&route.aircraft.manufacturer, query_lower)
                    || contains_ignore_ascii_case(&route.aircraft.variant, query_lower)
            }
            Self::History(history) => {
                contains_ignore_ascii_case(&history.departure_icao, query_lower)
                    || contains_ignore_ascii_case(&history.arrival_icao, query_lower)
                    || contains_ignore_ascii_case(&history.aircraft_name, query_lower)
                    || contains_ignore_ascii_case(&history.date, query_lower)
            }
            Self::Aircraft(aircraft) => {
                contains_ignore_ascii_case(&aircraft.manufacturer, query_lower)
                    || contains_ignore_ascii_case(&aircraft.variant, query_lower)
                    || contains_ignore_ascii_case(&aircraft.icao_code, query_lower)
                    || contains_ignore_ascii_case(&aircraft.category, query_lower)
                    || contains_ignore_ascii_case(&aircraft.date_flown, query_lower)
            }
        }
    }
}

/// Efficient case-insensitive contains check without string allocations
fn contains_ignore_ascii_case(haystack: &str, needle_lower: &str) -> bool {
    if needle_lower.is_empty() {
        return true;
    }
    if haystack.len() < needle_lower.len() {
        return false;
    }

    haystack
        .as_bytes()
        .windows(needle_lower.len())
        .any(|window| {
            window
                .iter()
                .zip(needle_lower.bytes())
                .all(|(a, b)| a.to_ascii_lowercase() == b)
        })
}
