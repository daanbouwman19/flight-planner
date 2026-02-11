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
/// We implement `Ord` to compare primarily by score, using original_index as a stable tie-breaker.
#[derive(Clone)]
struct ScoredItem {
    score: u8,
    original_index: usize,
    item: Arc<TableItem>,
}

impl PartialEq for ScoredItem {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score && self.original_index == other.original_index
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
        // Implementation defines priority where higher scores and lower original indices are considered "greater".
        // Score comparison: ascending (higher score is Greater).
        // Index comparison: descending (lower index is Greater).
        self.score
            .cmp(&other.score)
            .then_with(|| other.original_index.cmp(&self.original_index))
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
    fn push(&mut self, item: &Arc<TableItem>, score: u8, original_index: usize) {
        if score == 0 {
            return;
        }

        if self.heap.len() < MAX_SEARCH_RESULTS {
            self.heap.push(std::cmp::Reverse(ScoredItem {
                score,
                original_index,
                item: item.clone(),
            }));
        } else if let Some(worst) = self.heap.peek() {
            // Check if the new item is better than the worst item currently in the heap
            // We must peek to compare.
            // Since we use Reverse, peek gives us the "smallest" Reverse element,
            // which corresponds to the ScoredItem with the smallest Ord value (worst item).
            // If our new item is "greater" (better) than the worst item, we replace it.
            let worst_item = &worst.0;
            // Optimization: Check if new item is "better" than worst BEFORE cloning the Arc item
            // Better = Higher score OR (Equal score AND Lower index)
            // Note: worst.0 is the wrapped item. Reverse gives smallest element first, which is the "worst" ScoredItem.
            let is_better = match score.cmp(&worst_item.score) {
                Ordering::Greater => true,
                Ordering::Less => false,
                Ordering::Equal => original_index < worst_item.original_index,
            };

            if is_better {
                self.heap.pop();
                self.heap.push(std::cmp::Reverse(ScoredItem {
                    score,
                    original_index,
                    item: item.clone(),
                }));
            }
        }
    }

    /// Merges another accumulator into this one, maintaining the top K results.
    fn merge(mut self, other: Self) -> Self {
        for reversed_item in other.heap {
            let item = reversed_item.0;
            // logic similar to push, but reusing the item
            if self.heap.len() < MAX_SEARCH_RESULTS {
                self.heap.push(std::cmp::Reverse(item));
            } else if let Some(worst) = self.heap.peek() {
                // Optimization: Check if new item is "better" than worst BEFORE cloning the Arc item
                // Better = Higher score OR (Equal score AND Lower index)
                let is_better = item > worst.0;

                if is_better {
                    self.heap.pop();
                    self.heap.push(std::cmp::Reverse(item));
                }
            }
        }
        self
    }

    /// Flattens the accumulator into a single sorted vector of results (descending by score, stable).
    fn into_vec(self) -> Vec<Arc<TableItem>> {
        let mut items: Vec<ScoredItem> = self.heap.into_iter().map(|r| r.0).collect();
        // Sort descending by score, using the defined Ord which handles stability via original_index
        items.sort_unstable_by(|a, b| b.cmp(a));
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
    /// The number of currently active search operations (background threads).
    active_searches: usize,
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
                .enumerate()
                .fold(SearchResults::new, |mut acc, (index, item)| {
                    let score = item.search_score_lower(&query_lower);
                    if score > 0 {
                        acc.push(item, score, index);
                    }
                    acc
                })
                .reduce(SearchResults::new, |acc, other| acc.merge(other))
                .into_vec()
        } else {
            // Sequential processing for smaller datasets using BinaryHeap for top N results
            use std::cmp::Reverse;
            use std::collections::BinaryHeap;

            // We use (score, Reverse(index)) as the key.
            // Reverse(index) ensures that for equal scores, smaller index is considered "larger"
            // (because Reverse(Small) > Reverse(Large)).
            // The heap stores Reverse<Key>, so pop() removes the "smallest" Key.
            // Smallest Key = Lowest Score, or (Equal Score and Largest Index).
            // This means we discard low scores and late items, keeping high scores and early items.
            let mut heap = BinaryHeap::with_capacity(MAX_SEARCH_RESULTS + 1);
            for (i, item) in items.iter().enumerate() {
                let score = item.search_score_lower(&query_lower);
                if score > 0 {
                    heap.push(Reverse((score, Reverse(i))));
                    if heap.len() > MAX_SEARCH_RESULTS {
                        heap.pop();
                    }
                }
            }
            let sorted_indices = heap.into_sorted_vec(); // Ascending order of Reverse<Key> -> Descending order of Key
            // Key is (score, Reverse(index)). Descending Key -> High Score, Low Index.
            sorted_indices
                .into_iter()
                .map(|Reverse((_score, Reverse(i)))| items[i].clone())
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

    /// Increments the count of active search operations.
    pub fn increment_active_searches(&mut self) {
        self.active_searches += 1;
    }

    /// Decrements the count of active search operations.
    pub fn decrement_active_searches(&mut self) {
        if self.active_searches > 0 {
            self.active_searches -= 1;
        }
    }

    /// Checks if there are any active search operations.
    pub fn is_searching(&self) -> bool {
        self.active_searches > 0
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
            // Optimization: If the query is empty, we can return the items directly
            // without calling filter_items_static, which would clone the vector
            // (O(N) atomic increments) unnecessarily.
            let filtered_items = if query.is_empty() {
                all_items
            } else {
                Self::filter_items_static(&all_items, &query)
            };
            on_complete(filtered_items);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_service_active_searches() {
        let mut service = SearchService::new();

        assert!(!service.is_searching(), "Should not be searching initially");

        service.increment_active_searches();
        assert!(
            service.is_searching(),
            "Should be searching after increment"
        );

        service.increment_active_searches();
        assert!(
            service.is_searching(),
            "Should still be searching after second increment"
        );

        service.decrement_active_searches();
        assert!(
            service.is_searching(),
            "Should still be searching after one decrement"
        );

        service.decrement_active_searches();
        assert!(
            !service.is_searching(),
            "Should not be searching after all decrements"
        );

        // Verify it doesn't underflow
        service.decrement_active_searches();
        assert!(!service.is_searching(), "Should stay not searching");
    }
}
