use crate::gui::components::dropdowns::{
    DropdownAction, render_aircraft_dropdown, render_airport_dropdown,
};
use crate::gui::events::Event;
use crate::gui::state::AddHistoryState;
use crate::models::{Aircraft, Airport};
use std::sync::Arc;

// --- Component ---

/// A UI component that renders a popup window for adding a new flight history entry.
///
/// This component provides a form with searchable dropdowns for selecting an
/// aircraft, departure airport, and destination airport.
pub struct AddHistoryPopup;

impl AddHistoryPopup {
    /// Renders the "Add History" popup window and handles its interactions.
    ///
    /// # Arguments
    ///
    /// * `all_aircraft` - A slice of all available aircraft for the selection dropdown.
    /// * `all_airports` - A slice of all available airports for the selection dropdowns.
    /// * `state` - A mutable reference to the `AddHistoryState` for managing the popup's state.
    /// * `ctx` - The `egui::Context` for rendering the window.
    ///
    /// # Returns
    ///
    /// A `Vec<Event>` containing any events triggered by user interaction within the popup.
    pub fn render(
        all_aircraft: &[Arc<Aircraft>],
        all_airports: &[Arc<Airport>],
        state: &mut AddHistoryState,
        ctx: &egui::Context,
    ) -> Vec<Event> {
        let mut events = Vec::new();
        let mut open = true;

        egui::Window::new("Add History Entry")
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.set_width(320.0);
                ui.vertical_centered_justified(|ui| {
                    events.extend(Self::aircraft_selection(all_aircraft, state, ui));
                    events.extend(Self::departure_selection(all_airports, state, ui));
                    events.extend(Self::destination_selection(all_airports, state, ui));

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    let mut missing_fields = Vec::new();
                    if state.selected_aircraft.is_none() {
                        missing_fields.push("Select an aircraft");
                    }
                    if state.selected_departure.is_none() {
                        missing_fields.push("Select a departure airport");
                    }
                    if state.selected_destination.is_none() {
                        missing_fields.push("Select a destination airport");
                    } else if state.selected_departure == state.selected_destination {
                        missing_fields.push("Departure and destination must be different");
                    }

                    let add_button_enabled = missing_fields.is_empty();

                    let tooltip = if add_button_enabled {
                        "Add this flight to your history".to_string()
                    } else {
                        format!("Cannot add flight:\n• {}", missing_fields.join("\n• "))
                    };

                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            events.push(Event::CloseAddHistoryPopup);
                        }

                        if ui
                            .add_enabled(add_button_enabled, egui::Button::new("Add"))
                            .on_hover_text(&tooltip)
                            .on_disabled_hover_text(&tooltip)
                            .clicked()
                        {
                            // The `add_button_enabled` check guarantees these are `Some`.
                            let aircraft = state.selected_aircraft.clone().unwrap();
                            let departure = state.selected_departure.clone().unwrap();
                            let destination = state.selected_destination.clone().unwrap();
                            events.push(Event::AddHistoryEntry {
                                aircraft,
                                departure,
                                destination,
                            });
                        }
                    });
                });
            });

        if !open {
            events.push(Event::CloseAddHistoryPopup);
        }

        events
    }

    fn aircraft_selection(
        all_aircraft: &[Arc<Aircraft>],
        state: &mut AddHistoryState,
        ui: &mut egui::Ui,
    ) -> Vec<Event> {
        let mut events = Vec::new();
        let action = render_aircraft_dropdown(
            ui,
            "Aircraft:",
            "Select Aircraft",
            state.selected_aircraft.as_ref(),
            all_aircraft,
            &mut state.aircraft_search,
            &mut state.aircraft_display_count,
            state.aircraft_dropdown_open,
            &mut state.aircraft_search_autofocus,
            Event::ToggleAddHistoryAircraftDropdown,
        );

        match action {
            DropdownAction::Toggle => events.push(Event::ToggleAddHistoryAircraftDropdown),
            DropdownAction::Select(item) => {
                state.selected_aircraft = Some(item);
                events.push(Event::ToggleAddHistoryAircraftDropdown);
            }
            DropdownAction::Unselect => {
                state.selected_aircraft = None;
            }
            DropdownAction::None => {}
        }
        events
    }

    fn departure_selection(
        all_airports: &[Arc<Airport>],
        state: &mut AddHistoryState,
        ui: &mut egui::Ui,
    ) -> Vec<Event> {
        let mut events = Vec::new();
        let action = render_airport_dropdown(
            ui,
            "Departure:",
            "Select Departure",
            state.selected_departure.as_ref(),
            all_airports,
            &mut state.departure_search,
            &mut state.departure_display_count,
            state.departure_dropdown_open,
            &mut state.departure_search_autofocus,
            Event::ToggleAddHistoryDepartureDropdown,
        );

        match action {
            DropdownAction::Toggle => events.push(Event::ToggleAddHistoryDepartureDropdown),
            DropdownAction::Select(item) => {
                state.selected_departure = Some(item);
                events.push(Event::ToggleAddHistoryDepartureDropdown);
            }
            DropdownAction::Unselect => {
                state.selected_departure = None;
            }
            DropdownAction::None => {}
        }
        events
    }

    fn destination_selection(
        all_airports: &[Arc<Airport>],
        state: &mut AddHistoryState,
        ui: &mut egui::Ui,
    ) -> Vec<Event> {
        let mut events = Vec::new();
        let action = render_airport_dropdown(
            ui,
            "Destination:",
            "Select Destination",
            state.selected_destination.as_ref(),
            all_airports,
            &mut state.destination_search,
            &mut state.destination_display_count,
            state.destination_dropdown_open,
            &mut state.destination_search_autofocus,
            Event::ToggleAddHistoryDestinationDropdown,
        );

        match action {
            DropdownAction::Toggle => events.push(Event::ToggleAddHistoryDestinationDropdown),
            DropdownAction::Select(item) => {
                state.selected_destination = Some(item);
                events.push(Event::ToggleAddHistoryDestinationDropdown);
            }
            DropdownAction::Unselect => {
                state.selected_destination = None;
            }
            DropdownAction::None => {}
        }
        events
    }
}
