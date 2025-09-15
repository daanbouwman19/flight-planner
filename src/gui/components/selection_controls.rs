use crate::gui::components::searchable_dropdown::{DropdownSelection, SearchableDropdown};
use crate::gui::events::Event;
use crate::models::{Aircraft, Airport};
use egui::Ui;
use std::sync::Arc;

/// ViewModel for the SelectionControls component.
pub struct SelectionControlsViewModel<'a> {
    pub selected_departure_airport: &'a Option<Arc<Airport>>,
    pub selected_aircraft: &'a Option<Arc<Aircraft>>,
    pub departure_dropdown_open: &'a mut bool,
    pub aircraft_dropdown_open: &'a mut bool,
    pub departure_airport_search: &'a mut String,
    pub aircraft_search: &'a mut String,
    pub departure_display_count: &'a mut usize,
    pub aircraft_display_count: &'a mut usize,
    pub available_airports: &'a [Arc<Airport>],
    pub all_aircraft: &'a [Arc<Aircraft>],
}

/// The selection controls component.
pub struct SelectionControls;

impl SelectionControls {
    /// Renders the selection controls for departure airport and aircraft.
    pub fn render(vm: &mut SelectionControlsViewModel, ui: &mut Ui) -> Vec<Event> {
        let mut events = Vec::new();

        ui.add_space(10.0);
        ui.label("Selections");
        ui.separator();

        let departure_display_text = if let Some(airport) = vm.selected_departure_airport {
            airport.Name.clone()
        } else if !vm.departure_airport_search.is_empty() {
            vm.departure_airport_search.clone()
        } else {
            "Select departure airport".to_string()
        };

        // Departure airport dropdown
        let departure_dropdown = SearchableDropdown::new(
            "Departure",
            vm.departure_airport_search,
            &departure_display_text,
            vm.departure_dropdown_open,
            vm.available_airports,
            vm.departure_display_count,
            |airport: &Arc<Airport>| format!("{} ({})", airport.Name, airport.ICAO),
        );
        if let Some(DropdownSelection::Item(selected_airport)) = departure_dropdown.render(ui) {
            events.push(Event::DepartureAirportSelected(Some(selected_airport)));
        }

        let aircraft_display_text = if let Some(aircraft) = vm.selected_aircraft {
            aircraft.variant.clone()
        } else if !vm.aircraft_search.is_empty() {
            vm.aircraft_search.clone()
        } else {
            "Select aircraft".to_string()
        };

        // Aircraft dropdown
        let aircraft_dropdown = SearchableDropdown::new(
            "Aircraft",
            vm.aircraft_search,
            &aircraft_display_text,
            vm.aircraft_dropdown_open,
            vm.all_aircraft,
            vm.aircraft_display_count,
            |aircraft: &Arc<Aircraft>| aircraft.variant.clone(),
        );
        if let Some(DropdownSelection::Item(selected_aircraft)) = aircraft_dropdown.render(ui) {
            events.push(Event::AircraftSelected(Some(selected_aircraft)));
        }

        events
    }
}
