use crate::gui::data::ListItemRoute;

/// Defines the different content types that can be displayed in the main view.
///
/// This enum is used to control which set of data is shown in the central table
/// and to tailor the UI controls accordingly.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub enum DisplayMode {
    /// Displaying randomly generated routes from all available aircraft.
    #[default]
    RandomRoutes,
    /// Displaying routes generated exclusively from aircraft that have not yet been flown.
    NotFlownRoutes,
    /// Displaying routes generated for a single, specifically selected aircraft.
    SpecificAircraftRoutes,
    /// Displaying a list of randomly selected airports with runway information.
    RandomAirports,
    /// Displaying the user's flight history.
    History,
    /// Displaying a list of all available airports.
    Airports,
    /// Displaying flight statistics.
    Statistics,
    /// A catch-all for other display types, such as the full aircraft list.
    Other,
}

/// A service dedicated to managing the state of popups and the main display mode.
///
/// This service encapsulates the logic for showing and hiding popups, tracking
/// the currently selected item for a popup, and managing the overall display
/// mode of the application's main view.
pub struct PopupService {
    /// A flag indicating whether a popup/alert is currently visible.
    show_alert: bool,
    /// The route that has been selected for display in the route details popup.
    selected_route: Option<ListItemRoute>,
    /// The current display mode of the main content area.
    display_mode: DisplayMode,
}

impl Default for PopupService {
    fn default() -> Self {
        Self::new()
    }
}

impl PopupService {
    /// Creates a new `PopupService` with default values.
    pub fn new() -> Self {
        Self {
            show_alert: false,
            selected_route: None,
            display_mode: DisplayMode::default(),
        }
    }

    // --- Alert Management ---

    /// Checks if a popup/alert is currently visible.
    pub fn is_alert_visible(&self) -> bool {
        self.show_alert
    }

    /// Sets the alert state to visible.
    pub fn show_alert(&mut self) {
        self.show_alert = true;
    }

    /// Sets the alert state to hidden.
    pub fn hide_alert(&mut self) {
        self.show_alert = false;
    }

    /// Sets the visibility of the alert to a specific state.
    ///
    /// # Arguments
    ///
    /// * `visible` - `true` to show the alert, `false` to hide it.
    pub fn set_alert_visibility(&mut self, visible: bool) {
        self.show_alert = visible;
    }

    // --- Route Selection ---

    /// Returns a reference to the currently selected route, if any.
    pub fn selected_route(&self) -> Option<&ListItemRoute> {
        self.selected_route.as_ref()
    }

    /// Sets the currently selected route.
    ///
    /// # Arguments
    ///
    /// * `route` - The `ListItemRoute` to be selected.
    pub fn select_route(&mut self, route: ListItemRoute) {
        self.selected_route = Some(route);
    }

    /// Clears the currently selected route.
    pub fn clear_route_selection(&mut self) {
        self.selected_route = None;
    }

    /// Sets or clears the selected route.
    ///
    /// # Arguments
    ///
    /// * `route` - An `Option<ListItemRoute>` to set as the current selection.
    pub fn set_selected_route(&mut self, route: Option<ListItemRoute>) {
        self.selected_route = route;
    }

    // --- Display Mode Management ---

    /// Returns a reference to the current `DisplayMode`.
    pub fn display_mode(&self) -> &DisplayMode {
        &self.display_mode
    }

    /// Sets the application's current `DisplayMode`.
    ///
    /// # Arguments
    ///
    /// * `mode` - The `DisplayMode` to set.
    pub fn set_display_mode(&mut self, mode: DisplayMode) {
        self.display_mode = mode;
    }

    // --- Business Logic Queries ---

    /// Checks if the current display mode is any of the route-related modes.
    pub fn is_route_mode(&self) -> bool {
        matches!(
            self.display_mode,
            DisplayMode::RandomRoutes
                | DisplayMode::NotFlownRoutes
                | DisplayMode::SpecificAircraftRoutes
        )
    }

    /// Checks if the current display mode is for routes from not-flown aircraft.
    pub fn routes_from_not_flown(&self) -> bool {
        matches!(self.display_mode, DisplayMode::NotFlownRoutes)
    }

    /// Checks if the current display mode is for routes for a specific aircraft.
    pub fn routes_for_specific_aircraft(&self) -> bool {
        matches!(self.display_mode, DisplayMode::SpecificAircraftRoutes)
    }

    /// Checks if the current display mode is for showing random airports.
    pub fn is_showing_random_airports(&self) -> bool {
        matches!(self.display_mode, DisplayMode::RandomAirports)
    }

    /// Checks if the current display mode is for showing statistics.
    pub fn is_showing_statistics(&self) -> bool {
        matches!(self.display_mode, DisplayMode::Statistics)
    }

    // --- Mode Transitions ---

    /// Determines the appropriate route mode based on whether an aircraft is selected.
    ///
    /// # Arguments
    ///
    /// * `has_selected_aircraft` - `true` if an aircraft is currently selected.
    ///
    /// # Returns
    ///
    /// The `DisplayMode` that should be active.
    pub fn get_appropriate_route_mode(&self, has_selected_aircraft: bool) -> DisplayMode {
        if has_selected_aircraft {
            DisplayMode::SpecificAircraftRoutes
        } else {
            DisplayMode::RandomRoutes
        }
    }

    /// Determines if the application should switch to a route mode.
    ///
    /// This is typically true when a user makes a selection (like an aircraft or
    /// airport) and the UI is not already in a route display mode.
    ///
    /// # Arguments
    ///
    /// * `selection_being_made` - `true` if the user is actively selecting an item.
    pub fn should_switch_to_route_mode(&self, selection_being_made: bool) -> bool {
        selection_being_made && !self.is_route_mode()
    }
}
