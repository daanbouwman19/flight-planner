use crate::gui::data::ListItemRoute;

/// State for handling popup dialogs and modal interactions.
#[derive(Default)]
pub struct PopupState {
    /// Whether to show the alert popup.
    show_alert: bool,
    /// The currently selected route.
    selected_route: Option<ListItemRoute>,
    /// Whether the routes are generated from not flown aircraft list.
    routes_from_not_flown: bool,
    /// Whether the current routes are for a specific aircraft.
    routes_for_specific_aircraft: bool,
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
        self.routes_from_not_flown
    }

    /// Gets whether routes are for a specific aircraft.
    pub const fn routes_for_specific_aircraft(&self) -> bool {
        self.routes_for_specific_aircraft
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
        self.routes_from_not_flown = from_not_flown;
    }

    /// Sets whether routes are for a specific aircraft.
    pub const fn set_routes_for_specific_aircraft(&mut self, for_specific: bool) {
        self.routes_for_specific_aircraft = for_specific;
    }
}
