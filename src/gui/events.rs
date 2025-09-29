use crate::{
    gui::data::ListItemRoute,
    gui::services::popup_service::DisplayMode,
    models::{Aircraft, Airport},
};
use std::sync::Arc;

/// Represents all possible UI actions that can be triggered by components.
#[derive(Debug, Clone)]
pub enum Event {
    // --- SelectionControls Events ---
    /// A new departure airport has been selected.
    DepartureAirportSelected(Option<Arc<Airport>>),
    /// A new aircraft has been selected.
    AircraftSelected(Option<Arc<Aircraft>>),
    /// Toggles the visibility of the departure airport dropdown.
    ToggleDepartureAirportDropdown,
    /// Toggles the visibility of the aircraft dropdown.
    ToggleAircraftDropdown,

    // --- ActionButtons Events ---
    /// Sets the current display mode.
    SetDisplayMode(DisplayMode),
    /// Triggers a regeneration of routes based on current selections.
    RegenerateRoutesForSelectionChange,

    // --- TableDisplay Events ---
    /// A route has been selected to be shown in a popup.
    RouteSelectedForPopup(ListItemRoute),
    /// Sets the visibility of the popup.
    SetShowPopup(bool),
    /// Toggles the flown status of an aircraft.
    ToggleAircraftFlownStatus(i32),
    /// Requests to load more routes for infinite scrolling.
    LoadMoreRoutes,

    // --- SearchControls Events ---
    /// The search query has been updated.
    SearchQueryChanged,
    /// The search query has been cleared.
    ClearSearch,

    // --- RoutePopup Events ---
    /// Marks a route as flown.
    MarkRouteAsFlown(ListItemRoute),
    /// Closes the popup dialog.
    ClosePopup,

    // --- AddHistoryPopup Events ---
    /// Shows the "Add History" popup.
    ShowAddHistoryPopup,
    /// Closes the "Add History" popup.
    CloseAddHistoryPopup,
    /// Adds a new entry to the flight history.
    AddHistoryEntry {
        aircraft: Arc<Aircraft>,
        departure: Arc<Airport>,
        destination: Arc<Airport>,
    },
    /// Toggles the aircraft dropdown in the "Add History" popup.
    ToggleAddHistoryAircraftDropdown,
    /// Toggles the departure dropdown in the "Add History" popup.
    ToggleAddHistoryDepartureDropdown,
    /// Toggles the destination dropdown in the "Add History" popup.
    ToggleAddHistoryDestinationDropdown,
}
