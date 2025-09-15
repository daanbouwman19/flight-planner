use crate::gui::data::TableItem;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Service for handling search functionality with debouncing.
/// This is a **Model** in MVVM - it contains business logic for searching.
#[derive(Default)]
pub struct SearchService {
    /// The current search query.
    query: String,
    /// The items filtered based on the search query (temporary cache).
    filtered_items: Vec<Arc<TableItem>>,
    /// The last time a search was requested (for debouncing).
    last_search_request: Option<Instant>,
    /// Whether a search is pending (for debouncing).
    search_pending: bool,
}

impl SearchService {
    /// Creates a new search service.
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets the current search query.
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Gets a mutable reference to the search query.
    pub fn query_mut(&mut self) -> &mut String {
        &mut self.query
    }

    /// Sets the search query and returns whether it changed.
    pub fn set_query(&mut self, query: String) -> bool {
        if self.query != query {
            self.query = query;
            self.set_search_pending(true);
            self.set_last_search_request(Some(Instant::now()));
            true
        } else {
            false
        }
    }

    /// Clears the search query.
    pub fn clear_query(&mut self) {
        self.query.clear();
        self.filtered_items.clear();
        self.search_pending = false;
        self.last_search_request = None;
    }

    /// Gets the filtered items.
    pub fn filtered_items(&self) -> &[Arc<TableItem>] {
        &self.filtered_items
    }

    /// Sets the filtered items (called after filtering).
    pub fn set_filtered_items(&mut self, items: Vec<Arc<TableItem>>) {
        self.filtered_items = items;
    }

    /// Checks if there are any filtered results.
    pub fn has_results(&self) -> bool {
        !self.filtered_items.is_empty()
    }

    /// Gets the number of filtered results.
    pub fn result_count(&self) -> usize {
        self.filtered_items.len()
    }

    /// Static method for filtering items (used by components).
    pub fn filter_items_static(items: &[Arc<TableItem>], query: &str) -> Vec<Arc<TableItem>> {
        if query.is_empty() {
            return items.to_vec();
        }

        items
            .iter()
            .filter(|item| item.matches_query(query))
            .cloned()
            .collect()
    }

    /// Updates the search query and triggers search if changed.
    pub fn update_query(&mut self, query: String) {
        if self.query != query {
            self.query = query;
            self.set_search_pending(true);
            self.set_last_search_request(Some(Instant::now()));
        }
    }

    /// Checks if a search is pending (for debouncing).
    pub fn is_search_pending(&self) -> bool {
        self.search_pending
    }

    /// Sets the search pending flag.
    pub fn set_search_pending(&mut self, pending: bool) {
        self.search_pending = pending;
    }

    /// Gets the last search request time.
    pub fn last_search_request(&self) -> Option<Instant> {
        self.last_search_request
    }

    /// Sets the last search request time.
    pub fn set_last_search_request(&mut self, time: Option<Instant>) {
        self.last_search_request = time;
    }

    /// Checks if a search should be executed based on debouncing logic.
    pub fn should_execute_search(&mut self) -> bool {
        if self.is_search_pending()
            && let Some(last_request) = self.last_search_request()
            && last_request.elapsed() > Duration::from_millis(300)
        {
            self.set_search_pending(false);
            return true;
        }
        false
    }
}
