use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

use crate::gui::data::ListItemAirport;
use crate::models::{Airport, Runway};

/// Transforms a slice of `Airport` models into `ListItemAirport`s, including runway data.
///
/// This function iterates through the provided airports and uses the `runway_data`
/// `HashMap` to find the longest runway for each airport. This information is then
/// included in the resulting `ListItemAirport`.
///
/// # Arguments
///
/// * `airports` - A slice of `Arc<Airport>` to be transformed.
/// * `runway_data` - A `HashMap` where the key is the airport ID and the value is
///   a vector of its runways.
///
/// # Returns
///
/// A `Vec<ListItemAirport>` where each item is enriched with runway information.
pub fn transform_to_list_items_with_runways(
    airports: &[Arc<Airport>],
    runway_data: &HashMap<i32, Arc<Vec<Runway>>>,
) -> Vec<ListItemAirport> {
    // Use parallel iterator for improved performance on large datasets
    airports
        .par_iter()
        .map(|airport| {
            let runway_length = runway_data
                .get(&airport.ID)
                .and_then(|runways| {
                    runways
                        .iter()
                        .max_by_key(|r| r.Length)
                        .map(|r| format!("{}ft", r.Length))
                })
                .unwrap_or_else(|| "No runways".to_string());

            ListItemAirport {
                name: airport.Name.clone(),
                icao: airport.ICAO.clone(),
                longest_runway_length: runway_length,
            }
        })
        .collect()
}

/// Transforms a slice of `Airport` models into `ListItemAirport`s without runway data.
///
/// This version is provided for compatibility or for cases where runway information
/// is not needed. It sets the `longest_runway_length` to a default value. For a
/// version with runway data, use `transform_to_list_items_with_runways`.
///
/// # Arguments
///
/// * `airports` - A slice of `Arc<Airport>` to be transformed.
///
/// # Returns
///
/// A `Vec<ListItemAirport>` with default runway information.
pub fn transform_to_list_items(airports: &[Arc<Airport>]) -> Vec<ListItemAirport> {
    airports
        .par_iter()
        .map(|airport| ListItemAirport {
            name: airport.Name.clone(),
            icao: airport.ICAO.clone(),
            longest_runway_length: "0".to_string(), // This would need runway data to calculate properly
        })
        .collect()
}

/// Filters a slice of `ListItemAirport` based on a search string.
///
/// The search is case-insensitive and checks for matches in the ICAO code
/// and name fields of each airport item.
///
/// # Arguments
///
/// * `items` - A slice of `ListItemAirport` to be filtered.
/// * `search_text` - The string to search for within the airport items.
///
/// # Returns
///
/// A new `Vec<ListItemAirport>` containing only the items that match the search criteria.
/// If `search_text` is empty, a clone of the original slice is returned.
pub fn filter_items(items: &[ListItemAirport], search_text: &str) -> Vec<ListItemAirport> {
    if search_text.is_empty() {
        return items.to_vec();
    }

    let search_lower = search_text.to_lowercase();
    // Helper predicate to avoid duplication
    let predicate = |item: &ListItemAirport| {
        item.icao.to_lowercase().contains(&search_lower)
            || item.name.to_lowercase().contains(&search_lower)
    };

    // Use parallel iterator if the dataset is large enough
    if items.len() > 1000 {
        items.par_iter().filter(|i| predicate(i)).cloned().collect()
    } else {
        items.iter().filter(|i| predicate(i)).cloned().collect()
    }
}

/// Retrieves the display name for an airport given its ICAO code.
///
/// The display name is constructed as "Name (ICAO)". If the airport with the
/// given ICAO code is not found, a default string indicating an unknown
/// airport is returned.
///
/// # Arguments
///
/// * `airports` - A slice of `Arc<Airport>` to search through.
/// * `icao` - The ICAO code of the airport to find.
///
/// # Returns
///
/// A `String` containing the display name or an "Unknown Airport" message.
pub fn get_display_name(airports: &[Arc<Airport>], icao: &str) -> String {
    airports.iter().find(|a| a.ICAO == icao).map_or_else(
        || format!("Unknown Airport ({icao})"),
        |a| format!("{} ({})", a.Name, a.ICAO),
    )
}
