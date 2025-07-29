use std::sync::Arc;

use super::{PopupState, SearchState, UiState};
use crate::gui::data::TableItem;
use crate::models::{Aircraft, Airport};
use crate::modules::routes::RouteGenerator;

/// Main application state that combines all sub-states.
pub struct AppState {
    /// The items currently displayed in the GUI.
    displayed_items: Vec<Arc<TableItem>>,
    /// All available aircraft.
    all_aircraft: Vec<Arc<Aircraft>>,
    /// State for handling popups.
    popup_state: PopupState,
    /// State for handling search.
    search_state: SearchState,
    /// State for UI interactions.
    ui_state: UiState,
    /// Route generator for creating flight routes.
    route_generator: RouteGenerator,
}

impl AppState {
    /// Creates a new application state.
    pub fn new(all_aircraft: Vec<Arc<Aircraft>>, route_generator: RouteGenerator) -> Self {
        Self {
            displayed_items: Vec::new(),
            all_aircraft,
            popup_state: PopupState::new(),
            search_state: SearchState::new(),
            ui_state: UiState::new(),
            route_generator,
        }
    }

    /// Gets a reference to the displayed items.
    pub fn get_displayed_items(&self) -> &[Arc<TableItem>] {
        &self.displayed_items
    }

    /// Sets the displayed items.
    pub fn set_displayed_items(&mut self, items: Vec<Arc<TableItem>>) {
        self.displayed_items = items;
    }

    /// Clears the displayed items.
    pub fn clear_displayed_items(&mut self) {
        self.displayed_items.clear();
    }

    /// Extends the displayed items with additional items.
    pub fn extend_displayed_items(&mut self, items: impl IntoIterator<Item = Arc<TableItem>>) {
        self.displayed_items.extend(items);
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

    /// Gets a reference to the route generator.
    pub const fn get_route_generator(&self) -> &RouteGenerator {
        &self.route_generator
    }

    /// Gets a reference to the available airports from the route generator.
    pub fn get_available_airports(&self) -> &[Arc<Airport>] {
        &self.route_generator.all_airports
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
}
