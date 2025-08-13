use std::sync::Arc;

use crate::models::{Aircraft, Airport};
use crate::gui::data::TableItem;
use crate::gui::state::popup_state::DisplayMode;

/// State for UI-specific interactions and selections.
#[derive(Default)]
pub struct UiState {
    /// Currently selected aircraft for route generation.
    selected_aircraft: Option<Arc<Aircraft>>,
    /// Search text for aircraft selection.
    aircraft_search: String,
    /// Whether the aircraft dropdown is open.
    aircraft_dropdown_open: bool,
    /// Number of aircraft currently displayed in dropdown (for chunked loading).
    aircraft_dropdown_display_count: usize,
    /// The selected departure airport.
    departure_airport: Option<Arc<Airport>>,
    /// Cached validation result for departure airport
    departure_airport_valid: Option<bool>,
    /// Last validated departure airport ICAO to detect changes
    last_validated_departure_icao: String,
    /// Search text for departure airport selection.
    departure_airport_search: String,
    /// Whether the departure airport dropdown is open.
    departure_airport_dropdown_open: bool,
    /// Number of airports currently displayed in dropdown (for chunked loading).
    departure_airport_dropdown_display_count: usize,
    /// General search query for table filtering
    search_query: String,
    /// Filtered items for display
    filtered_items: Vec<TableItem>,
    /// All items cache
    all_items: Vec<TableItem>,
    /// Current display mode
    display_mode: DisplayMode,
    /// Whether popups are shown
    show_popup: bool,
    /// Selected route for popup display
    selected_route_for_popup: Option<crate::gui::data::ListItemRoute>,
    /// Whether search should execute
    search_should_execute: bool,
    /// Whether more routes are being loaded
    is_loading_more_routes: bool,
}

impl UiState {
    /// Creates a new UI state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets the currently selected aircraft for route generation.
    pub const fn get_selected_aircraft(&self) -> Option<&Arc<Aircraft>> {
        self.selected_aircraft.as_ref()
    }

    /// Gets the current aircraft search text.
    pub fn get_aircraft_search(&self) -> &str {
        &self.aircraft_search
    }

    /// Gets whether the aircraft dropdown is open.
    pub const fn is_aircraft_dropdown_open(&self) -> bool {
        self.aircraft_dropdown_open
    }

    /// Gets the current departure airport.
    pub const fn get_departure_airport(&self) -> Option<&Arc<Airport>> {
        self.departure_airport.as_ref()
    }

    /// Gets the current departure airport ICAO code.
    pub fn get_departure_airport_icao(&self) -> &str {
        self.departure_airport
            .as_ref()
            .map_or("", |airport| &airport.ICAO)
    }

    /// Sets the selected departure airport.
    pub fn set_departure_airport(&mut self, airport: Option<Arc<Airport>>) {
        self.departure_airport = airport;
        // Clear validation cache when departure changes
        self.clear_departure_validation_cache();
    }

    /// Gets the departure airport validation result if available.
    pub const fn get_departure_airport_validation(&self) -> Option<bool> {
        self.departure_airport_valid
    }

    /// Gets the current departure airport search text.
    pub fn get_departure_airport_search(&self) -> &str {
        &self.departure_airport_search
    }

    /// Gets whether the departure airport dropdown is open.
    pub const fn is_departure_airport_dropdown_open(&self) -> bool {
        self.departure_airport_dropdown_open
    }

    /// Sets the selected aircraft.
    pub fn set_selected_aircraft(&mut self, aircraft: Option<Arc<Aircraft>>) {
        self.selected_aircraft = aircraft;
    }

    /// Sets the aircraft search text.
    pub fn set_aircraft_search(&mut self, search: String) {
        self.aircraft_search = search;
    }

    /// Sets whether the aircraft dropdown is open.
    pub const fn set_aircraft_dropdown_open(&mut self, open: bool) {
        self.aircraft_dropdown_open = open;
    }

    /// Sets the departure airport search text.
    pub fn set_departure_airport_search(&mut self, search: String) {
        self.departure_airport_search = search;
    }

