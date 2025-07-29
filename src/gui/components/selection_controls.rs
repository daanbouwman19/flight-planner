use crate::gui::components::searchable_dropdown::{
    DropdownConfig, DropdownSelection, SearchableDropdown,
};
use crate::gui::ui::Gui;
use crate::models::{Aircraft, Airport};
use egui::Ui;
use rand::prelude::*;
use std::sync::Arc;

pub struct SelectionControls;

impl SelectionControls {
    /// Renders the selection controls (departure airport and aircraft) in the original grouped layout.
    pub fn render(gui: &mut Gui, ui: &mut Ui) {
        // Group related route generation parameters together (original layout)
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label("Route Generation Parameters");
                ui.separator();

                // Departure airport input
                Self::render_departure_input(gui, ui);
                ui.add_space(5.0);

                // Aircraft selection
                Self::render_aircraft_selection(gui, ui);
            });
        });
    }

    /// Renders the departure airport input field with dropdown search functionality.
    fn render_departure_input(gui: &mut Gui, ui: &mut Ui) {
        ui.label("Departure airport:");

        // Button showing current selection
        let button_text = gui.get_departure_airport().map_or_else(
            || "🔀 No specific departure".to_string(),
            |airport| format!("{} ({})", airport.Name, airport.ICAO),
        );

        if ui.button(button_text).clicked() {
            let is_open = gui.is_departure_airport_dropdown_open();
            gui.set_departure_airport_dropdown_open(!is_open);
            gui.set_aircraft_dropdown_open(false); // Close other dropdown
        }

        // Only show dropdown if it's open
        if gui.is_departure_airport_dropdown_open() {
            ui.push_id("departure_dropdown", |ui| {
                let all_airports = gui.get_available_airports().to_vec();
                let mut current_search = gui.get_departure_airport_search().to_string();
                let current_departure_airport = gui.get_departure_airport().cloned();
                let mut display_count = *gui.get_departure_airport_dropdown_display_count_mut();

                let config = DropdownConfig {
                    random_option_text: "🎲 Pick random departure airport",
                    unspecified_option_text: "🔀 No specific departure (system picks randomly)",
                    is_unspecified_selected: current_departure_airport.is_none(),
                    search_hint: "Search by name or ICAO (e.g. 'London' or 'EGLL')",
                    empty_search_help: &[
                        "� Browse airports or search by name/ICAO code",
                        "   Examples: 'London', 'EGLL', 'JFK', 'Amsterdam'",
                    ],
                    show_items_when_empty: true,
                    initial_chunk_size: 50,
                    min_search_length: 2,
                    // Limit airport results due to large dataset (thousands of airports)
                    max_results: 50,
                    no_results_text: "🔍 No airports found",
                    no_results_help: &["   Try different search terms"],
                    min_width: 400.0,
                    max_height: 300.0,
                };

                let selection = {
                    let mut dropdown = SearchableDropdown::new(
                        &all_airports,
                        &mut current_search,
                        Box::new(move |airport| {
                            current_departure_airport
                                .as_ref()
                                .is_some_and(|selected| Arc::ptr_eq(selected, airport))
                        }),
                        Box::new(|airport| format!("{} ({})", airport.Name, airport.ICAO)),
                        Box::new(|airport, search_lower| {
                            let name_matches = airport.Name.to_lowercase().contains(search_lower);
                            let icao_matches = airport.ICAO.to_lowercase().contains(search_lower);
                            name_matches || icao_matches
                        }),
                        Box::new(|airports| airports.choose(&mut rand::rng()).cloned()),
                        config,
                        &mut display_count,
                    );

                    dropdown.render(ui)
                };

                // Update search text and display count
                gui.set_departure_airport_search(current_search);
                *gui.get_departure_airport_dropdown_display_count_mut() = display_count;

                // Handle selection
                match selection {
                    DropdownSelection::Item(airport) | DropdownSelection::Random(airport) => {
                        Self::handle_departure_airport_selection(gui, Some(airport));
                    }
                    DropdownSelection::Unspecified => {
                        Self::handle_departure_airport_selection(gui, None);
                    }
                    DropdownSelection::None => {
                        // No action needed
                    }
                }
            });
        }
    }

    /// Renders the aircraft selection section.
    fn render_aircraft_selection(gui: &mut Gui, ui: &mut Ui) {
        ui.label("Aircraft:");

        // Button showing current selection
        let button_text = gui.get_selected_aircraft().map_or_else(
            || "🔀 No specific aircraft".to_string(),
            |aircraft| format!("{} {}", aircraft.manufacturer, aircraft.variant),
        );

        if ui.button(button_text).clicked() {
            let is_open = gui.is_aircraft_dropdown_open();
            gui.set_aircraft_dropdown_open(!is_open);
            gui.set_departure_airport_dropdown_open(false); // Close other dropdown
        }

        // Only show dropdown if it's open
        if gui.is_aircraft_dropdown_open() {
            ui.push_id("aircraft_dropdown", |ui| {
                let all_aircraft = gui.get_all_aircraft().to_vec();
                let mut current_search = gui.get_aircraft_search().to_string();
                let selected_aircraft = gui.get_selected_aircraft().map(Arc::clone);
                let mut display_count = *gui.get_aircraft_dropdown_display_count_mut();

                let config = DropdownConfig {
                    random_option_text: "🎲 Pick random aircraft",
                    unspecified_option_text: "🔀 No specific aircraft (system picks randomly)",
                    is_unspecified_selected: selected_aircraft.is_none(),
                    search_hint: "Search by manufacturer or variant (e.g. 'Boeing' or '737')",
                    empty_search_help: &[
                        "💡 Browse aircraft or search by manufacturer/variant",
                        "   Examples: 'Boeing', '737', 'Airbus', 'A320'",
                    ],
                    show_items_when_empty: true,
                    initial_chunk_size: 100,
                    min_search_length: 0,
                    // Unlimited results for aircraft (smaller dataset, typically hundreds vs thousands)
                    max_results: 0,
                    no_results_text: "🔍 No aircraft found",
                    no_results_help: &["   Try different search terms"],
                    min_width: 300.0,
                    max_height: 300.0,
                };

                let selection = {
                    let mut dropdown = SearchableDropdown::new(
                        &all_aircraft,
                        &mut current_search,
                        Box::new(move |aircraft| {
                            selected_aircraft
                                .as_ref()
                                .is_some_and(|selected| Arc::ptr_eq(selected, aircraft))
                        }),
                        Box::new(|aircraft| {
                            format!("{} {}", aircraft.manufacturer, aircraft.variant)
                        }),
                        Box::new(|aircraft, search_lower| {
                            let display_text =
                                format!("{} {}", aircraft.manufacturer, aircraft.variant);
                            display_text.to_lowercase().contains(search_lower)
                        }),
                        Box::new(|aircraft_list| {
                            aircraft_list.choose(&mut rand::rng()).map(Arc::clone)
                        }),
                        config,
                        &mut display_count,
                    );

                    dropdown.render(ui)
                };

                // Update search text and display count
                gui.set_aircraft_search(current_search);
                *gui.get_aircraft_dropdown_display_count_mut() = display_count;

                // Handle selection
                match selection {
                    DropdownSelection::Item(aircraft) | DropdownSelection::Random(aircraft) => {
                        Self::handle_aircraft_selection(gui, &aircraft);
                    }
                    DropdownSelection::Unspecified => {
                        Self::handle_no_aircraft_selection(gui);
                    }
                    DropdownSelection::None => {
                        // No action needed
                    }
                }
            });
        }
    }

    /// Handles departure airport selection
    fn handle_departure_airport_selection(gui: &mut Gui, airport: Option<Arc<Airport>>) {
        gui.set_departure_airport(airport);
        gui.close_departure_dropdown();
        gui.regenerate_routes_for_selection_change();
    }

    /// Handles aircraft selection
    fn handle_aircraft_selection(gui: &mut Gui, aircraft: &Arc<Aircraft>) {
        gui.set_selected_aircraft(Some(Arc::clone(aircraft)));
        gui.close_aircraft_dropdown();
        gui.regenerate_routes_for_selection_change();
    }

    /// Handles no aircraft selection
    fn handle_no_aircraft_selection(gui: &mut Gui) {
        gui.set_selected_aircraft(None);
        gui.close_aircraft_dropdown();
        gui.regenerate_routes_for_selection_change();
    }
}
