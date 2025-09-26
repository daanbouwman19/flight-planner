use super::list_items::{ListItemAircraft, ListItemAirport, ListItemHistory, ListItemRoute};
use crate::traits::Searchable;
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

impl Searchable for TableItem {
    /// Returns a score indicating how well the item matches the query.
    /// A higher score indicates a better match.
    /// A score of 0 indicates no match.
    fn search_score(&self, query: &str) -> u8 {
        let query_lower = query.to_lowercase();

        match self {
            Self::Airport(airport) => {
                if contains_case_insensitive(&airport.icao, &query_lower) {
                    2
                } else if contains_case_insensitive(&airport.name, &query_lower) {
                    1
                } else {
                    0
                }
            }
            Self::Route(route) => {
                if contains_case_insensitive(&route.departure.ICAO, &query_lower)
                    || contains_case_insensitive(&route.destination.ICAO, &query_lower)
                {
                    2
                } else if contains_case_insensitive(&route.departure.Name, &query_lower)
                    || contains_case_insensitive(&route.destination.Name, &query_lower)
                    || contains_case_insensitive(&route.aircraft.manufacturer, &query_lower)
                    || contains_case_insensitive(&route.aircraft.variant, &query_lower)
                {
                    1
                } else {
                    0
                }
            }
            Self::History(history) => {
                if contains_case_insensitive(&history.departure_icao, &query_lower)
                    || contains_case_insensitive(&history.arrival_icao, &query_lower)
                {
                    2
                } else if contains_case_insensitive(&history.aircraft_name, &query_lower)
                    || contains_case_insensitive(&history.date, &query_lower)
                {
                    1
                } else {
                    0
                }
            }
            Self::Aircraft(aircraft) => {
                if contains_case_insensitive(&aircraft.icao_code, &query_lower) {
                    2
                } else if contains_case_insensitive(&aircraft.manufacturer, &query_lower)
                    || contains_case_insensitive(&aircraft.variant, &query_lower)
                    || contains_case_insensitive(&aircraft.category, &query_lower)
                    || contains_case_insensitive(&aircraft.date_flown, &query_lower)
                {
                    1
                } else {
                    0
                }
            }
        }
    }
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
}

/// Optimized case-insensitive substring search that minimizes allocations.
/// For ASCII text (the vast majority of cases), uses zero-allocation comparison.
/// For Unicode text, falls back to correct but allocating comparison.
/// Assumes `query_lower` is already lowercase for optimal performance.
fn contains_case_insensitive(haystack: &str, query_lower: &str) -> bool {
    // Fast path: if query is empty, always matches
    if query_lower.is_empty() {
        return true;
    }

    // Optimization: if both haystack and query are pure ASCII, use fast non-allocating path
    if haystack.is_ascii() && query_lower.is_ascii() {
        // Convert to bytes for efficient ASCII comparison
        let haystack_bytes = haystack.as_bytes();
        let query_bytes = query_lower.as_bytes();

        if query_bytes.len() > haystack_bytes.len() {
            return false;
        }

        // Idiomatic sliding window search using `windows` and `eq_ignore_ascii_case`
        haystack_bytes
            .windows(query_bytes.len())
            .any(|window| window.eq_ignore_ascii_case(query_bytes))
    } else {
        // Unicode fallback: correct but allocating for complex cases like Turkish İ
        haystack.to_lowercase().contains(query_lower)
    }
}
