use crate::gui::data::{ListItemRoute, TableItem};
use crate::models::{Aircraft, Airport};
use crate::modules::data_operations::FlightStatistics;
use std::error::Error;
use std::sync::Arc;
use std::time::Instant;

/// Represents the type of items currently being displayed.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub enum DisplayMode {
    /// Regular routes from all aircraft.
    #[default]
    RandomRoutes,
    /// Routes generated from not flown aircraft only.
    NotFlownRoutes,
    /// Routes for a specific selected aircraft.
    SpecificAircraftRoutes,
    /// Random airports with runway information.
    RandomAirports,
    /// Flight history display.
    History,
    /// Airport list display.
    Airports,
    /// Flight statistics display.
    Statistics,
    /// Other items (history, aircraft list, etc.).
    Other,
}

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

    // --- UI Interaction State ---
    /// Whether the departure airport dropdown is open.
    pub departure_dropdown_open: bool,
    /// Whether the aircraft dropdown is open.
    pub aircraft_dropdown_open: bool,
    /// Whether we're currently loading more routes.
    pub is_loading_more_routes: bool,

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
            ..Default::default()
        }
    }
}

/// UI-specific state that is not part of the core application logic.
#[derive(Default)]
pub struct ViewState {
    // --- Search State ---
    /// Global search query for table items.
    pub table_search: String,
    /// The items filtered based on the search query (temporary cache).
    pub filtered_items: Vec<Arc<TableItem>>,
    /// The last time a search was requested (for debouncing).
    pub last_search_request: Option<Instant>,
    /// Whether a search is pending (for debouncing).
    pub search_pending: bool,

    // --- Popup State ---
    /// Whether to show the route popup.
    pub show_route_popup: bool,
    /// The route selected for the popup.
    pub selected_route_for_popup: Option<ListItemRoute>,

    // --- Display State ---
    /// The current display mode.
    pub display_mode: DisplayMode,
}

impl ViewState {
    /// Creates a new view state with default values.
    pub fn new() -> Self {
        Self::default()
    }
}
