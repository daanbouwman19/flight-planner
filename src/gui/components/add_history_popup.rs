use crate::gui::components::dropdowns::{
    DropdownAction, DropdownParams, render_aircraft_dropdown, render_airport_dropdown,
};
use crate::gui::events::{AppEvent, DataEvent, SelectionEvent, UiEvent};
use crate::gui::state::AddHistoryState;
use crate::models::{Aircraft, Airport};
use std::sync::Arc;

// --- Component ---

/// A UI component that renders a popup window for adding a new flight history entry.
///
/// This component provides a form with searchable dropdowns for selecting an
/// aircraft, departure airport, and destination airport.
pub struct AddHistoryPopup;

#[cfg(not(tarpaulin_include))]
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
    #[cfg(not(tarpaulin_include))]
    pub fn render(
        all_aircraft: &[Arc<Aircraft>],
        all_airports: &[Arc<Airport>],
        state: &mut AddHistoryState,
        ctx: &egui::Context,
    ) -> Vec<AppEvent> {
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

                    ui.add_space(5.0);
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                        ui.label(egui::RichText::new("* Required").small().weak());
                    });
                    ui.add_space(5.0);

                    ui.separator();
                    ui.add_space(10.0);

                    let mut blocking_issues = Vec::new();
                    let mut display_errors = Vec::new();

                    if state.selected_aircraft.is_none() {
                        blocking_issues.push("Select an aircraft".to_string());
                    }
                    if state.selected_departure.is_none() {
                        blocking_issues.push("Select a departure airport".to_string());
                    }
                    if state.selected_destination.is_none() {
                        blocking_issues.push("Select a destination airport".to_string());
                    } else if state.selected_departure == state.selected_destination {
                        let msg = "Departure and destination must be different".to_string();
                        blocking_issues.push(msg.clone());
                        display_errors.push(msg);
                    }

                    if let (Some(aircraft), Some(dep), Some(dest)) = (
                        &state.selected_aircraft,
                        &state.selected_departure,
                        &state.selected_destination,
                    ) {
                        let dist = crate::util::calculate_haversine_distance_nm(dep, dest);
                        if dist > aircraft.aircraft_range {
                            let msg = format!(
                                "Route distance ({} nm) exceeds aircraft range ({} nm)",
                                dist, aircraft.aircraft_range
                            );
                            blocking_issues.push(msg.clone());
                            display_errors.push(msg);
                        }
                    }

                    let add_button_enabled = blocking_issues.is_empty();

                    let tooltip = if add_button_enabled {
                        "Add this flight to your history".to_string()
                    } else {
                        format!("Cannot add flight:\n• {}", blocking_issues.join("\n• "))
                    };

                    // Inline validation feedback (only for logic errors, not just missing fields)
                    if !display_errors.is_empty() {
                        ui.group(|ui| {
                            ui.set_width(ui.available_width());
                            ui.label(
                                egui::RichText::new("⚠️ Cannot add flight:")
                                    .color(ui.visuals().error_fg_color)
                                    .strong(),
                            );
                            for error in &display_errors {
                                ui.label(
                                    egui::RichText::new(format!("• {}", error))
                                        .color(ui.visuals().error_fg_color)
                                        .small(),
                                );
                            }
                        });
                        ui.add_space(5.0);
                    }

                    ui.horizontal(|ui| {
                        if ui
                            .button("❌ Cancel")
                            .on_hover_text("Discard entry and close (Esc)")
                            .clicked()
                            || ui.input(|i| i.key_pressed(egui::Key::Escape))
                        {
                            events.push(AppEvent::Ui(UiEvent::CloseAddHistoryPopup));
                        }

                        if ui
                            .add_enabled(add_button_enabled, egui::Button::new("➕ Add"))
                            .on_hover_text(format!("{} (Ctrl+Enter)", tooltip))
                            .on_disabled_hover_text(&tooltip)
                            .clicked()
                            || (add_button_enabled
                                && ui.input(|i| {
                                    i.modifiers.command && i.key_pressed(egui::Key::Enter)
                                }))
                        {
                            // The `add_button_enabled` check guarantees these are `Some`.
                            let aircraft = state.selected_aircraft.clone().unwrap();
                            let departure = state.selected_departure.clone().unwrap();
                            let destination = state.selected_destination.clone().unwrap();
                            events.push(AppEvent::Data(DataEvent::AddHistoryEntry {
                                aircraft,
                                departure,
                                destination,
                            }));
                        }
                    });
                });
            });

        if !open {
            events.push(AppEvent::Ui(UiEvent::CloseAddHistoryPopup));
        }

        events
    }

    fn aircraft_selection(
        all_aircraft: &[Arc<Aircraft>],
        state: &mut AddHistoryState,
        ui: &mut egui::Ui,
    ) -> Vec<AppEvent> {
        let mut events = Vec::new();
        let params = DropdownParams {
            ui,
            id: "add_history_aircraft",
            label: "Aircraft: *",
            placeholder: "Select Aircraft",
            selected_item: state.selected_aircraft.as_ref(),
            all_items: all_aircraft,
            search_text: &mut state.aircraft_search,
            display_count: &mut state.aircraft_display_count,
            is_open: state.aircraft_dropdown_open,
            autofocus: &mut state.aircraft_search_autofocus,
        };

        match render_aircraft_dropdown(params) {
            DropdownAction::Toggle => {
                events.push(AppEvent::Selection(
                    SelectionEvent::ToggleAddHistoryAircraftDropdown,
                ));
            }
            DropdownAction::Select(item) => {
                state.selected_aircraft = Some(item);
                events.push(AppEvent::Selection(
                    SelectionEvent::ToggleAddHistoryAircraftDropdown,
                ));
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
    ) -> Vec<AppEvent> {
        let mut events = Vec::new();
        let params = DropdownParams {
            ui,
            id: "add_history_departure",
            label: "Departure: *",
            placeholder: "Select Departure",
            selected_item: state.selected_departure.as_ref(),
            all_items: all_airports,
            search_text: &mut state.departure_search,
            display_count: &mut state.departure_display_count,
            is_open: state.departure_dropdown_open,
            autofocus: &mut state.departure_search_autofocus,
        };

        match render_airport_dropdown(params) {
            DropdownAction::Toggle => {
                events.push(AppEvent::Selection(
                    SelectionEvent::ToggleAddHistoryDepartureDropdown,
                ));
            }
            DropdownAction::Select(item) => {
                state.selected_departure = Some(item);
                events.push(AppEvent::Selection(
                    SelectionEvent::ToggleAddHistoryDepartureDropdown,
                ));
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
    ) -> Vec<AppEvent> {
        let mut events = Vec::new();
        let params = DropdownParams {
            ui,
            id: "add_history_destination",
            label: "Destination: *",
            placeholder: "Select Destination",
            selected_item: state.selected_destination.as_ref(),
            all_items: all_airports,
            search_text: &mut state.destination_search,
            display_count: &mut state.destination_display_count,
            is_open: state.destination_dropdown_open,
            autofocus: &mut state.destination_search_autofocus,
        };

        match render_airport_dropdown(params) {
            DropdownAction::Toggle => {
                events.push(AppEvent::Selection(
                    SelectionEvent::ToggleAddHistoryDestinationDropdown,
                ));
            }
            DropdownAction::Select(item) => {
                state.selected_destination = Some(item);
                events.push(AppEvent::Selection(
                    SelectionEvent::ToggleAddHistoryDestinationDropdown,
                ));
            }
            DropdownAction::Unselect => {
                state.selected_destination = None;
            }
            DropdownAction::None => {}
        }
        events
    }
}
