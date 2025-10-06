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

/// A service dedicated to managing the main display mode.
pub struct ViewModeService {
    /// The current display mode of the main content area.
    display_mode: DisplayMode,
}

impl Default for ViewModeService {
    fn default() -> Self {
        Self::new()
    }
}

impl ViewModeService {
    /// Creates a new `ViewModeService` with default values.
    pub fn new() -> Self {
        Self {
            display_mode: DisplayMode::default(),
        }
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
}