use crate::gui::data::TableItem;
use crate::traits::Searchable;
use rayon::prelude::*;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// The debouncing duration for search requests to avoid excessive searches on every keystroke.
const SEARCH_DEBOUNCE_DURATION: Duration = Duration::from_millis(50);

/// Maximum number of search results to return to prevent UI slowdown with large datasets
const MAX_SEARCH_RESULTS: usize = 1000;

/// Threshold for using parallel processing for large datasets to improve performance
const PARALLEL_SEARCH_THRESHOLD: usize = 5000;

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

        if items.len() > PARALLEL_SEARCH_THRESHOLD {
            // Parallel processing for large datasets
            let mut filtered: Vec<(u8, Arc<TableItem>)> = items
                .par_iter()
                .map(|item| (item.search_score(query), item.clone()))
                .filter(|(score, _)| *score > 0)
                .collect();

            filtered.par_sort_unstable_by_key(|(score, _)| std::cmp::Reverse(*score));

            filtered
                .into_iter()
                .map(|(_, item)| item)
                .take(MAX_SEARCH_RESULTS)
                .collect::<Vec<_>>()
        } else {
            // Sequential processing for smaller datasets
            let mut filtered: Vec<(u8, Arc<TableItem>)> = items
                .iter()
                .map(|item| (item.search_score(query), item.clone()))
                .filter(|(score, _)| *score > 0)
                .collect();

            filtered.sort_unstable_by_key(|(score, _)| std::cmp::Reverse(*score));

            filtered
                .into_iter()
                .map(|(_, item)| item)
                .take(MAX_SEARCH_RESULTS)
                .collect()
        }
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
            && self
                .last_search_request()
                .is_some_and(|lr| lr.elapsed() > SEARCH_DEBOUNCE_DURATION)
        {
            self.set_search_pending(false);
            return true;
        }
        false
    }

    /// Forces the search to be pending for testing purposes, bypassing the debounce timer.
    pub fn force_search_pending(&mut self) {
        self.set_search_pending(true);
        self.set_last_search_request(Some(Instant::now() - Duration::from_secs(1)));
    }

    pub fn spawn_search_thread<F>(&self, all_items: Vec<Arc<TableItem>>, on_complete: F)
    where
        F: FnOnce(Vec<Arc<TableItem>>) + Send + 'static,
    {
        let query = self.query.clone();
        std::thread::spawn(move || {
            let filtered_items = Self::filter_items_static(&all_items, &query);
            on_complete(filtered_items);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::data::{ListItemAirport, TableItem};
    use std::sync::{Arc, mpsc};
    use std::time::Duration;

    // Helper to create a test airport item
    fn create_airport_item(name: &str, icao: &str) -> Arc<TableItem> {
        Arc::new(TableItem::Airport(ListItemAirport::new(
            name.to_string(),
            icao.to_string(),
            "10000ft".to_string(),
        )))
    }

    #[test]
    fn test_filter_items_static_prioritizes_icao_matches() {
        let items = vec![
            create_airport_item("London Heathrow", "EGLL"),
            create_airport_item("Los Angeles", "KLAX"),
        ];

        // Search for "LAX" - should match KLAX (ICAO) with higher score
        let results = SearchService::filter_items_static(&items, "LAX");
        assert_eq!(results.len(), 1);
        assert_eq!(*results[0], *create_airport_item("Los Angeles", "KLAX"));

        // Search for "London" - should match London Heathrow (name)
        let results = SearchService::filter_items_static(&items, "London");
        assert_eq!(results.len(), 1);
        assert_eq!(*results[0], *create_airport_item("London Heathrow", "EGLL"));
    }

    #[test]
    fn test_filter_items_static_sorts_by_score() {
        let items = vec![
            create_airport_item("LCY Airport", "EGLC"), // Name match, score 1
            create_airport_item("London City", "LCY"),  // ICAO match, score 2
        ];

        let results = SearchService::filter_items_static(&items, "LCY");
        assert_eq!(results.len(), 2);
        // First result should be the ICAO match (score 2)
        assert_eq!(*results[0], *create_airport_item("London City", "LCY"));
        // Second result should be the name match (score 1)
        assert_eq!(*results[1], *create_airport_item("LCY Airport", "EGLC"));
    }

    #[test]
    fn test_filter_items_static_no_matches() {
        let items = vec![
            create_airport_item("Paris Charles de Gaulle", "LFPG"),
            create_airport_item("Tokyo Haneda", "RJTT"),
        ];

        let results = SearchService::filter_items_static(&items, "Berlin");
        assert!(results.is_empty());
    }

    #[test]
    fn test_filter_items_static_empty_query_returns_all() {
        let items = vec![
            create_airport_item("Sydney Kingsford Smith", "YSSY"),
            create_airport_item("Dubai International", "OMDB"),
        ];

        let results = SearchService::filter_items_static(&items, "");
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_filter_items_static_case_insensitive() {
        let items = vec![
            create_airport_item("Amsterdam Schiphol", "EHAM"),
            create_airport_item("Frankfurt Airport", "EDDF"),
        ];

        // Case-insensitive ICAO search
        let results = SearchService::filter_items_static(&items, "eham");
        assert_eq!(results.len(), 1);
        assert_eq!(
            *results[0],
            *create_airport_item("Amsterdam Schiphol", "EHAM")
        );

        // Case-insensitive name search
        let results = SearchService::filter_items_static(&items, "frankfurt");
        assert_eq!(results.len(), 1);
        assert_eq!(
            *results[0],
            *create_airport_item("Frankfurt Airport", "EDDF")
        );
    }

    #[test]
    fn test_spawn_search_thread_calls_callback() {
        let search_service = SearchService::new();
        let (tx, rx) = mpsc::channel();

        let item1 = Arc::new(TableItem::Airport(ListItemAirport::new(
            "Airport A".to_string(),
            "AAAA".to_string(),
            "10000ft".to_string(),
        )));
        let item2 = Arc::new(TableItem::Airport(ListItemAirport::new(
            "Airport B".to_string(),
            "BBBB".to_string(),
            "12000ft".to_string(),
        )));
        let all_items = vec![item1.clone(), item2.clone()];

        search_service.spawn_search_thread(all_items, move |filtered_items| {
            tx.send(filtered_items)
                .expect("Test channel should accept search results");
        });

        let received_items = rx
            .recv_timeout(Duration::from_secs(5))
            .expect("Test should complete within 5 seconds");
        assert_eq!(received_items.len(), 2);
    }
}
