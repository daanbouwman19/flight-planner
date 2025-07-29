use std::collections::HashMap;
use std::sync::Arc;

use crate::gui::data::ListItemAirport;
use crate::models::{Airport, Runway};

/// Transforms airport models to UI list items with runway data.
///
/// # Arguments
///
/// * `airports` - The airports to transform
/// * `runway_data` - HashMap of airport ID to runway vectors
///
/// # Returns
///
/// Returns a vector of airport list items.
pub fn transform_to_list_items_with_runways(
    airports: &[Arc<Airport>],
    runway_data: &HashMap<i32, Arc<Vec<Runway>>>,
) -> Vec<ListItemAirport> {
    airports
        .iter()
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

/// Transforms airport models to UI list items (legacy version for compatibility).
///
/// # Arguments
///
/// * `airports` - The airports to transform
///
/// # Returns
///
/// Returns a vector of airport list items.
pub fn transform_to_list_items(airports: &[Arc<Airport>]) -> Vec<ListItemAirport> {
    airports
        .iter()
        .map(|airport| ListItemAirport {
            name: airport.Name.clone(),
            icao: airport.ICAO.clone(),
            longest_runway_length: "0".to_string(), // This would need runway data to calculate properly
        })
        .collect()
}

/// Filters airport items based on search text.
///
/// # Arguments
///
/// * `items` - The airport items to filter
/// * `search_text` - The text to search for
///
/// # Returns
///
/// Returns a vector of filtered airport items.
pub fn filter_items(items: &[ListItemAirport], search_text: &str) -> Vec<ListItemAirport> {
    if search_text.is_empty() {
        items.to_vec()
    } else {
        let search_lower = search_text.to_lowercase();
        items
            .iter()
            .filter(|item| {
                item.icao.to_lowercase().contains(&search_lower)
                    || item.name.to_lowercase().contains(&search_lower)
            })
            .cloned()
            .collect()
    }
}

/// Gets the display name for an airport by its ICAO.
///
/// # Arguments
///
/// * `airports` - All available airports
/// * `icao` - The ICAO of the airport
///
/// # Returns
///
/// Returns the display name of the airport.  
pub fn get_display_name(airports: &[Arc<Airport>], icao: &str) -> String {
    airports.iter().find(|a| a.ICAO == icao).map_or_else(
        || format!("Unknown Airport ({icao})"),
        |a| format!("{} ({})", a.Name, a.ICAO),
    )
}
