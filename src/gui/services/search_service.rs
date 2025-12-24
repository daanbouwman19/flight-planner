use crate::gui::data::TableItem;
use crate::traits::Searchable;
use rayon::prelude::*;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// The debouncing duration for search requests to avoid excessive searches on every keystroke.
const SEARCH_DEBOUNCE_DURATION: Duration = Duration::from_millis(50);

/// Maximum number of search results to return to prevent UI slowdown with large datasets
const MAX_SEARCH_RESULTS: usize = 1000;

/// Threshold for using parallel processing for large datasets to improve performance
const PARALLEL_SEARCH_THRESHOLD: usize = 5000;

/// A wrapper struct to enable storing items in a BinaryHeap ordered by score.
/// We implement `Ord` to compare primarily by score.
#[derive(Clone)]
struct ScoredItem {
    score: u8,
    item: Arc<TableItem>,
}

impl PartialEq for ScoredItem {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}

impl Eq for ScoredItem {}

impl PartialOrd for ScoredItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScoredItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.score.cmp(&other.score)
    }
}

/// A specialized accumulator for collecting top search results in parallel.
/// It uses a Min-Heap (via Reverse) to maintain the Top K items with the highest scores.
struct SearchResults {
    /// Min-Heap of top items. We wrap in Reverse to make it a Min-Heap based on score,
    /// so we can efficiently access/remove the item with the LOWEST score among the top K.
    heap: BinaryHeap<std::cmp::Reverse<ScoredItem>>,
}

impl SearchResults {
    fn new() -> Self {
        Self {
            heap: BinaryHeap::with_capacity(MAX_SEARCH_RESULTS),
        }
    }

    /// Adds an item to the accumulator if it qualifies (score > 0 and fits in top K).
    fn push(&mut self, item: &Arc<TableItem>, score: u8) {
        if score == 0 {
            return;
        }

        let scored_item = std::cmp::Reverse(ScoredItem {
            score,
            item: item.clone(),
        });

        if self.heap.len() < MAX_SEARCH_RESULTS {
            self.heap.push(scored_item);
        } else {
            // Check if the new item is better than the worst item currently in the heap
            // We must peek and clone the score to drop the borrow before modifying the heap
            let min_score = self.heap.peek().map(|min| min.0.score);

            if let Some(min_s) = min_score {
                if score > min_s {
                    self.heap.pop();
                    self.heap.push(scored_item);
                }
            }
        }
    }

    /// Merges another accumulator into this one, maintaining the top K results.
    fn merge(mut self, other: Self) -> Self {
        for reversed_item in other.heap {
            let item = reversed_item.0;

            if self.heap.len() < MAX_SEARCH_RESULTS {
                self.heap.push(std::cmp::Reverse(item));
            } else {
                let min_score = self.heap.peek().map(|min| min.0.score);

                if let Some(min_s) = min_score {
                    if item.score > min_s {
                        self.heap.pop();
                        self.heap.push(std::cmp::Reverse(item));
                    }
                }
            }
        }
        self
    }

    /// Flattens the accumulator into a single sorted vector of results (descending by score).
    fn into_vec(self) -> Vec<Arc<TableItem>> {
        let mut items: Vec<ScoredItem> = self.heap.into_iter().map(|r| r.0).collect();
        // Sort descending by score
        items.sort_unstable_by(|a, b| b.score.cmp(&a.score));
        items.into_iter().map(|si| si.item).collect()
    }
}

/// A service dedicated to managing search functionality with debouncing and optimized filtering.
///
/// This service encapsulates the state and logic for handling user search queries,
/// including debouncing input to prevent excessive searches, performing the actual
/// filtering (with parallel processing for large datasets), and caching the results.
#[derive(Default)]
pub struct SearchService {
    /// The current search query string entered by the user.
    query: String,
    /// A cached vector of items that match the current search query.
    filtered_items: Vec<Arc<TableItem>>,
    /// The timestamp of the last search request, used for debouncing.
    last_search_request: Option<Instant>,
    /// A flag indicating whether a search is pending execution after a debounce delay.
    search_pending: bool,
}

impl SearchService {
    /// Creates a new `SearchService` with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a reference to the current search query string.
    pub fn query(&self) -> &str {
        &self.query
    }

    /// Returns a mutable reference to the search query string.
    ///
    /// This allows UI components to bind directly to the query string for input.
    pub fn query_mut(&mut self) -> &mut String {
        &mut self.query
    }

    /// Sets the search query and flags that a new search is pending.
    ///
    /// # Arguments
    ///
    /// * `query` - The new search string.
    ///
    /// # Returns
    ///
    /// `true` if the query was changed, `false` otherwise.
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

