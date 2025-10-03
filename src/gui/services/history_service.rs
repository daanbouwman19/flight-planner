use crate::gui::data::{ListItemHistory, TableItem};
use crate::traits::Searchable;

/// Filters a slice of `ListItemHistory` based on a search string.
///
/// This function delegates the filtering logic to the `Searchable` trait
/// implementation for `TableItem::History`, which provides a relevance score
/// for each item against the search query.
///
/// # Arguments
///
/// * `items` - A slice of `ListItemHistory` to be filtered.
/// * `search_text` - The string to search for.
///
/// # Returns
///
/// A new `Vec<ListItemHistory>` containing only the items that match the search criteria.
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

/// Sorts a slice of `ListItemHistory` in place based on a specified column.
///
/// # Arguments
///
/// * `items` - A mutable slice of `ListItemHistory` to be sorted.
/// * `column` - The name of the column to sort by (e.g., "departure", "date").
/// * `ascending` - A boolean indicating the sort direction (`true` for ascending,
///   `false` for descending).
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
