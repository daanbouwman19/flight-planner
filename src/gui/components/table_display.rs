use crate::gui::data::{ListItemAircraft, ListItemAirport, ListItemHistory, ListItemRoute, TableItem};
use crate::gui::events::Event;
use crate::gui::state::popup_state::DisplayMode;
use crate::modules::data_operations::FlightStatistics;
use egui::Ui;
use std::error::Error;
use std::sync::Arc;

// --- View Model ---

/// View-model for the `TableDisplay` component.
pub struct TableDisplayViewModel<'a> {
    pub items: &'a [Arc<TableItem>],
    pub display_mode: &'a DisplayMode,
    pub is_loading_more_routes: bool,
    pub statistics: &'a Option<Result<FlightStatistics, Box<dyn Error + Send + Sync>>>,
}

// --- Component ---

/// Standard button size for aircraft action buttons
const AIRCRAFT_ACTION_BUTTON_SIZE: [f32; 2] = [120.0, 20.0];

pub struct TableDisplay;

impl TableDisplay {
    /// Renders the main display area with items.
    pub fn render(vm: &TableDisplayViewModel, ui: &mut Ui) -> Vec<Event> {
        let mut events = Vec::new();

        if *vm.display_mode == DisplayMode::Statistics {
            Self::render_statistics(vm, ui);
            return events;
        }

        let items_to_display = vm.items;

        if items_to_display.is_empty() {
            ui.label("No items to display");
            return events;
        }

        let scroll_area = egui::ScrollArea::vertical().auto_shrink([false, true]);
        let scroll_response = scroll_area.show(ui, |ui| {
            egui::Grid::new("items_grid").striped(true).show(ui, |ui| {
                Self::render_headers(vm, ui);
                events.extend(Self::render_data_rows(vm, ui, items_to_display));

                if vm.is_loading_more_routes {
                    ui.horizontal(|ui| {
                        ui.label("Loading more routes...");
                        ui.spinner();
                    });
                    ui.end_row();
                }
            });
        });

        if let Some(event) = Self::handle_infinite_scrolling(vm, &scroll_response, items_to_display) {
            events.push(event);
        }

        events
    }

    fn handle_infinite_scrolling(
        vm: &TableDisplayViewModel,
        scroll_response: &egui::scroll_area::ScrollAreaOutput<()>,
        items: &[Arc<TableItem>],
    ) -> Option<Event> {
        let is_route_mode = matches!(
            vm.display_mode,
            DisplayMode::RandomRoutes | DisplayMode::NotFlownRoutes | DisplayMode::SpecificAircraftRoutes
        );

        if !is_route_mode || vm.is_loading_more_routes {
            return None;
        }

        let state = &scroll_response.state;
        let content_size = scroll_response.content_size;
        let available_size = scroll_response.inner_rect.size();
        let scroll_position = state.offset.y;
        let max_scroll = (content_size.y - available_size.y).max(0.0);

        if items.len() >= 10 && max_scroll > 0.0 && (scroll_position / max_scroll) > 0.8 {
            return Some(Event::LoadMoreRoutes);
        }
        None
    }

    fn render_headers(vm: &TableDisplayViewModel, ui: &mut Ui) {
        match vm.display_mode {
            DisplayMode::RandomRoutes | DisplayMode::NotFlownRoutes | DisplayMode::SpecificAircraftRoutes => {
                ui.label("Aircraft");
                ui.label("From");
                ui.label("To");
                ui.label("Distance");
                ui.label("Actions");
                ui.end_row();
            }
            DisplayMode::History => {
                ui.label("Aircraft");
                ui.label("From");
                ui.label("To");
                ui.label("Date Flown");
                ui.end_row();
            }
            DisplayMode::Airports | DisplayMode::RandomAirports => {
                ui.label("ICAO");
                ui.label("Name");
                ui.label("Runway Length");
                ui.end_row();
            }
            DisplayMode::Other => {
                ui.label("Manufacturer");
                ui.label("Variant");
                ui.label("ICAO Code");
                ui.label("Range");
                ui.label("Category");
                ui.label("Date Flown");
                ui.label("Action");
                ui.end_row();
            }
            DisplayMode::Statistics => {}
        }
    }

