use crate::gui::data::ListItemRoute;

// parse_distance is no longer needed as we store raw numbers

/// Filters a slice of `ListItemRoute` based on a search string.
///
/// The search is case-insensitive and checks for matches in the departure ICAO,
/// destination ICAO, aircraft manufacturer/variant, and route length.
///
/// # Arguments
///
/// * `items` - A slice of `ListItemRoute` to be filtered.
/// * `search_text` - The string to search for.
///
/// # Returns
///
/// A new `Vec<ListItemRoute>` containing only the items that match the search criteria.
pub fn filter_items(items: &[ListItemRoute], search_text: &str) -> Vec<ListItemRoute> {
    if search_text.is_empty() {
        items.to_vec()
    } else {
        let search_lower = search_text.to_lowercase();
        items
            .iter()
            .filter(|item| {
                item.departure.ICAO.to_lowercase().contains(&search_lower)
                    || item.destination.ICAO.to_lowercase().contains(&search_lower)
                    || format!("{} {}", item.aircraft.manufacturer, item.aircraft.variant)
                        .to_lowercase()
                        .contains(&search_lower)
                    || format!("{:.1}", item.route_length).contains(&search_lower)
            })
            .cloned()
            .collect()
    }
}

/// Sorts a slice of `ListItemRoute` in place based on a specified column.
///
/// # Arguments
///
/// * `items` - A mutable slice of `ListItemRoute` to be sorted.
/// * `column` - The name of the column to sort by (e.g., "departure", "distance").
/// * `ascending` - A boolean indicating the sort direction (`true` for ascending,
///   `false` for descending).
pub fn sort_items(items: &mut [ListItemRoute], column: &str, ascending: bool) {
    match column {
        "departure" => {
            items.sort_by(|a, b| {
                if ascending {
                    a.departure.ICAO.cmp(&b.departure.ICAO)
                } else {
                    b.departure.ICAO.cmp(&a.departure.ICAO)
                }
            });
        }
        "arrival" => {
            items.sort_by(|a, b| {
                if ascending {
                    a.destination.ICAO.cmp(&b.destination.ICAO)
                } else {
                    b.destination.ICAO.cmp(&a.destination.ICAO)
                }
            });
        }
        "aircraft" => {
            items.sort_by(|a, b| {
                let a_name = format!("{} {}", a.aircraft.manufacturer, a.aircraft.variant);
                let b_name = format!("{} {}", b.aircraft.manufacturer, b.aircraft.variant);
                if ascending {
                    a_name.cmp(&b_name)
                } else {
                    b_name.cmp(&a_name)
                }
            });
        }
        "distance" => {
            items.sort_by(|a, b| {
                if ascending {
                    a.route_length
                        .partial_cmp(&b.route_length)
                        .unwrap_or(std::cmp::Ordering::Equal)
                } else {
                    b.route_length
                        .partial_cmp(&a.route_length)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }
            });
        }
        _ => {} // Unknown column, do nothing
    }
}