    /// Clears the current search query and resets the search state.
    pub fn clear_query(&mut self) {
        self.query.clear();
        self.filtered_items.clear();
        self.search_pending = false;
        self.last_search_request = None;
    }

    /// Returns a slice of the items that have been filtered by the current query.
    pub fn filtered_items(&self) -> &[Arc<TableItem>] {
        &self.filtered_items
    }

    /// Replaces the cached filtered items with a new set.
    ///
    /// This is typically called after a background search operation completes.
    ///
    /// # Arguments
    ///
    /// * `items` - The new vector of filtered items.
    pub fn set_filtered_items(&mut self, items: Vec<Arc<TableItem>>) {
        self.filtered_items = items;
    }

    /// Checks if the current search has produced any results.
    pub fn has_results(&self) -> bool {
        !self.filtered_items.is_empty()
    }

    /// Returns the number of items in the filtered result set.
    pub fn result_count(&self) -> usize {
        self.filtered_items.len()
    }

    /// Filters a slice of `TableItem`s based on a search query.
    ///
    /// This static method contains the core search logic. It calculates a relevance
    /// score for each item and returns a sorted list of matching items. It employs
    /// parallel processing for large datasets to improve performance.
    ///
    /// # Arguments
    ///
    /// * `items` - A slice of `Arc<TableItem>` to be filtered.
    /// * `query` - The search query string.
    ///
    /// # Returns
    ///
    /// A `Vec<Arc<TableItem>>` containing the filtered and sorted results, capped
    /// by `MAX_SEARCH_RESULTS`.
    pub fn filter_items_static(items: &[Arc<TableItem>], query: &str) -> Vec<Arc<TableItem>> {
        if query.is_empty() {
            return items.to_vec();
        }

        // Calculate lowercased query once to avoid repetitive allocations in the loop
        let query_lower = query.to_lowercase();

        if items.len() > PARALLEL_SEARCH_THRESHOLD {
            // Parallel processing for large datasets using optimized reduction.
            // We use a fold-reduce pattern with a local bounded Min-Heap to maintain
            // only the top K results. This avoids allocating a vector of all matches
            // (O(N) memory) and sorting it (O(N log N) time), replacing it with
            // O(K) memory and O(N log K) time.
            items
                .par_iter()
                .fold(SearchResults::new, |mut acc, item| {
                    let score = item.search_score_lower(&query_lower);
                    if score > 0 {
                        acc.push(item, score);
                    }
                    acc
                })
                .reduce(SearchResults::new, |acc, other| acc.merge(other))
                .into_vec()
        } else {
            // Sequential processing for smaller datasets using BinaryHeap for top N results
            use std::cmp::Reverse;
            use std::collections::BinaryHeap;

            let mut heap = BinaryHeap::with_capacity(MAX_SEARCH_RESULTS + 1);
            for (i, item) in items.iter().enumerate() {
                let score = item.search_score_lower(&query_lower);
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

    /// A convenience method to update the search query and trigger the debouncing mechanism.
    pub fn update_query(&mut self, query: String) {
        if self.query != query {
            self.query = query;
            self.set_search_pending(true);
            self.set_last_search_request(Some(Instant::now()));
        }
    }

    /// Checks if a search operation is currently pending (waiting for the debounce timer).
    pub fn is_search_pending(&self) -> bool {
        self.search_pending
    }

    /// Manually sets the search pending flag.
    pub fn set_search_pending(&mut self, pending: bool) {
        self.search_pending = pending;
    }

    /// Returns the timestamp of the last search request.
    pub fn last_search_request(&self) -> Option<Instant> {
        self.last_search_request
    }

    /// Manually sets the timestamp of the last search request.
    pub fn set_last_search_request(&mut self, time: Option<Instant>) {
        self.last_search_request = time;
    }

    /// Determines whether a search should be executed based on the debouncing logic.
    ///
    /// A search is executed if it is pending and the debounce duration has elapsed.
    ///
    /// # Returns
    ///
    /// `true` if a search should be executed, `false` otherwise.
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

    /// Forces a search to be pending, bypassing the debounce timer.
    ///
    /// This is primarily used for testing purposes to trigger an immediate search.
    pub fn force_search_pending(&mut self) {
        self.set_search_pending(true);
        self.set_last_search_request(Some(Instant::now() - Duration::from_secs(1)));
    }

    /// Spawns a new thread to perform the search operation in the background.
    ///
    /// # Arguments
    ///
    /// * `all_items` - The complete list of items to be searched.
    /// * `on_complete` - A callback function to be executed with the search results
    ///   once the thread completes.
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
