use crate::gui::data::ListItemRoute;

/// Filters route items based on search text.
///
/// # Arguments
///
/// * `items` - The route items to filter
/// * `search_text` - The text to search for
///
/// # Returns
///
/// Returns a vector of filtered route items.
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
                    || item.route_length.contains(&search_lower)
            })
            .cloned()
            .collect()
    }
}

/// Sorts route items by the given column and direction.
///
/// # Arguments
///
/// * `items` - The route items to sort (modified in place)
/// * `column` - The column to sort by
/// * `ascending` - Whether to sort in ascending order
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
                // Parse route_length (which is a string like "123 NM") for comparison
                let a_distance = a
                    .route_length
                    .split_whitespace()
                    .next()
                    .unwrap_or("0")
                    .parse::<f64>()
                    .unwrap_or(0.0);
                let b_distance = b
                    .route_length
                    .split_whitespace()
                    .next()
                    .unwrap_or("0")
                    .parse::<f64>()
                    .unwrap_or(0.0);
                if ascending {
                    a_distance
                        .partial_cmp(&b_distance)
                        .unwrap_or(std::cmp::Ordering::Equal)
                } else {
                    b_distance
                        .partial_cmp(&a_distance)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }
            });
        }
        _ => {} // Unknown column, do nothing
    }
}
