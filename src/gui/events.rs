use crate::gui::data::ListItemRoute;
use crate::gui::state::DisplayMode;
use crate::models::{Aircraft, Airport};
use std::sync::Arc;

/// Represents all possible events that can be triggered by the UI.
#[derive(Debug, Clone)]
pub enum Event {
    // --- SelectionControls Events ---
    DepartureAirportSelected(Option<Arc<Airport>>),
    AircraftSelected(Option<Arc<Aircraft>>),
    ToggleDepartureAirportDropdown,
    ToggleAircraftDropdown,

    // --- ActionButtons Events ---
    SetDisplayMode(DisplayMode),
    RegenerateRoutesForSelectionChange,

    // --- TableDisplay Events ---
    RouteSelectedForPopup(ListItemRoute),
    SetShowPopup(bool),
    ToggleAircraftFlownStatus(i32),
    LoadMoreRoutes,

    // --- SearchControls Events ---
    SearchQueryChanged,
    ClearSearch,
}
