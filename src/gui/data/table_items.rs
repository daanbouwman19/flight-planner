use super::list_items::{ListItemAircraft, ListItemAirport, ListItemHistory, ListItemRoute};
use crate::traits::Searchable;
use crate::util::contains_case_insensitive_optimized;
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
    /// Returns a score indicating how well the item matches the lowercased query.
    fn search_score_lower(&self, query_lower: &str) -> u8 {
        // Fallback to optimized implementation with is_ascii check
        self.search_score_optimized(query_lower, query_lower.is_ascii())
    }

    /// Returns a score indicating how well the item matches the lowercased query.
    /// Optimized version that takes a pre-calculated is_ascii flag.
    fn search_score_optimized(&self, query_lower: &str, is_ascii: bool) -> u8 {
        match self {
            Self::Airport(airport) => {
                if contains_case_insensitive_optimized(&airport.icao, query_lower, is_ascii) {
                    return 2;
                }
                if contains_case_insensitive_optimized(&airport.name, query_lower, is_ascii)
                    || contains_case_insensitive_optimized(
                        &airport.longest_runway_length,
                        query_lower,
                        is_ascii,
                    )
                {
                    return 1;
                }
                0
            }
            Self::Route(route) => {
                // Optimization: Check ICAO first (fastest, high priority)
                if contains_case_insensitive_optimized(&route.departure.ICAO, query_lower, is_ascii)
                    || contains_case_insensitive_optimized(
                        &route.destination.ICAO,
                        query_lower,
                        is_ascii,
                    )
                {
                    return 2;
                }

                // Optimization: Check combined info strings instead of separate fields.
                // departure_info contains "Name (ICAO)". If it matches, and we know ICAO didn't match (above),
                // then Name must have matched.
                // aircraft_info contains "Manufacturer Variant". Checks both at once.
                if contains_case_insensitive_optimized(&route.departure_info, query_lower, is_ascii)
                    || contains_case_insensitive_optimized(
                        &route.destination_info,
                        query_lower,
                        is_ascii,
                    )
                    || contains_case_insensitive_optimized(
                        &route.aircraft_info,
                        query_lower,
                        is_ascii,
                    )
                {
                    return 1;
                }
                0
            }
            Self::History(history) => {
                if contains_case_insensitive_optimized(
                    &history.departure_icao,
                    query_lower,
                    is_ascii,
                ) || contains_case_insensitive_optimized(
                    &history.arrival_icao,
                    query_lower,
                    is_ascii,
                ) {
                    return 2;
                }
                if contains_case_insensitive_optimized(
                    &history.aircraft_name,
                    query_lower,
                    is_ascii,
                ) || contains_case_insensitive_optimized(&history.date, query_lower, is_ascii)
                {
                    return 1;
                }
                0
            }
            Self::Aircraft(aircraft) => {
                if contains_case_insensitive_optimized(&aircraft.icao_code, query_lower, is_ascii) {
                    return 2;
                }
                if contains_case_insensitive_optimized(
                    &aircraft.manufacturer,
                    query_lower,
                    is_ascii,
                ) || contains_case_insensitive_optimized(
                    &aircraft.variant,
                    query_lower,
                    is_ascii,
                ) || contains_case_insensitive_optimized(
                    &aircraft.category,
                    query_lower,
                    is_ascii,
                ) || contains_case_insensitive_optimized(
                    &aircraft.date_flown,
                    query_lower,
                    is_ascii,
                ) {
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
                vec![
                    Cow::Borrowed(&route.departure.Name),
                    Cow::Borrowed(&route.departure.ICAO),
                    Cow::Owned(format!("{}ft", route.departure_runway_length)),
                    Cow::Borrowed(&route.destination.Name),
                    Cow::Borrowed(&route.destination.ICAO),
                    Cow::Owned(format!("{}ft", route.destination_runway_length)),
                    Cow::Borrowed(&route.aircraft.manufacturer),
                    Cow::Borrowed(&route.aircraft.variant),
                    Cow::Borrowed(&route.distance_str),
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
