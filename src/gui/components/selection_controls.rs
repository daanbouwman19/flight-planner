use crate::gui::components::searchable_dropdown::{
    DropdownConfig, DropdownSelection, SearchableDropdown,
};
use crate::gui::events::Event;
use crate::models::{Aircraft, Airport};
use egui::Ui;
use rand::prelude::*;
use std::sync::Arc;

// --- View Model ---

/// View-model for the `SelectionControls` component.
pub struct SelectionControlsViewModel<'a> {
    pub selected_departure_airport: &'a Option<Arc<Airport>>,
    pub selected_aircraft: &'a Option<Arc<Aircraft>>,
    pub departure_dropdown_open: bool,
    pub aircraft_dropdown_open: bool,
    pub departure_airport_search: &'a mut String,
    pub aircraft_search: &'a mut String,
    pub departure_display_count: &'a mut usize,
    pub aircraft_display_count: &'a mut usize,
    pub available_airports: &'a [Arc<Airport>],
    pub all_aircraft: &'a [Arc<Aircraft>],
}

impl<'a> SelectionControlsViewModel<'a> {
    /// Creates a new view-model instance with a builder-like pattern.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        selected_departure_airport: &'a Option<Arc<Airport>>,
        selected_aircraft: &'a Option<Arc<Aircraft>>,
        departure_dropdown_open: bool,
        aircraft_dropdown_open: bool,
        departure_airport_search: &'a mut String,
        aircraft_search: &'a mut String,
        departure_display_count: &'a mut usize,
        aircraft_display_count: &'a mut usize,
        available_airports: &'a [Arc<Airport>],
        all_aircraft: &'a [Arc<Aircraft>],
    ) -> Self {
        Self {
            selected_departure_airport,
            selected_aircraft,
            departure_dropdown_open,
            aircraft_dropdown_open,
            departure_airport_search,
            aircraft_search,
            departure_display_count,
            aircraft_display_count,
            available_airports,
            all_aircraft,
        }
    }

    /// Helper method to check if a departure airport is selected.
    pub fn has_departure_airport(&self) -> bool {
        self.selected_departure_airport.is_some()
    }

    /// Helper method to check if an aircraft is selected.
    pub fn has_aircraft(&self) -> bool {
        self.selected_aircraft.is_some()
    }
}

// --- Component ---

pub struct SelectionControls;

impl SelectionControls {
    /// Renders the selection controls (departure airport and aircraft) in the original grouped layout.
    pub fn render(vm: &mut SelectionControlsViewModel, ui: &mut Ui) -> Vec<Event> {
        let mut events = Vec::new();

        // Group related route generation parameters together (original layout)
        ui.group(|ui| {
            ui.vertical(|ui| {
                ui.label("Route Generation Parameters");
                ui.separator();

                // Departure airport input
                events.extend(Self::render_departure_input(vm, ui));
                ui.add_space(5.0);

                // Aircraft selection
                events.extend(Self::render_aircraft_selection(vm, ui));
            });
        });

        events
    }

    /// Renders the departure airport input field with dropdown search functionality.
    fn render_departure_input(vm: &mut SelectionControlsViewModel, ui: &mut Ui) -> Vec<Event> {
        let mut events = Vec::new();

        ui.label("Departure airport:");

        // Button showing current selection
        let button_text = vm.selected_departure_airport.as_ref().map_or_else(
            || "🔀 No specific departure".to_string(),
            |airport| format!("{} ({})", airport.Name, airport.ICAO),
        );

        if ui.button(button_text).clicked() {
            events.push(Event::ToggleDepartureAirportDropdown);
        }

        // Only show dropdown if it's open
        if vm.departure_dropdown_open {
            ui.push_id("departure_dropdown", |ui| {
                let all_airports = vm.available_airports;
                let current_departure_airport = vm.selected_departure_airport.clone();

                let config = DropdownConfig {
                    random_option_text: "🎲 Pick random departure airport",
                    unspecified_option_text: "🔀 No specific departure (system picks randomly)",
                    is_unspecified_selected: current_departure_airport.is_none(),
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
                        all_airports,
                        vm.departure_airport_search,
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
                        vm.departure_display_count,
                    );
                    dropdown.render(ui)
                };

                // Handle selection
                match selection {
                    DropdownSelection::Item(airport) | DropdownSelection::Random(airport) => {
                        events.extend(Self::handle_departure_airport_selection(Some(airport)));
                    }
                    DropdownSelection::Unspecified => {
                        events.extend(Self::handle_departure_airport_selection(None));
                    }
                    DropdownSelection::None => {}
                }
            });
        }
        events
    }

    /// Renders the aircraft selection section.
    fn render_aircraft_selection(vm: &mut SelectionControlsViewModel, ui: &mut Ui) -> Vec<Event> {
        let mut events = Vec::new();
        ui.label("Aircraft:");

        let button_text = vm.selected_aircraft.as_ref().map_or_else(
            || "🔀 No specific aircraft".to_string(),
            |aircraft| format!("{} {}", aircraft.manufacturer, aircraft.variant),
        );

        if ui.button(button_text).clicked() {
            events.push(Event::ToggleAircraftDropdown);
        }

        if vm.aircraft_dropdown_open {
            ui.push_id("aircraft_dropdown", |ui| {
                let all_aircraft = vm.all_aircraft;
                let selected_aircraft = vm.selected_aircraft.clone();

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
                        all_aircraft,
                        vm.aircraft_search,
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
                        vm.aircraft_display_count,
                    );
                    dropdown.render(ui)
                };

                match selection {
                    DropdownSelection::Item(aircraft) | DropdownSelection::Random(aircraft) => {
                        events.extend(Self::handle_aircraft_selection(Some(aircraft)));
                    }
                    DropdownSelection::Unspecified => {
                        events.extend(Self::handle_aircraft_selection(None));
                    }
                    DropdownSelection::None => {}
                }
            });
        }
        events
    }

    /// Handles departure airport selection
    fn handle_departure_airport_selection(airport: Option<Arc<Airport>>) -> Vec<Event> {
        vec![
            Event::DepartureAirportSelected(airport),
            Event::RegenerateRoutesForSelectionChange,
        ]
    }

    /// Handles aircraft selection
    fn handle_aircraft_selection(aircraft: Option<Arc<Aircraft>>) -> Vec<Event> {
        vec![
            Event::AircraftSelected(aircraft),
            Event::RegenerateRoutesForSelectionChange,
        ]
    }
}
