use crate::gui::data::TableItem;
use crate::models::{Aircraft, Airport};
use crate::modules::data_operations::FlightStatistics;
use std::error::Error;
use std::sync::Arc;

/// Unified application state for the GUI.
/// Contains all UI-specific state (what to display, user interactions).
/// This is NOT business logic - that stays in services.
#[derive(Default)]
pub struct ApplicationState {
    // --- Selection State ---
    /// Currently selected departure airport.
    pub selected_departure_airport: Option<Arc<Airport>>,
    /// Currently selected aircraft.
    pub selected_aircraft: Option<Arc<Aircraft>>,

    // --- Search State ---
    /// Search query for departure airports.
    pub departure_search: String,
    /// Search query for aircraft.
    pub aircraft_search: String,
    /// Global search query for table items.
    pub table_search: String,

    // --- UI Interaction State ---
    /// Whether the departure airport dropdown is open.
    pub departure_dropdown_open: bool,
    /// Whether the aircraft dropdown is open.
    pub aircraft_dropdown_open: bool,
    /// Whether we're currently loading more routes.
    pub is_loading_more_routes: bool,
    /// Whether we're currently searching.
    pub is_searching: bool,
    /// Whether the "Add History" popup is visible.
    pub show_add_history_popup: bool,

    // --- "Add History" Popup State ---
    pub add_history_selected_aircraft: Option<Arc<Aircraft>>,
    pub add_history_selected_departure: Option<Arc<Airport>>,
    pub add_history_selected_destination: Option<Arc<Airport>>,
    pub add_history_aircraft_search: String,
    pub add_history_departure_search: String,
    pub add_history_destination_search: String,

    // --- Display Pagination ---
    /// Number of departure airports to display in dropdown.
    pub departure_display_count: usize,
    /// Number of aircraft to display in dropdown.
    pub aircraft_display_count: usize,
    /// Number of aircraft to display in the "Add History" popup.
    pub add_history_aircraft_display_count: usize,
    /// Number of departure airports to display in the "Add History" popup.
    pub add_history_departure_display_count: usize,
    /// Number of destination airports to display in the "Add History" popup.
    pub add_history_destination_display_count: usize,

    // --- Current Data Views ---
    /// All items available for the current table view.
    pub all_items: Vec<Arc<TableItem>>,

    // --- Statistics ---
    /// Cached flight statistics result
    pub statistics: Option<Result<FlightStatistics, Box<dyn Error + Send + Sync>>>,
}

impl ApplicationState {
    /// Creates a new application state with default values.
    pub fn new() -> Self {
        Self {
            departure_display_count: 50,
            aircraft_display_count: 50,
            add_history_aircraft_display_count: 50,
            add_history_departure_display_count: 50,
            add_history_destination_display_count: 50,
            ..Default::default()
        }
    }

    /// Gets the current search query for the table.
    pub fn table_search_query(&self) -> &str {
        &self.table_search
    }

    /// Sets the table search query.
    pub fn set_table_search(&mut self, query: String) {
        self.table_search = query;
    }

    /// Clears all search queries.
    pub fn clear_all_searches(&mut self) {
        self.departure_search.clear();
        self.aircraft_search.clear();
        self.table_search.clear();
    }

    /// Resets dropdown states.
    pub fn close_all_dropdowns(&mut self) {
        self.departure_dropdown_open = false;
        self.aircraft_dropdown_open = false;
    }

    /// Checks if any dropdown is open.
    pub fn has_open_dropdown(&self) -> bool {
        self.departure_dropdown_open || self.aircraft_dropdown_open
    }

    /// Gets a display name for the selected departure airport.
    pub fn departure_display_text(&self) -> &str {
        if let Some(airport) = &self.selected_departure_airport {
            &airport.Name
        } else if !self.departure_search.is_empty() {
            &self.departure_search
        } else {
            "Select departure airport"
        }
    }

    /// Gets a display name for the selected aircraft.
    pub fn aircraft_display_text(&self) -> &str {
        if let Some(aircraft) = &self.selected_aircraft {
            &aircraft.variant
        } else if !self.aircraft_search.is_empty() {
            &self.aircraft_search
        } else {
            "Select aircraft"
        }
    }

    /// Checks if we have valid selections for route generation.
    pub fn can_generate_routes(&self) -> bool {
        self.selected_departure_airport.is_some() && self.selected_aircraft.is_some()
    }

    /// Selects a departure airport and updates related state.
    pub fn select_departure_airport(&mut self, airport: Option<Arc<Airport>>) {
        self.selected_departure_airport = airport;
        self.departure_dropdown_open = false;
        if self.selected_departure_airport.is_some() {
            self.departure_search.clear();
        }
    }

    /// Selects an aircraft and updates related state.
    pub fn select_aircraft(&mut self, aircraft: Option<Arc<Aircraft>>) {
        self.selected_aircraft = aircraft;
        self.aircraft_dropdown_open = false;
        if self.selected_aircraft.is_some() {
            self.aircraft_search.clear();
        }
    }
}