    /// Sets whether the departure airport dropdown is open.
    pub const fn set_departure_airport_dropdown_open(&mut self, open: bool) {
        self.departure_airport_dropdown_open = open;
    }

    /// Clears the departure airport validation cache.
    pub fn clear_departure_validation_cache(&mut self) {
        self.departure_airport_valid = None;
        self.last_validated_departure_icao.clear();
    }

    /// Sets the departure airport validation result and updates the cache.
    pub fn set_departure_validation(&mut self, icao: &str, is_valid: bool) {
        self.departure_airport_valid = Some(is_valid);
        self.last_validated_departure_icao = icao.to_string();
    }

    /// Resets the UI state to default values, optionally preserving the departure airport.
    pub fn reset(&mut self, preserve_departure: bool) {
        let departure_airport = if preserve_departure {
            self.departure_airport.clone()
        } else {
            None
        };

        let departure_valid = if preserve_departure {
            self.departure_airport_valid
        } else {
            None
        };

        let last_validated = if preserve_departure {
            self.last_validated_departure_icao.clone()
        } else {
            String::new()
        };

        *self = Self::default();

        if preserve_departure {
            self.departure_airport = departure_airport;
            self.departure_airport_valid = departure_valid;
            self.last_validated_departure_icao = last_validated;
        }
    }

    /// Gets a mutable reference to the aircraft dropdown display count.
    pub const fn get_aircraft_dropdown_display_count_mut(&mut self) -> &mut usize {
        &mut self.aircraft_dropdown_display_count
    }

    /// Gets a mutable reference to the departure airport dropdown display count.
    pub const fn get_departure_airport_dropdown_display_count_mut(&mut self) -> &mut usize {
        &mut self.departure_airport_dropdown_display_count
    }

    /// Gets the general search query.
    pub fn get_search_query(&self) -> &str {
        &self.search_query
    }

    /// Sets the general search query.
    pub fn set_search_query(&mut self, query: String) {
        self.search_query = query;
    }

    /// Gets the filtered items.
    pub fn get_filtered_items(&self) -> &[TableItem] {
        &self.filtered_items
    }

    /// Sets the filtered items.
    pub fn set_filtered_items(&mut self, items: Vec<TableItem>) {
        self.filtered_items = items;
    }

    /// Gets the all items cache.
    pub fn get_all_items(&self) -> &[TableItem] {
        &self.all_items
    }

    /// Sets the all items cache.
    pub fn set_all_items(&mut self, items: Vec<TableItem>) {
        self.all_items = items;
    }

    /// Gets the display mode.
    pub const fn get_display_mode(&self) -> &DisplayMode {
        &self.display_mode
    }

    /// Sets the display mode.
    pub fn set_display_mode(&mut self, mode: DisplayMode) {
        self.display_mode = mode;
    }

    /// Gets whether the popup is shown.
    pub const fn is_popup_shown(&self) -> bool {
        self.show_popup
    }

    /// Sets whether the popup is shown.
    pub const fn set_popup_shown(&mut self, shown: bool) {
        self.show_popup = shown;
    }

    /// Gets the selected route for popup.
    pub fn get_selected_route_for_popup(&self) -> Option<&crate::gui::data::ListItemRoute> {
        self.selected_route_for_popup.as_ref()
    }

    /// Sets the selected route for popup.
    pub fn set_selected_route_for_popup(&mut self, route: Option<crate::gui::data::ListItemRoute>) {
        self.selected_route_for_popup = route;
    }

    /// Gets whether search should execute.
    pub const fn get_search_should_execute(&self) -> bool {
        self.search_should_execute
    }

    /// Sets whether search should execute.
    pub const fn set_search_should_execute(&mut self, should_execute: bool) {
        self.search_should_execute = should_execute;
    }

    /// Gets whether more routes are being loaded.
    pub const fn is_loading_more_routes(&self) -> bool {
        self.is_loading_more_routes
    }

    /// Sets whether more routes are being loaded.
    pub const fn set_loading_more_routes(&mut self, loading: bool) {
        self.is_loading_more_routes = loading;
    }
}
