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

/// Service for handling popup dialogs and modal interactions.
/// This is a **Model** in MVVM - it contains business logic for popup management.
pub struct PopupService {
    show_alert: bool,
    selected_route: Option<ListItemRoute>,
    display_mode: DisplayMode,
}

impl Default for PopupService {
    fn default() -> Self {
        Self::new()
    }
}

impl PopupService {
    /// Creates a new popup service.
    pub fn new() -> Self {
        Self {
            show_alert: false,
            selected_route: None,
            display_mode: DisplayMode::default(),
        }
    }

    // --- Alert Management ---

    pub fn is_alert_visible(&self) -> bool {
        self.show_alert
    }

    pub fn show_alert(&mut self) {
        self.show_alert = true;
    }

    pub fn hide_alert(&mut self) {
        self.show_alert = false;
    }

    pub fn set_alert_visibility(&mut self, visible: bool) {
        self.show_alert = visible;
    }

    // --- Route Selection ---

    pub fn selected_route(&self) -> Option<&ListItemRoute> {
        self.selected_route.as_ref()
    }

    pub fn select_route(&mut self, route: ListItemRoute) {
        self.selected_route = Some(route);
    }

    pub fn clear_route_selection(&mut self) {
        self.selected_route = None;
    }

    pub fn set_selected_route(&mut self, route: Option<ListItemRoute>) {
        self.selected_route = route;
    }

    // --- Display Mode Management ---

    pub fn display_mode(&self) -> &DisplayMode {
        &self.display_mode
    }

    pub fn set_display_mode(&mut self, mode: DisplayMode) {
        self.display_mode = mode;
    }

    // --- Business Logic Queries ---

    pub fn is_route_mode(&self) -> bool {
        matches!(
            self.display_mode,
            DisplayMode::RandomRoutes
                | DisplayMode::NotFlownRoutes
                | DisplayMode::SpecificAircraftRoutes
        )
    }

    pub fn routes_from_not_flown(&self) -> bool {
        matches!(self.display_mode, DisplayMode::NotFlownRoutes)
    }

    pub fn routes_for_specific_aircraft(&self) -> bool {
        matches!(self.display_mode, DisplayMode::SpecificAircraftRoutes)
    }

    pub fn is_showing_random_airports(&self) -> bool {
        matches!(self.display_mode, DisplayMode::RandomAirports)
    }

    pub fn is_showing_statistics(&self) -> bool {
        matches!(self.display_mode, DisplayMode::Statistics)
    }

    // --- Mode Transitions ---

    pub fn get_appropriate_route_mode(&self, has_selected_aircraft: bool) -> DisplayMode {
        if has_selected_aircraft {
            DisplayMode::SpecificAircraftRoutes
        } else {
            DisplayMode::RandomRoutes
        }
    }

    pub fn should_switch_to_route_mode(&self, selection_being_made: bool) -> bool {
        selection_being_made && !self.is_route_mode()
    }
}
