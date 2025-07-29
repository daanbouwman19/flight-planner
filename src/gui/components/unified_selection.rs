use egui::{Color32, Ui};
use rand::prelude::*;
use std::sync::Arc;

use crate::{
    gui::components::searchable_dropdown::{DropdownConfig, DropdownSelection, SearchableDropdown},
    gui::ui::Gui,
    models::Aircraft,
};

/// Enum to define the type of selection being rendered
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SelectionType {
    DepartureAirport,
    Aircraft,
}

/// Unified selection component that handles both departure airport and aircraft selection
pub struct UnifiedSelection;

impl UnifiedSelection {
    /// Renders a unified selection component that can handle both departure airport and aircraft
    ///
    /// # Arguments
    ///
    /// * `gui` - The GUI state
    /// * `ui` - The UI context
    /// * `selection_type` - The type of selection to render
    pub fn render(gui: &mut Gui, ui: &mut Ui, selection_type: SelectionType) {
        match selection_type {
            SelectionType::DepartureAirport => Self::render_departure_section(gui, ui),
            SelectionType::Aircraft => Self::render_aircraft_section(gui, ui),
        }
    }

    /// Renders the departure airport selection section
    fn render_departure_section(gui: &mut Gui, ui: &mut Ui) {
        ui.label("Departure airport:");

        ui.horizontal(|ui| {
            let button_text = if gui.get_departure_airport_icao().is_empty() {
                "🔀 No specific departure".to_string()
            } else {
                // Try to find the airport name to show in the button
                let airport_name = gui
                    .get_available_airports()
                    .iter()
                    .find(|airport| airport.ICAO == gui.get_departure_airport_icao())
                    .map_or_else(
                        || gui.get_departure_airport_icao().to_string(),
                        |airport| format!("{} ({})", airport.Name, airport.ICAO),
                    );
                airport_name
            };

            let button_response = ui
                .button(button_text)
                .on_hover_text("Click to select departure airport");

            if button_response.clicked() {
                let is_open = gui.is_departure_airport_dropdown_open();
                gui.set_departure_airport_dropdown_open(!is_open);
            }
        });

        // Show the dropdown below the buttons if open
        if gui.is_departure_airport_dropdown_open() {
            Self::render_departure_dropdown(gui, ui);
        }

        // Show validation feedback if applicable
        Self::render_departure_validation_feedback(gui, ui);
    }

    /// Renders the aircraft selection section
    fn render_aircraft_section(gui: &mut Gui, ui: &mut Ui) {
        ui.label("Aircraft:");

        ui.horizontal(|ui| {
            let button_text = gui.get_selected_aircraft().map_or_else(
                || "🔀 No specific aircraft".to_string(),
                |aircraft| format!("{} {}", aircraft.manufacturer, aircraft.variant),
            );

            let button_response = ui
                .button(button_text)
                .on_hover_text("Click to select aircraft");

            if button_response.clicked() {
                let is_open = gui.is_aircraft_dropdown_open();
                gui.set_aircraft_dropdown_open(!is_open);
            }
        });

        // Show the dropdown below the buttons if open
        if gui.is_aircraft_dropdown_open() {
            Self::render_aircraft_dropdown(gui, ui);
        }
    }

    /// Renders the departure airport dropdown
    fn render_departure_dropdown(gui: &mut Gui, ui: &mut Ui) {
        ui.push_id("departure_airport_dropdown", |ui| {
            let all_airports = gui.get_available_airports().to_vec();
            let mut current_search = gui.get_departure_airport_search().to_string();
            let current_departure_icao = gui.get_departure_airport_icao().to_string();
            let mut display_count = *gui.get_departure_airport_dropdown_display_count_mut();

            let config = DropdownConfig {
                random_option_text: "🎲 Pick random departure airport",
                unspecified_option_text: "🔀 No specific departure (system picks randomly)",
                is_unspecified_selected: current_departure_icao.is_empty(),
                search_hint: "Search by name or ICAO (e.g. 'London' or 'EGLL')",
                empty_search_help: &[
                    "💡 Browse airports or search by name/ICAO code",
                    "   Examples: 'London', 'EGLL', 'JFK', 'Amsterdam'",
                ],
                show_items_when_empty: true,
                initial_chunk_size: 50,
                min_search_length: 2,
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
                    Box::new(move |airport| airport.ICAO == current_departure_icao),
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
                    Self::handle_departure_airport_selection(gui, &airport.ICAO);
                }
                DropdownSelection::Unspecified => {
                    Self::handle_departure_airport_selection(gui, "");
                }
                DropdownSelection::None => {
                    // No action needed
                }
            }
        });
    }

    /// Renders the aircraft dropdown
    fn render_aircraft_dropdown(gui: &mut Gui, ui: &mut Ui) {
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
                    Box::new(|aircraft| format!("{} {}", aircraft.manufacturer, aircraft.variant)),
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

    /// Handles departure airport selection
    fn handle_departure_airport_selection(gui: &mut Gui, icao: &str) {
        *gui.get_departure_airport_icao_mut() = icao.to_string();
        gui.set_departure_airport_search(String::new());
        gui.set_departure_airport_dropdown_open(false);
        gui.update_departure_validation_state();
        gui.regenerate_routes_for_departure_change();
    }

    /// Handles aircraft selection
    fn handle_aircraft_selection(gui: &mut Gui, aircraft: &Arc<Aircraft>) {
        gui.set_selected_aircraft(Some(Arc::clone(aircraft)));
        gui.set_aircraft_search(String::new());
        gui.set_aircraft_dropdown_open(false);
        gui.generate_routes_for_selected_aircraft(aircraft);
    }

    /// Handles no aircraft selection
    fn handle_no_aircraft_selection(gui: &mut Gui) {
        gui.set_selected_aircraft(None);
        gui.set_aircraft_search(String::new());
        gui.set_aircraft_dropdown_open(false);
    }

    /// Renders validation feedback for departure airport
    fn render_departure_validation_feedback(gui: &Gui, ui: &mut Ui) {
        let icao = gui.get_departure_airport_icao();
        if !icao.is_empty() {
            if let Some(is_valid) = gui.get_departure_airport_validation() {
                if is_valid {
                    ui.colored_label(Color32::GREEN, "✓ Valid airport");
                } else {
                    ui.colored_label(Color32::RED, "✗ Airport not found");
                }
            }
        }
    }
}
