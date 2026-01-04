use crate::gui::components::dropdowns::INITIAL_DISPLAY_COUNT;
use crate::gui::data::TableItem;
use crate::gui::services::popup_service::DisplayMode;
use crate::models::{Aircraft, Airport};
use crate::modules::data_operations::FlightStatistics;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

/// Holds the state for the "Add History" popup window.
///
/// This struct encapsulates all data and UI state related to adding a new
/// flight history entry, such as selected values, search queries, and the
/// visibility of dropdown menus.
#[derive(Default)]
pub struct AddHistoryState {
    /// Controls the visibility of the "Add History" popup.
    pub show_popup: bool,
    /// The aircraft selected for the new history entry.
    pub selected_aircraft: Option<Arc<Aircraft>>,
    /// The departure airport selected for the new history entry.
    pub selected_departure: Option<Arc<Airport>>,
    /// The destination airport selected for the new history entry.
    pub selected_destination: Option<Arc<Airport>>,
    /// The search text for the aircraft selection dropdown.
    pub aircraft_search: String,
    /// The search text for the departure airport selection dropdown.
    pub departure_search: String,
    /// The search text for the destination airport selection dropdown.
    pub destination_search: String,
    /// A flag indicating whether the aircraft selection dropdown is open.
    pub aircraft_dropdown_open: bool,
    /// A flag indicating whether the departure airport selection dropdown is open.
    pub departure_dropdown_open: bool,
    /// A flag indicating whether the destination airport selection dropdown is open.
    pub destination_dropdown_open: bool,
    /// A flag to control autofocus on the aircraft search input.
    pub aircraft_search_autofocus: bool,
    /// A flag to control autofocus on the departure search input.
    pub departure_search_autofocus: bool,
    /// A flag to control autofocus on the destination search input.
    pub destination_search_autofocus: bool,
    /// The number of aircraft to display in the selection dropdown.
    pub aircraft_display_count: usize,
    /// The number of departure airports to display in the selection dropdown.
    pub departure_display_count: usize,
    /// The number of destination airports to display in the selection dropdown.
    pub destination_display_count: usize,
}

impl AddHistoryState {
    /// Creates a new `AddHistoryState` with initial display counts for dropdowns.
    pub fn new() -> Self {
        Self {
            aircraft_display_count: INITIAL_DISPLAY_COUNT,
            departure_display_count: INITIAL_DISPLAY_COUNT,
            destination_display_count: INITIAL_DISPLAY_COUNT,
            ..Default::default()
        }
    }

    /// Toggles the visibility of the aircraft selection dropdown.
    ///
    /// This method also ensures that only one dropdown is open at a time by
    /// closing the others.
    pub fn toggle_aircraft_dropdown(&mut self) {
        self.aircraft_dropdown_open = !self.aircraft_dropdown_open;
        self.aircraft_search_autofocus = self.aircraft_dropdown_open;
        // Ensure other dropdowns are closed
        self.departure_dropdown_open = false;
        self.destination_dropdown_open = false;
    }

    /// Toggles the visibility of the departure airport selection dropdown.
    ///
    /// This method also ensures that only one dropdown is open at a time.
    pub fn toggle_departure_dropdown(&mut self) {
        self.departure_dropdown_open = !self.departure_dropdown_open;
        self.departure_search_autofocus = self.departure_dropdown_open;
        // Ensure other dropdowns are closed
        self.aircraft_dropdown_open = false;
        self.destination_dropdown_open = false;
    }

    /// Toggles the visibility of the destination airport selection dropdown.
    ///
    /// This method also ensures that only one dropdown is open at a time.
    pub fn toggle_destination_dropdown(&mut self) {
        self.destination_dropdown_open = !self.destination_dropdown_open;
        self.destination_search_autofocus = self.destination_dropdown_open;
        // Ensure other dropdowns are closed
        self.aircraft_dropdown_open = false;
        self.departure_dropdown_open = false;
    }
}

/// Represents the unified state of the entire GUI application.
///
/// This struct centralizes all state management, including user selections,
/// search queries, UI interaction flags, and data for views.
#[derive(Default)]
pub struct ApplicationState {
    // --- Selection State ---
    /// The currently selected departure airport, if any.
    pub selected_departure_airport: Option<Arc<Airport>>,
    /// The currently selected aircraft, if any.
    pub selected_aircraft: Option<Arc<Aircraft>>,

    // --- Search State ---
    /// The text entered in the departure airport search box.
    pub departure_search: String,
    /// The text entered in the aircraft search box.
    pub aircraft_search: String,

    // --- UI Interaction State ---
    /// A flag indicating whether the departure airport selection dropdown is open.
    pub departure_dropdown_open: bool,
    /// A flag indicating whether the aircraft selection dropdown is open.
    pub aircraft_dropdown_open: bool,
    /// A flag to indicate that more routes are currently being loaded in the background.
    pub is_loading_more_routes: bool,

    // --- "Settings" Popup State ---
    /// Controls the visibility of the "Settings" popup.
    pub show_settings_popup: bool,
    /// The API key for the AVWX service.
    pub api_key: String,

    // --- "Add History" Popup State ---
    /// The state for the "Add History" popup window.
    pub add_history: AddHistoryState,

    // --- Display Pagination ---
    /// The number of departure airports to show in the dropdown list.
    pub departure_display_count: usize,
    /// The number of aircraft to show in the dropdown list.
    pub aircraft_display_count: usize,

    // --- Current Data Views ---
    /// A vector holding all items for the current table view (e.g., routes, aircraft, history).
    pub all_items: Vec<Arc<TableItem>>,

    // --- Statistics ---
    /// A cached result of the flight statistics calculation.
    pub statistics: Option<Result<FlightStatistics, Box<dyn Error + Send + Sync>>>,

    // --- Table Layout ---
    /// Stores the relative column widths (as ratios 0.0-1.0) for each display mode.
    /// This allows for manual resizing that persists relatively when the window is resized.
    pub column_widths: HashMap<DisplayMode, Vec<f32>>,
}

impl ApplicationState {
    /// Creates a new `ApplicationState` with default values.
    pub fn new() -> Self {
        Self {
            departure_display_count: 50,
            aircraft_display_count: 50,
            add_history: AddHistoryState::new(),
            column_widths: HashMap::new(),
            ..Default::default()
        }
    }

    /// Closes all selection dropdowns in the UI.
    pub fn close_all_dropdowns(&mut self) {
        self.departure_dropdown_open = false;
        self.aircraft_dropdown_open = false;
    }

    /// Determines the appropriate text to display for the departure airport selection.
    ///
    /// It prioritizes the selected airport's name, falls back to the search text,
    /// and finally shows a default prompt.
    ///
    /// # Returns
    ///
    /// A string slice representing the text to be displayed.
    pub fn departure_display_text(&self) -> &str {
        if let Some(airport) = &self.selected_departure_airport {
            &airport.Name
        } else if !self.departure_search.is_empty() {
            &self.departure_search
        } else {
            "Select departure airport"
        }
    }

    /// Determines the appropriate text to display for the aircraft selection.
    ///
    /// It prioritizes the selected aircraft's variant, falls back to the search
    /// text, and finally shows a default prompt.
    ///
    /// # Returns
    ///
    /// A string slice representing the text to be displayed.
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
