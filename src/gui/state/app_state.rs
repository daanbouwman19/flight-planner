use std::sync::Arc;

use super::{popup_state::DisplayMode, PopupState, SearchState, UiState};
use crate::gui::data::TableItem;
use crate::gui::services::route_service::RouteService;
use crate::models::{Aircraft, Airport};
use crate::modules::routes::RouteGenerator;

/// Main application state that combines all sub-states.
pub struct AppState {
    /// All items available for display (source of truth).
    all_items: Vec<Arc<TableItem>>,
    /// All available aircraft.
    all_aircraft: Vec<Arc<Aircraft>>,
    /// State for handling popups.
    popup_state: PopupState,
    /// State for handling search.
    search_state: SearchState,
    /// State for UI interactions.
    ui_state: UiState,
    /// Service for handling route operations.
    route_service: RouteService,
}

impl AppState {
    /// Creates a new application state.
    pub fn new(all_aircraft: Vec<Arc<Aircraft>>, route_generator: RouteGenerator) -> Self {
        let route_service = RouteService::new(route_generator);
        Self {
            all_items: Vec::new(),
            all_aircraft,
            popup_state: PopupState::new(),
            search_state: SearchState::new(),
            ui_state: UiState::new(),
            route_service,
        }
    }

    /// Gets all items (the complete unfiltered list).
    /// This should be used when you need access to the full dataset,
    /// such as for search filtering operations.
    pub fn get_all_items(&self) -> &[Arc<TableItem>] {
        &self.all_items
    }

    /// Gets the items that should currently be displayed in the GUI.
    /// This applies search filtering if a search query is active.
    pub fn get_displayed_items(&self) -> &[Arc<TableItem>] {
        if self.search_state.get_query().is_empty() {
            &self.all_items
        } else {
            self.search_state.get_filtered_items()
        }
    }

    /// Sets all items (replaces the current item collection).
    pub fn set_all_items(&mut self, items: Vec<Arc<TableItem>>) {
        self.all_items = items;
        // Clear search results since the underlying data changed
        self.search_state.clear_query();
    }

    /// Clears all items.
    pub fn clear_all_items(&mut self) {
        self.all_items.clear();
        self.search_state.clear_query();
    }

    /// Extends all items with additional items.
    pub fn extend_all_items(&mut self, items: impl IntoIterator<Item = Arc<TableItem>>) {
        self.all_items.extend(items);
        // If search is active, we need to re-filter
        if !self.search_state.get_query().is_empty() {
            // The UI layer should handle re-triggering the search
            self.search_state.set_search_pending(true);
        }
    }

    /// Gets a reference to all aircraft.
    pub fn get_all_aircraft(&self) -> &[Arc<Aircraft>] {
        &self.all_aircraft
    }

    /// Sets all aircraft (used when refreshing data).
    pub fn set_all_aircraft(&mut self, aircraft: Vec<Arc<Aircraft>>) {
        self.all_aircraft = aircraft;
    }

    /// Gets a reference to the popup state.
    pub const fn get_popup_state(&self) -> &PopupState {
        &self.popup_state
    }

    /// Gets a mutable reference to the popup state.
    pub const fn get_popup_state_mut(&mut self) -> &mut PopupState {
        &mut self.popup_state
    }

    /// Gets a reference to the search state.
    pub const fn get_search_state(&self) -> &SearchState {
        &self.search_state
    }

    /// Gets a mutable reference to the search state.
    pub const fn get_search_state_mut(&mut self) -> &mut SearchState {
        &mut self.search_state
    }

    /// Gets a reference to the UI state.
    pub const fn get_ui_state(&self) -> &UiState {
        &self.ui_state
    }

    /// Gets a mutable reference to the UI state.
    pub const fn get_ui_state_mut(&mut self) -> &mut UiState {
        &mut self.ui_state
    }

    /// Gets a reference to the route service.
    pub const fn get_route_service(&self) -> &RouteService {
        &self.route_service
    }

    /// Gets a reference to the available airports from the route service.
    pub fn get_available_airports(&self) -> &[Arc<Airport>] {
        self.route_service.get_available_airports()
    }

    /// Resets all state to default values.
    ///
    /// # Arguments
    ///
    /// * `clear_search_query` - Whether to clear the search query or keep it
    /// * `preserve_departure` - Whether to preserve departure airport settings
    pub fn reset_state(&mut self, clear_search_query: bool, preserve_departure: bool) {
        if clear_search_query {
            self.search_state.clear_query();
        }
        self.ui_state.reset(preserve_departure);
        // Note: popup_state is typically not reset here as it manages its own lifecycle
    }

    /// Sets new items and updates the associated display mode atomically.
    /// This helps maintain consistency between data and UI state.
    ///
    /// # Arguments
    ///
    /// * `items` - The new items to display
    /// * `display_mode` - The display mode associated with these items
    pub fn set_items_with_display_mode(
        &mut self,
        items: Vec<Arc<TableItem>>,
        display_mode: DisplayMode,
    ) {
        self.set_all_items(items);
        self.popup_state.set_display_mode(display_mode);
    }

    /// Clears items and sets display mode atomically (for operations that generate data progressively).
    ///
    /// # Arguments
    ///
    /// * `display_mode` - The display mode for the new data
    pub fn clear_and_set_display_mode(&mut self, display_mode: DisplayMode) {
        self.clear_all_items();
        self.popup_state.set_display_mode(display_mode);
    }
}
