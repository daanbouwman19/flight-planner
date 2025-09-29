use crate::gui::data::TableItem;
use crate::models::{Aircraft, Airport};
use crate::modules::data_operations::FlightStatistics;
use std::error::Error;
use std::sync::Arc;

/// State specific to the "Add History" popup.
#[derive(Default)]
pub struct AddHistoryState {
    pub show_popup: bool,
    pub selected_aircraft: Option<Arc<Aircraft>>,
    pub selected_departure: Option<Arc<Airport>>,
    pub selected_destination: Option<Arc<Airport>>,
    pub aircraft_search: String,
    pub departure_search: String,
    pub destination_search: String,
    pub aircraft_dropdown_open: bool,
    pub departure_dropdown_open: bool,
    pub destination_dropdown_open: bool,
    pub aircraft_search_autofocus: bool,
    pub departure_search_autofocus: bool,
    pub destination_search_autofocus: bool,
    pub aircraft_display_count: usize,
    pub departure_display_count: usize,
    pub destination_display_count: usize,
}

impl AddHistoryState {
    /// Creates a new `AddHistoryState` with default values.
    pub fn new() -> Self {
        Self {
            aircraft_display_count:
                crate::gui::components::add_history_popup::INITIAL_DISPLAY_COUNT,
            departure_display_count:
                crate::gui::components::add_history_popup::INITIAL_DISPLAY_COUNT,
            destination_display_count:
                crate::gui::components::add_history_popup::INITIAL_DISPLAY_COUNT,
            ..Default::default()
        }
    }
}

/// Unified application state for the GUI.
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

    // --- UI Interaction State ---
    /// Whether the departure airport dropdown is open.
    pub departure_dropdown_open: bool,
    /// Whether the aircraft dropdown is open.
    pub aircraft_dropdown_open: bool,
    /// Whether we're currently loading more routes.
    pub is_loading_more_routes: bool,

    // --- "Add History" Popup State ---
    pub add_history: AddHistoryState,

    // --- Display Pagination ---
    /// Number of departure airports to display in dropdown.
    pub departure_display_count: usize,
    /// Number of aircraft to display in dropdown.
    pub aircraft_display_count: usize,

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
            add_history: AddHistoryState::new(),
            ..Default::default()
        }
    }

    /// Resets dropdown states.
    pub fn close_all_dropdowns(&mut self) {
        self.departure_dropdown_open = false;
        self.aircraft_dropdown_open = false;
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
}