    fn render_data_rows(vm: &TableDisplayViewModel, ui: &mut Ui, items: &[Arc<TableItem>]) -> Vec<Event> {
        let mut events = Vec::new();
        for item in items {
            match item.as_ref() {
                TableItem::Route(route) => events.extend(Self::render_route_row(vm, ui, route)),
                TableItem::History(history) => Self::render_history_row(ui, history),
                TableItem::Airport(airport) => Self::render_airport_row(ui, airport),
                TableItem::Aircraft(aircraft) => events.extend(Self::render_aircraft_row(ui, aircraft)),
            }
        }
        events
    }

    fn render_route_row(vm: &TableDisplayViewModel, ui: &mut Ui, route: &ListItemRoute) -> Vec<Event> {
        let mut events = Vec::new();
        ui.label(format!("{} {}", route.aircraft.manufacturer, route.aircraft.variant));
        ui.label(format!("{} ({})", route.departure.Name, route.departure.ICAO));
        ui.label(format!("{} ({})", route.destination.Name, route.destination.ICAO));
        ui.label(&route.route_length);

        ui.horizontal(|ui| {
            if matches!(
                vm.display_mode,
                DisplayMode::RandomRoutes
                    | DisplayMode::NotFlownRoutes
                    | DisplayMode::SpecificAircraftRoutes
            ) && ui.button("Select").clicked()
            {
                events.push(Event::RouteSelectedForPopup(route.clone()));
                events.push(Event::SetShowPopup(true));
            }
        });
        ui.end_row();
        events
    }

    fn render_history_row(ui: &mut Ui, history: &ListItemHistory) {
        ui.label(&history.aircraft_name);
        ui.label(&history.departure_icao);
        ui.label(&history.arrival_icao);
        ui.label(&history.date);
        ui.end_row();
    }

    fn render_airport_row(ui: &mut Ui, airport: &ListItemAirport) {
        ui.label(&airport.icao);
        ui.label(&airport.name);
        ui.label(&airport.longest_runway_length);
        ui.end_row();
    }

    fn render_aircraft_row(ui: &mut Ui, aircraft: &ListItemAircraft) -> Vec<Event> {
        let mut events = Vec::new();
        ui.label(&aircraft.manufacturer);
        ui.label(&aircraft.variant);
        ui.label(&aircraft.icao_code);
        ui.label(&aircraft.range);
        ui.label(&aircraft.category);
        ui.label(&aircraft.date_flown);

        ui.horizontal(|ui| {
            let button_text = if aircraft.flown > 0 { "Mark Not Flown" } else { " Mark Flown  " };
            if ui.add_sized(AIRCRAFT_ACTION_BUTTON_SIZE, egui::Button::new(button_text)).clicked() {
                events.push(Event::ToggleAircraftFlownStatus(aircraft.id));
            }
        });
        ui.end_row();
        events
    }

    fn render_statistics(vm: &TableDisplayViewModel, ui: &mut Ui) {
        ui.heading("Flight Statistics");
        ui.separator();
        ui.add_space(10.0);

        match vm.statistics {
            Some(Ok(stats)) => {
                egui::Grid::new("statistics_grid").striped(true).show(ui, |ui| {
                    ui.label("Total Flights:");
                    ui.label(stats.total_flights.to_string());
                    ui.end_row();
                    ui.label("Total Distance:");
                    ui.label(format!("{} NM", stats.total_distance));
                    ui.end_row();
                    // ... and so on for all stats
                });
            }
            Some(Err(e)) => {
                ui.label(format!("Error loading statistics: {e}"));
            }
            None => {
                ui.label("Loading statistics...");
                ui.spinner();
            }
        }
    }
}
