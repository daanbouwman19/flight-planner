use crate::{
    gui::data::ListItemRoute,
    gui::services::popup_service::DisplayMode,
    models::{Aircraft, Airport},
};
use std::sync::Arc;

/// Represents all possible UI actions that can be triggered by components.
///
/// This enum is central to the application's event-driven architecture. Each
/// variant corresponds to a specific user interaction or state change request
/// that originates from a UI component.
#[derive(Debug, Clone)]
pub enum Event {
    // --- SelectionControls Events ---
    /// A new departure airport has been selected from the dropdown.
    DepartureAirportSelected(Option<Arc<Airport>>),
    /// A new aircraft has been selected from the dropdown.
    AircraftSelected(Option<Arc<Aircraft>>),
    /// The user has clicked to toggle the visibility of the departure airport dropdown.
    ToggleDepartureAirportDropdown,
    /// The user has clicked to toggle the visibility of the aircraft dropdown.
    ToggleAircraftDropdown,

    // --- ActionButtons Events ---
    /// The user has selected a new display mode (e.g., "Random Routes", "History").
    SetDisplayMode(DisplayMode),
    /// A route regeneration has been triggered due to a change in selections.
    RegenerateRoutesForSelectionChange,

    // --- TableDisplay Events ---
    /// A route in the table has been selected to be shown in the details popup.
    RouteSelectedForPopup(ListItemRoute),
    /// A request to force the table to scroll to the top.
    ScrollTableToTop,
    /// A request to explicitly set the visibility of the popup.
    SetShowPopup(bool),
    /// The user has clicked to toggle the "flown" status of an aircraft.
    ToggleAircraftFlownStatus(i32),
    /// The user has clicked to mark all aircraft as not flown.
    MarkAllAircraftAsNotFlown,
    /// A request to load more routes, typically for infinite scrolling.
    LoadMoreRoutes,

    // --- SearchControls Events ---
    /// The text in the search query input has been updated.
    SearchQueryChanged,
    /// The user has cleared the search query.
    ClearSearch,

    // --- RoutePopup Events ---
    /// The user has confirmed to mark a route as flown from the popup.
    MarkRouteAsFlown(ListItemRoute),
    /// A request to close the currently open popup dialog.
    ClosePopup,

    // --- AddHistoryPopup Events ---
    /// A request to show the "Add History" popup.
    ShowAddHistoryPopup,
    /// A request to close the "Add History" popup.
    CloseAddHistoryPopup,
    /// The user has submitted the form to add a new entry to the flight history.
    AddHistoryEntry {
        aircraft: Arc<Aircraft>,
        departure: Arc<Airport>,
        destination: Arc<Airport>,
    },
    /// Toggles the aircraft selection dropdown within the "Add History" popup.
    ToggleAddHistoryAircraftDropdown,
    /// Toggles the departure airport selection dropdown within the "Add History" popup.
    ToggleAddHistoryDepartureDropdown,
    /// Toggles the destination airport selection dropdown within the "Add History" popup.
    ToggleAddHistoryDestinationDropdown,

    // --- SettingsPopup Events ---
    /// A request to show the "Settings" popup.
    ShowSettingsPopup,
    /// A request to close the "Settings" popup.
    CloseSettingsPopup,
    /// The user has clicked to save the settings.
    SaveSettings,

    // --- Table Layout Events ---
    /// A column in the table has been resized manually.
    ColumnResized {
        mode: DisplayMode,
        index: usize,
        delta: f32,
        total_width: f32,
    },
}
