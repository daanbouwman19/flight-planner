use super::list_items::{ListItemAircraft, ListItemAirport, ListItemHistory, ListItemRoute};
use crate::traits::Searchable;
use std::borrow::Cow;

/// An enum that unifies different types of list items for display in a generic table.
///
/// This allows the UI to handle various data types (airports, routes, etc.)
/// polymorphically, simplifying the rendering and data management logic.
#[derive(Clone, Debug, PartialEq)]
pub enum TableItem {
    /// A table item representing an airport.
    Airport(ListItemAirport),
    /// A table item representing a flight route.
    Route(ListItemRoute),
    /// A table item representing a flight history record.
    History(ListItemHistory),
    /// A table item representing an aircraft.
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
                    return 2;
                }
                if [&airport.name, &airport.longest_runway_length]
                    .iter()
                    .any(|f| contains_case_insensitive(f, &query_lower))
                {
                    return 1;
                }
                0
            }
            Self::Route(route) => {
                if [&route.departure.ICAO, &route.destination.ICAO]
                    .iter()
                    .any(|f| contains_case_insensitive(f, &query_lower))
                {
                    return 2;
                }
                if [
                    &route.departure.Name,
                    &route.destination.Name,
                    &route.aircraft.manufacturer,
                    &route.aircraft.variant,
                ]
                .iter()
                .any(|f| contains_case_insensitive(f, &query_lower))
                {
                    return 1;
                }
                0
            }
            Self::History(history) => {
                if [&history.departure_icao, &history.arrival_icao]
                    .iter()
                    .any(|f| contains_case_insensitive(f, &query_lower))
                {
                    return 2;
                }
                if [&history.aircraft_name, &history.date]
                    .iter()
                    .any(|f| contains_case_insensitive(f, &query_lower))
                {
                    return 1;
                }
                0
            }
            Self::Aircraft(aircraft) => {
                if contains_case_insensitive(&aircraft.icao_code, &query_lower) {
                    return 2;
                }
                if [
                    &aircraft.manufacturer,
                    &aircraft.variant,
                    &aircraft.category,
                    &aircraft.date_flown,
                ]
                .iter()
                .any(|f| contains_case_insensitive(f, &query_lower))
                {
                    return 1;
                }
                0
            }
        }
    }
}

impl TableItem {
    /// Returns the appropriate column headers based on the `TableItem` variant.
    ///
    /// This method provides the correct set of headers for rendering the table view,
    /// ensuring the UI adapts to the type of data being displayed.
    ///
    /// # Returns
    ///
    /// A `Vec<&'static str>` containing the column headers.
    pub fn get_columns(&self) -> Vec<&'static str> {
        match self {
            Self::Airport(_) => vec!["Name", "ICAO", "Longest Runway"],
            Self::Route(_) => vec![
                "Departure",
                "ICAO",
                "Runway length",
                "Departure Wind",
                "Destination",
                "ICAO",
                "Runway length",
                "Destination Wind",
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

    /// Returns the data for a single table row, corresponding to the `TableItem` variant.
    ///
    /// The data is returned as a `Vec<Cow<'_, str>>` to avoid unnecessary allocations
    /// for data that can be borrowed.
    ///
    /// # Returns
    ///
    /// A `Vec<Cow<'_, str>>` containing the data for each cell in the row.
    pub fn get_data(&self) -> Vec<Cow<'_, str>> {
        match self {
            Self::Airport(airport) => vec![
                Cow::Borrowed(&airport.name),
                Cow::Borrowed(&airport.icao),
                Cow::Borrowed(&airport.longest_runway_length),
            ],
            Self::Route(route) => {
                let departure_wind = route.departure_metar.as_ref().map_or_else(
                    || Cow::Borrowed("N/A"),
                    |m| Cow::Owned(format!("{:.0} kts", m.wind.speed_kts)),
                );
                let destination_wind = route.destination_metar.as_ref().map_or_else(
                    || Cow::Borrowed("N/A"),
                    |m| Cow::Owned(format!("{:.0} kts", m.wind.speed_kts)),
                );
                vec![
                    Cow::Borrowed(&route.departure.Name),
                    Cow::Borrowed(&route.departure.ICAO),
                    Cow::Borrowed(&route.departure_runway_length),
                    departure_wind,
                    Cow::Borrowed(&route.destination.Name),
                    Cow::Borrowed(&route.destination.ICAO),
                    Cow::Borrowed(&route.destination_runway_length),
                    destination_wind,
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
