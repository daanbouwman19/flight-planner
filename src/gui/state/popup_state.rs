use crate::gui::data::ListItemRoute;

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

/// State for handling popup dialogs and modal interactions.
#[derive(Default)]
pub struct PopupState {
    /// Whether to show the alert popup.
    show_alert: bool,
    /// The currently selected route.
    selected_route: Option<ListItemRoute>,
    /// The current display mode for items.
    display_mode: DisplayMode,
}

impl PopupState {
    /// Creates a new popup state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets whether to show the alert popup.
    pub const fn show_alert(&self) -> bool {
        self.show_alert
    }

    /// Gets the currently selected route.
    pub const fn get_selected_route(&self) -> Option<&ListItemRoute> {
        self.selected_route.as_ref()
    }

    /// Gets whether routes are from not flown aircraft.
    pub const fn routes_from_not_flown(&self) -> bool {
        matches!(self.display_mode, DisplayMode::NotFlownRoutes)
    }

    /// Gets whether routes are for a specific aircraft.
    pub const fn routes_for_specific_aircraft(&self) -> bool {
        matches!(self.display_mode, DisplayMode::SpecificAircraftRoutes)
    }

    /// Gets whether the current items are random airports.
    pub const fn airports_random(&self) -> bool {
        matches!(self.display_mode, DisplayMode::RandomAirports)
    }

    /// Sets whether to show the alert popup.
    pub const fn set_show_alert(&mut self, show: bool) {
        self.show_alert = show;
    }

    /// Sets the selected route in the popup state.
    pub fn set_selected_route(&mut self, route: Option<ListItemRoute>) {
        self.selected_route = route;
    }

    /// Sets the display mode directly.
    pub fn set_display_mode(&mut self, mode: DisplayMode) {
        self.display_mode = mode;
    }

    /// Gets the current display mode.
    pub const fn get_display_mode(&self) -> &DisplayMode {
        &self.display_mode
    }
}
