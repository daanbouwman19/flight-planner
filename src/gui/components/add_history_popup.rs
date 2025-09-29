use crate::gui::events::Event;
use crate::models::{Aircraft, Airport};
use egui::{ScrollArea, TextEdit, Ui};
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
    pub aircraft_dropdown_open: &'a mut bool,
    pub departure_dropdown_open: &'a mut bool,
    pub destination_dropdown_open: &'a mut bool,
    pub aircraft_search_autofocus: &'a mut bool,
    pub departure_search_autofocus: &'a mut bool,
    pub destination_search_autofocus: &'a mut bool,
}

// --- Component ---

pub struct AddHistoryPopup;

struct DropdownArgs<'a, T> {
    ui: &'a mut Ui,
    label: &'a str,
    placeholder: &'a str,
    selected_item: &'a mut Option<Arc<T>>,
    all_items: &'a [Arc<T>],
    search_text: &'a mut String,
    display_count: &'a mut usize,
    dropdown_open: &'a mut bool,
    search_autofocus: &'a mut bool,
    get_item_text: Box<dyn Fn(&T) -> &str + 'a>,
    toggle_event: Event,
    search_id: &'a str,
}

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
                    events.extend(Self::aircraft_selection(vm, ui));
                    events.extend(Self::departure_selection(vm, ui));
                    events.extend(Self::destination_selection(vm, ui));

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
                            // The `add_button_enabled` check guarantees these are `Some`.
                            let aircraft = vm.selected_aircraft.clone().unwrap();
                            let departure = vm.selected_departure.clone().unwrap();
                            let destination = vm.selected_destination.clone().unwrap();
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

    /// Renders a generic searchable selection dropdown.
    fn render_selection_dropdown<T: PartialEq + Clone + 'static>(
        args: &mut DropdownArgs<T>,
    ) -> Vec<Event> {
        let mut events = Vec::new();
        args.ui.label(args.label);
        let selected_text = args
            .selected_item
            .as_ref()
            .map_or(args.placeholder, |item| (args.get_item_text)(item));

        if args.ui.button(selected_text).clicked() {
            events.push(args.toggle_event.clone());
        }

        if *args.dropdown_open {
            let search_bar = TextEdit::singleline(args.search_text)
                .hint_text("Search...")
                .id(egui::Id::new(args.search_id));
            let search_response = args.ui.add(search_bar);

            if *args.search_autofocus {
                search_response.request_focus();
                *args.search_autofocus = false;
            }

            if search_response.changed() {
                *args.display_count = INITIAL_DISPLAY_COUNT;
            }
            args.ui.separator();

            // Performance: convert search text to lowercase once before the loop.
            let search_lower = args.search_text.to_lowercase();
            let filtered_items: Vec<_> = args
                .all_items
                .iter()
                .filter(|item| {
                    (args.get_item_text)(item)
                        .to_lowercase()
                        .contains(&search_lower)
                })
                .collect();

            let scroll_area = ScrollArea::vertical()
                .max_height(200.0)
                .show(args.ui, |ui| {
                    for item in filtered_items.iter().take(*args.display_count) {
                        if ui
                            .selectable_value(
                                args.selected_item,
                                Some(Arc::clone(item)),
                                (args.get_item_text)(item),
                            )
                            .clicked()
                        {
                            events.push(args.toggle_event.clone());
                        }
                    }
                });

            if *args.display_count < filtered_items.len() {
                let scroll_state = scroll_area.state;
                if scroll_state.offset.y + scroll_area.inner_rect.height()
                    >= scroll_area.content_size.y - 50.0
                {
                    *args.display_count += LOAD_MORE_CHUNK_SIZE;
                }
            }
        }
        events
    }

    fn aircraft_selection(vm: &mut AddHistoryPopupViewModel, ui: &mut Ui) -> Vec<Event> {
        let mut args = DropdownArgs {
            ui,
            label: "Aircraft:",
            placeholder: "Select Aircraft",
            selected_item: vm.selected_aircraft,
            all_items: vm.all_aircraft,
            search_text: vm.aircraft_search,
            display_count: vm.aircraft_display_count,
            dropdown_open: vm.aircraft_dropdown_open,
            search_autofocus: vm.aircraft_search_autofocus,
            get_item_text: Box::new(|a: &Aircraft| &a.variant),
            toggle_event: Event::ToggleAddHistoryAircraftDropdown,
            search_id: "add_history_aircraft_search",
        };
        Self::render_selection_dropdown(&mut args)
    }

    fn departure_selection(vm: &mut AddHistoryPopupViewModel, ui: &mut Ui) -> Vec<Event> {
        let mut args = DropdownArgs {
            ui,
            label: "Departure:",
            placeholder: "Select Departure",
            selected_item: vm.selected_departure,
            all_items: vm.all_airports,
            search_text: vm.departure_search,
            display_count: vm.departure_display_count,
            dropdown_open: vm.departure_dropdown_open,
            search_autofocus: vm.departure_search_autofocus,
            get_item_text: Box::new(|a: &Airport| &a.Name),
            toggle_event: Event::ToggleAddHistoryDepartureDropdown,
            search_id: "add_history_departure_search",
        };
        Self::render_selection_dropdown(&mut args)
    }

    fn destination_selection(vm: &mut AddHistoryPopupViewModel, ui: &mut Ui) -> Vec<Event> {
        let mut args = DropdownArgs {
            ui,
            label: "Destination:",
            placeholder: "Select Destination",
            selected_item: vm.selected_destination,
            all_items: vm.all_airports,
            search_text: vm.destination_search,
            display_count: vm.destination_display_count,
            dropdown_open: vm.destination_dropdown_open,
            search_autofocus: vm.destination_search_autofocus,
            get_item_text: Box::new(|a: &Airport| &a.Name),
            toggle_event: Event::ToggleAddHistoryDestinationDropdown,
            search_id: "add_history_destination_search",
        };
        Self::render_selection_dropdown(&mut args)
    }
}
