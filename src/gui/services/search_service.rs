use crate::gui::data::TableItem;
use crate::gui::state::ViewState;
use std::sync::Arc;
use std::time::{Duration, Instant};

const DEBOUNCE_DURATION: Duration = Duration::from_millis(300);

/// Service for handling search functionality with debouncing.
/// This is a **Model** in MVVM - it contains business logic for searching.
/// This service is now stateless and operates on the `ViewState`.
#[derive(Default)]
pub struct SearchService {}

impl SearchService {
    pub fn new() -> Self {
        Self::default()
    }

    /// Filters items based on the query in the `ViewState`.
    /// This is a static method to allow use without a service instance.
    pub fn filter_items(state: &mut ViewState, all_items: &[Arc<TableItem>]) {
        if state.table_search.is_empty() {
            state.filtered_items = all_items.to_vec();
            return;
        }

        state.filtered_items = all_items
            .iter()
            .filter(|item| item.matches_query(&state.table_search))
            .cloned()
            .collect();
    }

    /// Updates the search query in the `ViewState` and handles debouncing.
    /// Returns `true` if the search should be performed immediately.
    pub fn update_search_query(state: &mut ViewState, query: String) -> bool {
        if state.table_search != query {
            state.table_search = query;
            state.last_search_request = Some(Instant::now());
            state.search_pending = true;
        }
        // Always return false, letting the main loop handle the debounce timing
        false
    }

    /// Checks if a debounced search should be performed.
    pub fn should_perform_search(state: &mut ViewState) -> bool {
        if state.search_pending {
            if let Some(last_request) = state.last_search_request {
                if last_request.elapsed() >= DEBOUNCE_DURATION {
                    state.search_pending = false;
                    return true;
                }
            }
        }
        false
    }

    /// Clears the search query in the `ViewState`.
    pub fn clear_search(state: &mut ViewState) {
        state.table_search.clear();
        state.filtered_items.clear();
        state.search_pending = false;
        state.last_search_request = None;
    }

    pub fn query<'a>(&self, view_state: &'a ViewState) -> &'a str {
        &view_state.table_search
    }

    pub fn query_mut<'a>(&mut self, view_state: &'a mut ViewState) -> &'a mut String {
        &mut view_state.table_search
    }

    pub fn set_filtered_items(&self, view_state: &mut ViewState, items: Vec<Arc<TableItem>>) {
        view_state.filtered_items = items;
    }

    pub fn filtered_items<'a>(&self, view_state: &'a ViewState) -> &'a [Arc<TableItem>] {
        &view_state.filtered_items
    }

    pub fn clear_query(&self, view_state: &mut ViewState) {
        view_state.table_search.clear();
        view_state.filtered_items.clear();
    }
}
