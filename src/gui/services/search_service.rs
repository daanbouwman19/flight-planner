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
            // Sequential processing for smaller datasets using BinaryHeap for top N results
            use std::cmp::Reverse;
            use std::collections::BinaryHeap;

            let mut heap = BinaryHeap::with_capacity(MAX_SEARCH_RESULTS + 1);
            for (i, item) in items.iter().enumerate() {
                let score = item.search_score(query);
                if score > 0 {
                    heap.push(Reverse((score, i)));
                    if heap.len() > MAX_SEARCH_RESULTS {
                        heap.pop();
                    }
                }
            }
            let sorted_indices = heap.into_sorted_vec(); // Highest score first
            sorted_indices
                .into_iter()
                .map(|Reverse((_score, i))| items[i].clone())
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
