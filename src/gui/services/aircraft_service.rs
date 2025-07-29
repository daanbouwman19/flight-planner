use std::sync::Arc;

use crate::gui::data::ListItemAircraft;
use crate::models::Aircraft;

/// Transforms aircraft models to UI list items.
///
/// # Arguments
///
/// * `aircraft` - The aircraft to transform
///
/// # Returns
///
/// Returns a vector of aircraft list items.
pub fn transform_to_list_items(aircraft: &[Arc<Aircraft>]) -> Vec<ListItemAircraft> {
    aircraft.iter().map(ListItemAircraft::new).collect()
}

/// Filters aircraft items based on search text.
///
/// # Arguments
///
/// * `items` - The aircraft items to filter
/// * `search_text` - The text to search for
///
/// # Returns
///
/// Returns a vector of filtered aircraft items.
pub fn filter_items(items: &[ListItemAircraft], search_text: &str) -> Vec<ListItemAircraft> {
    if search_text.is_empty() {
        items.to_vec()
    } else {
        let search_lower = search_text.to_lowercase();
        items
            .iter()
            .filter(|item| {
                item.manufacturer.to_lowercase().contains(&search_lower)
                    || item.variant.to_lowercase().contains(&search_lower)
                    || item.icao_code.to_lowercase().contains(&search_lower)
            })
            .cloned()
            .collect()
    }
}

/// Gets the display name for an aircraft by its ID.
///
/// # Arguments
///
/// * `aircraft` - All available aircraft
/// * `aircraft_id` - The ID of the aircraft
///
/// # Returns
///
/// Returns the display name of the aircraft.
pub fn get_display_name(aircraft: &[Arc<Aircraft>], aircraft_id: i32) -> String {
    aircraft.iter().find(|a| a.id == aircraft_id).map_or_else(
        || format!("Unknown Aircraft (ID: {aircraft_id})"),
        |a| format!("{} {}", a.manufacturer, a.variant),
    )
}
