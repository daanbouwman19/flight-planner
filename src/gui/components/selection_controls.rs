use crate::gui::components::dropdowns::{
    DropdownAction, render_aircraft_dropdown, render_airport_dropdown,
};
use crate::gui::events::Event;
use crate::models::{Aircraft, Airport};
use egui::Ui;
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

        // Preserving "Always Focus" behavior by resetting to true every frame
        let mut always_focus = true;

        let departure_action = render_airport_dropdown(
            ui,
            "Departure Airport:",
            "Select departure airport",
            vm.selected_departure_airport.as_ref(),
            vm.available_airports,
            vm.departure_airport_search,
            vm.departure_display_count,
            *vm.departure_dropdown_open,
            &mut always_focus,
            Event::ToggleDepartureAirportDropdown,
        );

        match departure_action {
            DropdownAction::Toggle => events.push(Event::ToggleDepartureAirportDropdown),
            DropdownAction::Select(item) => {
                events.push(Event::DepartureAirportSelected(Some(item)));
                events.push(Event::RegenerateRoutesForSelectionChange);
            }
            DropdownAction::Unselect => {
                events.push(Event::DepartureAirportSelected(None));
                events.push(Event::RegenerateRoutesForSelectionChange);
            }
            DropdownAction::None => {}
        }

        // Reset for next control
        always_focus = true;

        let aircraft_action = render_aircraft_dropdown(
            ui,
            "Aircraft:",
            "Select aircraft",
            vm.selected_aircraft.as_ref(),
            vm.all_aircraft,
            vm.aircraft_search,
            vm.aircraft_display_count,
            *vm.aircraft_dropdown_open,
            &mut always_focus,
            Event::ToggleAircraftDropdown,
        );

        match aircraft_action {
            DropdownAction::Toggle => events.push(Event::ToggleAircraftDropdown),
            DropdownAction::Select(item) => {
                events.push(Event::AircraftSelected(Some(item)));
                events.push(Event::RegenerateRoutesForSelectionChange);
            }
            DropdownAction::Unselect => {
                events.push(Event::AircraftSelected(None));
                events.push(Event::RegenerateRoutesForSelectionChange);
            }
            DropdownAction::None => {}
        }

        events
    }
}
