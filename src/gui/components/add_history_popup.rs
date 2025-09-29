use crate::gui::events::Event;
use crate::models::{Aircraft, Airport};
use egui::{ComboBox, ScrollArea, Ui};
use std::sync::Arc;

const INITIAL_DISPLAY_COUNT: usize = 50;
const LOAD_MORE_CHUNK_SIZE: usize = 50;

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
    pub aircraft_display_count: &'a mut usize,
    pub departure_display_count: &'a mut usize,
    pub destination_display_count: &'a mut usize,
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
                if ui.text_edit_singleline(vm.aircraft_search).changed() {
                    *vm.aircraft_display_count = INITIAL_DISPLAY_COUNT;
                }
                ui.separator();

                let filtered_aircraft: Vec<_> = vm
                    .all_aircraft
                    .iter()
                    .filter(|a| {
                        a.variant
                            .to_lowercase()
                            .contains(&vm.aircraft_search.to_lowercase())
                    })
                    .collect();

                let scroll_area = ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                    for aircraft in filtered_aircraft.iter().take(*vm.aircraft_display_count) {
                        if ui
                            .selectable_value(
                                &mut *vm.selected_aircraft,
                                Some((*aircraft).clone()),
                                &aircraft.variant,
                            )
                            .clicked()
                        {
                            selection_made = true;
                        }
                    }
                });

                if *vm.aircraft_display_count < filtered_aircraft.len() {
                    let scroll_state = scroll_area.state;
                    if scroll_state.offset.y + scroll_area.inner_rect.height()
                        >= scroll_area.content_size.y - 50.0
                    {
                        *vm.aircraft_display_count += LOAD_MORE_CHUNK_SIZE;
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
                if ui.text_edit_singleline(vm.departure_search).changed() {
                    *vm.departure_display_count = INITIAL_DISPLAY_COUNT;
                }
                ui.separator();

                let filtered_airports: Vec<_> = vm
                    .all_airports
                    .iter()
                    .filter(|a| {
                        a.Name
                            .to_lowercase()
                            .contains(&vm.departure_search.to_lowercase())
                    })
                    .collect();

                let scroll_area = ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                    for airport in filtered_airports.iter().take(*vm.departure_display_count) {
                        if ui
                            .selectable_value(
                                &mut *vm.selected_departure,
                                Some((*airport).clone()),
                                &airport.Name,
                            )
                            .clicked()
                        {
                            selection_made = true;
                        }
                    }
                });

                if *vm.departure_display_count < filtered_airports.len() {
                    let scroll_state = scroll_area.state;
                    if scroll_state.offset.y + scroll_area.inner_rect.height()
                        >= scroll_area.content_size.y - 50.0
                    {
                        *vm.departure_display_count += LOAD_MORE_CHUNK_SIZE;
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
                if ui.text_edit_singleline(vm.destination_search).changed() {
                    *vm.destination_display_count = INITIAL_DISPLAY_COUNT;
                }
                ui.separator();

                let filtered_airports: Vec<_> = vm
                    .all_airports
                    .iter()
                    .filter(|a| {
                        a.Name
                            .to_lowercase()
                            .contains(&vm.destination_search.to_lowercase())
                    })
                    .collect();

                let scroll_area = ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                    for airport in filtered_airports.iter().take(*vm.destination_display_count) {
                        if ui
                            .selectable_value(
                                &mut *vm.selected_destination,
                                Some((*airport).clone()),
                                &airport.Name,
                            )
                            .clicked()
                        {
                            selection_made = true;
                        }
                    }
                });

                if *vm.destination_display_count < filtered_airports.len() {
                    let scroll_state = scroll_area.state;
                    if scroll_state.offset.y + scroll_area.inner_rect.height()
                        >= scroll_area.content_size.y - 50.0
                    {
                        *vm.destination_display_count += LOAD_MORE_CHUNK_SIZE;
                    }
                }
            });

        if selection_made {
            vm.destination_search.clear();
        }
    }
}
