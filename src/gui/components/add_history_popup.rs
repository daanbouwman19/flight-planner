use crate::gui::events::Event;
use crate::models::{Aircraft, Airport};
use egui::{ComboBox, Ui};
use std::sync::Arc;

// --- View Model ---

/// View-model for the `AddHistoryPopup` component.
#[derive(Debug)]
pub struct AddHistoryPopupViewModel<'a> {
    pub all_aircraft: &'a [Arc<Aircraft>],
    pub all_airports: &'a [Arc<Airport>],
    pub selected_aircraft: &'a mut Option<Arc<Aircraft>>,
    pub selected_departure: &'a mut Option<Arc<Airport>>,
    pub selected_destination: &'a mut Option<Arc<Airport>>,
    pub aircraft_search: &'a mut String,
    pub departure_search: &'a mut String,
    pub destination_search: &'a mut String,
}

// --- Component ---

pub struct AddHistoryPopup;

impl AddHistoryPopup {
    pub fn render(vm: &mut AddHistoryPopupViewModel, ctx: &egui::Context) -> Vec<Event> {
        let mut events = Vec::new();
        let mut open = true;

        egui::Window::new("Add History Entry")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.vertical_centered_justified(|ui| {
                    Self::aircraft_selection(vm, ui);
                    Self::departure_selection(vm, ui);
                    Self::destination_selection(vm, ui);

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    let add_button_enabled = vm.selected_aircraft.is_some()
                        && vm.selected_departure.is_some()
                        && vm.selected_destination.is_some()
                        && vm.selected_departure != vm.selected_destination;

                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            events.push(Event::CloseAddHistoryPopup);
                        }

                        if ui
                            .add_enabled(add_button_enabled, egui::Button::new("Add"))
                            .clicked()
                        {
                            if let (Some(aircraft), Some(departure), Some(destination)) = (
                                vm.selected_aircraft.clone(),
                                vm.selected_departure.clone(),
                                vm.selected_destination.clone(),
                            ) {
                                events.push(Event::AddHistoryEntry {
                                    aircraft,
                                    departure,
                                    destination,
                                });
                            }
                        }
                    });
                });
            });

        if !open {
            events.push(Event::CloseAddHistoryPopup);
        }

        events
    }

    fn aircraft_selection(vm: &mut AddHistoryPopupViewModel, ui: &mut Ui) {
        ui.label("Aircraft:");
        let selected_text = vm
            .selected_aircraft
            .as_ref()
            .map_or("Select Aircraft", |a| &a.variant);

        let mut selection_made = false;
        ComboBox::from_id_salt("add_history_aircraft")
            .selected_text(selected_text)
            .show_ui(ui, |ui| {
                ui.text_edit_singleline(vm.aircraft_search);
                ui.separator();
                for aircraft in vm.all_aircraft.iter().filter(|a| {
                    a.variant
                        .to_lowercase()
                        .contains(&vm.aircraft_search.to_lowercase())
                }) {
                    if ui
                        .selectable_value(
                            vm.selected_aircraft,
                            Some(aircraft.clone()),
                            &aircraft.variant,
                        )
                        .clicked()
                    {
                        selection_made = true;
                    }
                }
            });

        if selection_made {
            vm.aircraft_search.clear();
        }
    }

    fn departure_selection(vm: &mut AddHistoryPopupViewModel, ui: &mut Ui) {
        ui.label("Departure:");
        let selected_text = vm
            .selected_departure
            .as_ref()
            .map_or("Select Departure", |a| &a.Name);

        let mut selection_made = false;
        ComboBox::from_id_salt("add_history_departure")
            .selected_text(selected_text)
            .show_ui(ui, |ui| {
                ui.text_edit_singleline(vm.departure_search);
                ui.separator();
                for airport in vm.all_airports.iter().filter(|a| {
                    a.Name
                        .to_lowercase()
                        .contains(&vm.departure_search.to_lowercase())
                }) {
                    if ui
                        .selectable_value(
                            vm.selected_departure,
                            Some(airport.clone()),
                            &airport.Name,
                        )
                        .clicked()
                    {
                        selection_made = true;
                    }
                }
            });

        if selection_made {
            vm.departure_search.clear();
        }
    }

    fn destination_selection(vm: &mut AddHistoryPopupViewModel, ui: &mut Ui) {
        ui.label("Destination:");
        let selected_text = vm
            .selected_destination
            .as_ref()
            .map_or("Select Destination", |a| &a.Name);

        let mut selection_made = false;
        ComboBox::from_id_salt("add_history_destination")
            .selected_text(selected_text)
            .show_ui(ui, |ui| {
                ui.text_edit_singleline(vm.destination_search);
                ui.separator();
                for airport in vm.all_airports.iter().filter(|a| {
                    a.Name
                        .to_lowercase()
                        .contains(&vm.destination_search.to_lowercase())
                }) {
                    if ui
                        .selectable_value(
                            vm.selected_destination,
                            Some(airport.clone()),
                            &airport.Name,
                        )
                        .clicked()
                    {
                        selection_made = true;
                    }
                }
            });

        if selection_made {
            vm.destination_search.clear();
        }
    }
}
