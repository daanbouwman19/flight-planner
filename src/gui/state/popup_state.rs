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

    /// Sets whether routes are from not flown aircraft.
    pub const fn set_routes_from_not_flown(&mut self, from_not_flown: bool) {
        if from_not_flown {
            self.display_mode = DisplayMode::NotFlownRoutes;
        } else if matches!(self.display_mode, DisplayMode::NotFlownRoutes) {
            self.display_mode = DisplayMode::RandomRoutes;
        }
    }

    /// Sets whether routes are for a specific aircraft.
    pub const fn set_routes_for_specific_aircraft(&mut self, for_specific: bool) {
        if for_specific {
            self.display_mode = DisplayMode::SpecificAircraftRoutes;
        } else if matches!(self.display_mode, DisplayMode::SpecificAircraftRoutes) {
            self.display_mode = DisplayMode::RandomRoutes;
        }
    }

    /// Sets whether the current items are random airports.
    pub const fn set_airports_random(&mut self, airports_random: bool) {
        if airports_random {
            self.display_mode = DisplayMode::RandomAirports;
        } else if matches!(self.display_mode, DisplayMode::RandomAirports) {
            self.display_mode = DisplayMode::Other;
        }
    }
}
