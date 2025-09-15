use crate::gui::data::ListItemRoute;
use crate::gui::state::{DisplayMode, ViewState};

/// Service for handling popup dialogs and modal interactions.
/// This is a **Model** in MVVM - it contains business logic for popup management.
/// This service is now stateless and operates on the `ViewState`.
pub struct PopupService;

impl Default for PopupService {
    fn default() -> Self {
        Self::new()
    }
}

impl PopupService {
    /// Creates a new popup service.
    pub fn new() -> Self {
        Self
    }

    // --- Alert Management ---

    pub fn is_alert_visible(&self, view_state: &ViewState) -> bool {
        view_state.show_route_popup
    }

    pub fn set_alert_visibility(&self, view_state: &mut ViewState, visible: bool) {
        view_state.show_route_popup = visible;
    }

    // --- Route Selection ---

    pub fn selected_route<'a>(&self, view_state: &'a ViewState) -> Option<&'a ListItemRoute> {
        view_state.selected_route_for_popup.as_ref()
    }

    pub fn set_selected_route(&self, view_state: &mut ViewState, route: Option<ListItemRoute>) {
        view_state.selected_route_for_popup = route;
    }

    // --- Display Mode Management ---

    pub fn display_mode<'a>(&self, view_state: &'a ViewState) -> &'a DisplayMode {
        &view_state.display_mode
    }

    pub fn set_display_mode(&self, view_state: &mut ViewState, mode: DisplayMode) {
        view_state.display_mode = mode;
    }

    // --- Business Logic Queries ---

    pub fn routes_from_not_flown(&self, view_state: &ViewState) -> bool {
        matches!(view_state.display_mode, DisplayMode::NotFlownRoutes)
    }
}
