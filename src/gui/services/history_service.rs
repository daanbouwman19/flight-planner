use crate::gui::data::{ListItemHistory, TableItem};
use crate::traits::Searchable;

/// Filters history items based on search text.
///
/// # Arguments
///
/// * `items` - The history items to filter
/// * `search_text` - The text to search for
///
/// # Returns
///
/// Returns a vector of filtered history items.
pub fn filter_items(items: &[ListItemHistory], search_text: &str) -> Vec<ListItemHistory> {
    if search_text.is_empty() {
        items.to_vec()
    } else {
        items
            .iter()
            .filter(|item| TableItem::History((*item).clone()).search_score(search_text) > 0)
            .cloned()
            .collect()
    }
}

/// Sorts history items by the given column and direction.
///
/// # Arguments
///
/// * `items` - The history items to sort (modified in place)
/// * `column` - The column to sort by
/// * `ascending` - Whether to sort in ascending order
pub fn sort_items(items: &mut [ListItemHistory], column: &str, ascending: bool) {
    match column {
        "departure" => {
            items.sort_by(|a, b| {
                if ascending {
                    a.departure_icao.cmp(&b.departure_icao)
                } else {
                    b.departure_icao.cmp(&a.departure_icao)
                }
            });
        }
        "arrival" => {
            items.sort_by(|a, b| {
                if ascending {
                    a.arrival_icao.cmp(&b.arrival_icao)
                } else {
                    b.arrival_icao.cmp(&a.arrival_icao)
                }
            });
        }
        "aircraft" => {
            items.sort_by(|a, b| {
                if ascending {
                    a.aircraft_name.cmp(&b.aircraft_name)
                } else {
                    b.aircraft_name.cmp(&a.aircraft_name)
                }
            });
        }
        "date" => {
            items.sort_by(|a, b| {
                if ascending {
                    a.date.cmp(&b.date)
                } else {
                    b.date.cmp(&a.date)
                }
            });
        }
        _ => {} // Unknown column, do nothing
    }
}
