use std::sync::Arc;

use crate::gui::data::ListItemAircraft;
use crate::models::Aircraft;

/// Transforms a slice of `Aircraft` models into a `Vec` of `ListItemAircraft` for UI display.
///
/// This function iterates over the provided aircraft models and converts each one
/// into a `ListItemAircraft`, which contains formatted strings suitable for direct
/// rendering in the user interface.
///
/// # Arguments
///
/// * `aircraft` - A slice of `Arc<Aircraft>` to be transformed.
///
/// # Returns
///
/// A `Vec<ListItemAircraft>` where each item is ready for display.
pub fn transform_to_list_items(aircraft: &[Arc<Aircraft>]) -> Vec<ListItemAircraft> {
    aircraft.iter().map(ListItemAircraft::new).collect()
}

/// Filters a slice of `ListItemAircraft` based on a search string.
///
/// The search is case-insensitive and checks for matches in the manufacturer,
/// variant, and ICAO code fields of each aircraft item.
///
/// # Arguments
///
/// * `items` - A slice of `ListItemAircraft` to be filtered.
/// * `search_text` - The string to search for within the aircraft items.
///
/// # Returns
///
/// A new `Vec<ListItemAircraft>` containing only the items that match the search criteria.
/// If `search_text` is empty, a clone of the original slice is returned.
pub fn filter_items(items: &[ListItemAircraft], search_text: &str) -> Vec<ListItemAircraft> {
    if search_text.is_empty() {
        items.to_vec()
    } else {
        let search_lower = search_text.to_lowercase();
        items
            .iter()
            .filter(|item| {
                crate::util::contains_case_insensitive(&item.manufacturer, &search_lower)
                    || crate::util::contains_case_insensitive(&item.variant, &search_lower)
                    || crate::util::contains_case_insensitive(&item.icao_code, &search_lower)
            })
            .cloned()
            .collect()
    }
}

/// Retrieves the display name for an aircraft given its ID.
///
/// The display name is constructed as "Manufacturer Variant". If the aircraft
/// with the given ID is not found, a default string indicating an unknown
/// aircraft is returned.
///
/// # Arguments
///
/// * `aircraft` - A slice of `Arc<Aircraft>` to search through.
/// * `aircraft_id` - The unique identifier of the aircraft.
///
/// # Returns
///
/// A `String` containing the display name or an "Unknown Aircraft" message.
pub fn get_display_name(aircraft: &[Arc<Aircraft>], aircraft_id: i32) -> String {
    aircraft.iter().find(|a| a.id == aircraft_id).map_or_else(
        || format!("Unknown Aircraft (ID: {aircraft_id})"),
        |a| format!("{} {}", a.manufacturer, a.variant),
    )
}
