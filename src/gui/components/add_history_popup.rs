use crate::gui::events::Event;
use crate::gui::state::AddHistoryState;
use crate::models::{Aircraft, Airport};
use egui::{ScrollArea, TextEdit, Ui};
use std::sync::Arc;

/// The initial number of items to display in a searchable dropdown list.
pub const INITIAL_DISPLAY_COUNT: usize = 50;
/// The number of additional items to load when the user scrolls to the bottom of a dropdown list.
pub const LOAD_MORE_CHUNK_SIZE: usize = 50;

// --- Component ---

/// A UI component that renders a popup window for adding a new flight history entry.
///
/// This component provides a form with searchable dropdowns for selecting an
/// aircraft, departure airport, and destination airport.
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

            // Optimization: Only collect the items we need to display (plus one to check if there are more)
            // instead of allocating a Vec for ALL matching items (which could be 50,000+).
            let target_count = *args.display_count;
            let filtered_chunk: Vec<_> = args
                .all_items
                .iter()
                .filter(|item| {
                    crate::util::contains_case_insensitive(
                        (args.get_item_text)(item),
                        &search_lower,
                    )
                })
                .take(target_count + 1)
                .collect();

            let has_more_items = filtered_chunk.len() > target_count;
            let items_to_show = filtered_chunk.len().min(target_count);

            let scroll_area = ScrollArea::vertical()
                .max_height(200.0)
                .show(args.ui, |ui| {
                    for item in filtered_chunk.iter().take(items_to_show) {
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

            if has_more_items {
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

    fn aircraft_selection(
        all_aircraft: &[Arc<Aircraft>],
        state: &mut AddHistoryState,
        ui: &mut Ui,
    ) -> Vec<Event> {
        let mut args = DropdownArgs {
            ui,
            label: "Aircraft:",
            placeholder: "Select Aircraft",
            selected_item: &mut state.selected_aircraft,
            all_items: all_aircraft,
            search_text: &mut state.aircraft_search,
            display_count: &mut state.aircraft_display_count,
            dropdown_open: &mut state.aircraft_dropdown_open,
            search_autofocus: &mut state.aircraft_search_autofocus,
            get_item_text: Box::new(|a: &Aircraft| &a.variant),
            toggle_event: Event::ToggleAddHistoryAircraftDropdown,
            search_id: "add_history_aircraft_search",
        };
        Self::render_selection_dropdown(&mut args)
    }

    fn departure_selection(
        all_airports: &[Arc<Airport>],
        state: &mut AddHistoryState,
        ui: &mut Ui,
    ) -> Vec<Event> {
        let mut args = DropdownArgs {
            ui,
            label: "Departure:",
            placeholder: "Select Departure",
            selected_item: &mut state.selected_departure,
            all_items: all_airports,
            search_text: &mut state.departure_search,
            display_count: &mut state.departure_display_count,
            dropdown_open: &mut state.departure_dropdown_open,
            search_autofocus: &mut state.departure_search_autofocus,
            get_item_text: Box::new(|a: &Airport| &a.Name),
            toggle_event: Event::ToggleAddHistoryDepartureDropdown,
            search_id: "add_history_departure_search",
        };
        Self::render_selection_dropdown(&mut args)
    }

    fn destination_selection(
        all_airports: &[Arc<Airport>],
        state: &mut AddHistoryState,
        ui: &mut Ui,
    ) -> Vec<Event> {
        let mut args = DropdownArgs {
            ui,
            label: "Destination:",
            placeholder: "Select Destination",
            selected_item: &mut state.selected_destination,
            all_items: all_airports,
            search_text: &mut state.destination_search,
            display_count: &mut state.destination_display_count,
            dropdown_open: &mut state.destination_dropdown_open,
            search_autofocus: &mut state.destination_search_autofocus,
            get_item_text: Box::new(|a: &Airport| &a.Name),
            toggle_event: Event::ToggleAddHistoryDestinationDropdown,
            search_id: "add_history_destination_search",
        };
        Self::render_selection_dropdown(&mut args)
    }
}
