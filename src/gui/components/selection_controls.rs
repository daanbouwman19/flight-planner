use crate::gui::components::searchable_dropdown::{
    DropdownConfig, DropdownSelection, SearchableDropdown,
};
use crate::gui::events::Event;
use crate::models::{Aircraft, Airport};
use egui::Ui;
use rand::prelude::*;
use std::sync::Arc;

/// A view model that provides the necessary data and state for the `SelectionControls` component.
///
/// This struct aggregates all the state required to render the departure airport
/// and aircraft selection dropdowns, including current selections, search text,
/// and the full lists of available options.
pub struct SelectionControlsViewModel<'a> {
    /// The currently selected departure airport.
    pub selected_departure_airport: &'a Option<Arc<Airport>>,
    /// The currently selected aircraft.
    pub selected_aircraft: &'a Option<Arc<Aircraft>>,
    /// A mutable flag to control the visibility of the departure dropdown.
    pub departure_dropdown_open: &'a mut bool,
    /// A mutable flag to control the visibility of the aircraft dropdown.
    pub aircraft_dropdown_open: &'a mut bool,
    /// A mutable reference to the search text for the departure airport dropdown.
    pub departure_airport_search: &'a mut String,
    /// A mutable reference to the search text for the aircraft dropdown.
    pub aircraft_search: &'a mut String,
    /// A mutable reference to the number of departure airports to display (for lazy loading).
    pub departure_display_count: &'a mut usize,
    /// A mutable reference to the number of aircraft to display (for lazy loading).
    pub aircraft_display_count: &'a mut usize,
    /// A slice of all available airports to populate the dropdown.
    pub available_airports: &'a [Arc<Airport>],
    /// A slice of all available aircraft to populate the dropdown.
    pub all_aircraft: &'a [Arc<Aircraft>],
}

/// A UI component that renders the primary selection controls for the application.
///
/// This component includes searchable dropdowns for selecting a departure airport
/// and an aircraft, which are key inputs for route generation.
pub struct SelectionControls;

impl SelectionControls {
    /// Renders the selection controls for departure airport and aircraft.
    ///
    /// This method uses the `SearchableDropdown` component to create consistent
    /// and feature-rich selection widgets.
    ///
    /// # Arguments
    ///
    /// * `vm` - A mutable reference to the `SelectionControlsViewModel`.
    /// * `ui` - A mutable reference to the `egui::Ui` context for rendering.
    ///
    /// # Returns
    ///
    /// A `Vec<Event>` containing any events triggered by user selections.
    pub fn render(vm: &mut SelectionControlsViewModel, ui: &mut Ui) -> Vec<Event> {
        let mut events = Vec::new();

        ui.add_space(10.0);
        ui.label("Selections");
        ui.separator();

        let departure_display_text = if let Some(airport) = vm.selected_departure_airport {
            format!("{} ({})", airport.Name, airport.ICAO)
        } else {
            "Select departure airport".to_string()
        };

        ui.label("Departure Airport:");
        if ui.button(&departure_display_text).clicked() {
            events.push(Event::ToggleDepartureAirportDropdown);
        }

        if *vm.departure_dropdown_open {
            let config = DropdownConfig {
                id: "departure_airport_dropdown",
                search_hint: "Search for an airport",
                initial_chunk_size: 50,
                auto_focus: true,
                ..Default::default()
            };

            let mut departure_dropdown = SearchableDropdown::new(
                vm.available_airports,
                vm.departure_airport_search,
                Box::new(|airport| {
                    vm.selected_departure_airport
                        .as_ref()
                        .is_some_and(|a| a.ID == airport.ID)
                }),
                Box::new(|airport: &Arc<Airport>| format!("{} ({})", airport.Name, airport.ICAO)),
                Box::new(|airport, search| {
                    airport.Name.to_lowercase().contains(search)
                        || airport.ICAO.to_lowercase().contains(search)
                }),
                Box::new(|items| items.choose(&mut rand::rng()).cloned()),
                config,
                vm.departure_display_count,
            );

            match departure_dropdown.render(ui) {
                DropdownSelection::Item(airport) => {
                    events.push(Event::DepartureAirportSelected(Some(airport)));
                    events.push(Event::RegenerateRoutesForSelectionChange);
                }
                DropdownSelection::Random(airport) => {
                    events.push(Event::DepartureAirportSelected(Some(airport)));
                    events.push(Event::RegenerateRoutesForSelectionChange);
                }
                DropdownSelection::Unspecified => {
                    events.push(Event::DepartureAirportSelected(None));
                    events.push(Event::RegenerateRoutesForSelectionChange);
                }
                DropdownSelection::None => {}
            }
        }

        let aircraft_display_text = if let Some(aircraft) = vm.selected_aircraft {
            format!("{} {}", aircraft.manufacturer, aircraft.variant)
        } else {
            "Select aircraft".to_string()
        };

        ui.label("Aircraft:");
        if ui.button(&aircraft_display_text).clicked() {
            events.push(Event::ToggleAircraftDropdown);
        }

        if *vm.aircraft_dropdown_open {
            let config = DropdownConfig {
                id: "aircraft_dropdown",
                search_hint: "Search for an aircraft",
                initial_chunk_size: 50,
                auto_focus: true,
                ..Default::default()
            };

            let mut aircraft_dropdown = SearchableDropdown::new(
                vm.all_aircraft,
                vm.aircraft_search,
                Box::new(|aircraft| {
                    vm.selected_aircraft
                        .as_ref()
                        .is_some_and(|a| a.id == aircraft.id)
                }),
                Box::new(|aircraft: &Arc<Aircraft>| {
                    format!("{} {}", aircraft.manufacturer, aircraft.variant)
                }),
                Box::new(|aircraft, search| {
                    aircraft.manufacturer.to_lowercase().contains(search)
                        || aircraft.variant.to_lowercase().contains(search)
                }),
                Box::new(|items| items.choose(&mut rand::rng()).cloned()),
                config,
                vm.aircraft_display_count,
            );

            match aircraft_dropdown.render(ui) {
                DropdownSelection::Item(aircraft) => {
                    events.push(Event::AircraftSelected(Some(aircraft)));
                    events.push(Event::RegenerateRoutesForSelectionChange);
                }
                DropdownSelection::Random(aircraft) => {
                    events.push(Event::AircraftSelected(Some(aircraft)));
                    events.push(Event::RegenerateRoutesForSelectionChange);
                }
                DropdownSelection::Unspecified => {
                    events.push(Event::AircraftSelected(None));
                    events.push(Event::RegenerateRoutesForSelectionChange);
                }
                DropdownSelection::None => {}
            }
        }

        events
    }
}
