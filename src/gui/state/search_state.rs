use std::sync::Arc;
use std::time::Instant;

use crate::gui::data::TableItem;

/// State for handling search functionality with debouncing.
#[derive(Default)]
pub struct SearchState {
    /// The current search query.
    query: String,
    /// The items filtered based on the search query.
    filtered_items: Vec<Arc<TableItem>>,
    /// The last time a search was requested (for debouncing).
    last_search_request: Option<Instant>,
    /// Whether a search is pending (for debouncing).
    search_pending: bool,
}

impl SearchState {
    /// Creates a new search state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets the current search query.
    pub fn get_query(&self) -> &str {
        &self.query
    }

    /// Gets the filtered items from search.
    pub fn get_filtered_items(&self) -> &[Arc<TableItem>] {
        &self.filtered_items
    }

    /// Updates the search query and triggers search if changed.
    pub fn update_query(&mut self, query: String) {
        if self.query != query {
            self.query = query;
            self.set_search_pending(true);
            self.set_last_search_request(Some(std::time::Instant::now()));
        }
    }

    /// Clears the search query.
    pub fn clear_query(&mut self) {
        self.query.clear();
    }

    /// Sets the filtered items.
    pub fn set_filtered_items(&mut self, items: Vec<Arc<TableItem>>) {
        self.filtered_items = items;
    }

    /// Sets whether a search is pending.
    pub const fn set_search_pending(&mut self, pending: bool) {
        self.search_pending = pending;
    }

    /// Sets the last search request time.
    pub const fn set_last_search_request(&mut self, time: Option<Instant>) {
        self.last_search_request = time;
    }

    /// Checks if enough time has passed for debounced search (300ms).
    pub fn should_execute_search(&self) -> bool {
        self.last_search_request.is_some_and(|last_request_time| {
            self.search_pending && last_request_time.elapsed() >= std::time::Duration::from_millis(300)
        })
    }
}
