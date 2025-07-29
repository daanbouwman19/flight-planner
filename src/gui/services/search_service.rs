use std::sync::Arc;

use crate::gui::data::TableItem;

/// Service for handling search functionality.
pub struct SearchService;

impl SearchService {
    /// Filters items based on a search query.
    ///
    /// # Arguments
    ///
    /// * `items` - The items to filter
    /// * `query` - The search query
    ///
    /// # Returns
    ///
    /// Returns a vector of filtered items.
    pub fn filter_items(items: &[Arc<TableItem>], query: &str) -> Vec<Arc<TableItem>> {
        if query.is_empty() {
            items.to_vec()
        } else {
            items
                .iter()
                .filter(|item| item.matches_query(query))
                .cloned()
                .collect()
        }
    }
}
