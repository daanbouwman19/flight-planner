use crate::database::DatabasePool;
use crate::gui::components::{
    action_buttons::ActionButtons,
    route_popup::RoutePopup,
    search_controls::SearchControls,
    selection_controls::SelectionControls,
    table_display::TableDisplay,
};
use crate::gui::data::{ListItemAircraft, ListItemAirport, TableItem};
use crate::gui::events::Event;
use crate::gui::services::{AppService, SearchService, Services};
use crate::gui::state::{ApplicationState, DisplayMode, ViewState};
use crate::gui::view_model::ViewModel;
use crate::models::Airport;
use eframe::egui::{self};
use log;
use rstar::{AABB, RTreeObject};
use std::error::Error;
use std::sync::Arc;

// UI Constants
const RANDOM_AIRPORTS_COUNT: usize = 50;

/// Simplified GUI application using unified state and services.
/// Much cleaner than having many small ViewModels!
pub struct Gui {
    /// All UI state in one organized place.
    state: ApplicationState,
    /// View-specific state.
    view_state: ViewState,
    /// All business logic services in one container.
    services: Services,
}

/// A spatial index object for airports.
pub struct SpatialAirport {
    pub airport: Arc<Airport>,
}

impl RTreeObject for SpatialAirport {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        let point = [self.airport.Latitude, self.airport.Longtitude];
        AABB::from_point(point)
    }
}

impl Gui {
    /// Creates a new simplified GUI instance.
    pub fn new(
        _cc: &eframe::CreationContext,
        database_pool: DatabasePool,
    ) -> Result<Self, Box<dyn Error>> {
        // Create services container
        let app_service = AppService::new(database_pool)?;
        let services = Services::new(app_service);

        // Create unified application state
        let state = ApplicationState::new();
        let view_state = ViewState::new();

        Ok(Gui {
            services,
            state,
            view_state,
        })
    }

    /// Handles a single UI event, updating the state accordingly.
    fn handle_event(&mut self, event: Event) {
        match event {
            // --- SelectionControls Events ---
            Event::DepartureAirportSelected(airport) => {
                self.maybe_switch_to_route_mode(airport.is_some());
                self.state.selected_departure_airport = airport;
                self.state.departure_dropdown_open = false;
                if self.state.selected_departure_airport.is_some() {
                    self.state.departure_search.clear();
                }
            }
            Event::AircraftSelected(aircraft) => {
                let aircraft_being_selected = aircraft.is_some();
                self.maybe_switch_to_route_mode(aircraft_being_selected);
                self.state.selected_aircraft = aircraft;
                self.handle_route_mode_transition();
                self.state.aircraft_dropdown_open = false;
                if self.state.selected_aircraft.is_some() {
                    self.state.aircraft_search.clear();
                }
            }
            Event::ToggleDepartureAirportDropdown => {
                self.state.departure_dropdown_open = !self.state.departure_dropdown_open;
                if self.state.departure_dropdown_open {
                    self.state.aircraft_dropdown_open = false;
                }
            }
            Event::ToggleAircraftDropdown => {
                self.state.aircraft_dropdown_open = !self.state.aircraft_dropdown_open;
                if self.state.aircraft_dropdown_open {
                    self.state.departure_dropdown_open = false;
                }
            }

            // --- ActionButtons Events ---
            Event::SetDisplayMode(mode) => self.process_display_mode_change(mode),
            Event::RegenerateRoutesForSelectionChange => {
                self.regenerate_routes_for_selection_change()
            }

            // --- TableDisplay Events ---
            Event::RouteSelectedForPopup(route) => {
                self.services
                    .popup
                    .set_selected_route(&mut self.view_state, Some(route))
            }
            Event::SetShowPopup(show) => self
                .services
                .popup
                .set_alert_visibility(&mut self.view_state, show),
            Event::ToggleAircraftFlownStatus(aircraft_id) => {
                if let Err(e) = self.services.app.toggle_aircraft_flown_status(aircraft_id) {
                    log::error!("Failed to toggle aircraft flown status: {e}");
                } else {
                    self.refresh_aircraft_items_if_needed();
                }
            }
            Event::LoadMoreRoutes => self.load_more_routes_if_needed(),

            // --- SearchControls Events ---
            Event::SearchQueryChanged => self.update_filtered_items(),
            Event::ClearSearch => {
                // The mutable borrow in the VM clears the text.
                // We just need to update the filtered items.
                self.update_filtered_items();
            }
        }
    }

    /// Handles multiple events efficiently in batch.
    pub fn handle_events(&mut self, events: Vec<Event>) {
        for event in events {
            self.handle_event(event);
        }
    }

    /// Central logic for processing a display mode change.
    fn process_display_mode_change(&mut self, mode: DisplayMode) {
        self.services
            .popup
            .set_display_mode(&mut self.view_state, mode.clone());
        match mode {
            DisplayMode::RandomAirports => {
                let random_airports = self.services.app.get_random_airports(RANDOM_AIRPORTS_COUNT);
                let airport_items: Vec<_> = random_airports
                    .iter()
                    .map(|airport| {
                        let runways = self.services.app.get_runways_for_airport(airport);
                        let runway_length = runways
                            .iter()
                            .max_by_key(|r| r.Length)
                            .map_or("No runways".to_string(), |r| format!("{}ft", r.Length));
                        ListItemAirport::new(
                            airport.Name.clone(),
                            airport.ICAO.clone(),
                            runway_length,
                        )
                    })
                    .collect();
                let table_items: Vec<Arc<TableItem>> = airport_items
                    .into_iter()
                    .map(|item| Arc::new(TableItem::Airport(item)))
                    .collect();
                self.set_all_items(table_items);
            }
            DisplayMode::Other => {
                // Currently "List all aircraft"
                let aircraft_items: Vec<_> = self
                    .services
                    .app
                    .aircraft()
                    .iter()
                    .map(ListItemAircraft::new)
                    .collect();
                let table_items: Vec<Arc<TableItem>> = aircraft_items
                    .into_iter()
                    .map(|item| Arc::new(TableItem::Aircraft(item)))
                    .collect();
                self.set_all_items(table_items);
            }
            DisplayMode::Statistics => {
                // We'll handle flight statistics in the state later
                self.state.all_items.clear();
                self.services.search.clear_query(&mut self.view_state);
            }
            _ => self.update_displayed_items(),
        }
    }

    /// Updates the displayed items based on the current mode.
    pub fn update_displayed_items(&mut self) {
        self.state.all_items = match self.services.popup.display_mode(&self.view_state) {
            DisplayMode::RandomRoutes
            | DisplayMode::NotFlownRoutes
            | DisplayMode::SpecificAircraftRoutes => self
                .services
                .app
                .route_items()
                .iter()
                .map(|r| Arc::new(TableItem::Route(r.clone())))
                .collect(),
            DisplayMode::History => self
                .services
                .app
                .history_items()
                .iter()
                .map(|h| Arc::new(TableItem::History(h.clone())))
                .collect(),
            DisplayMode::Airports | DisplayMode::RandomAirports => self
                .services
                .app
                .airport_items()
                .iter()
                .map(|a| Arc::new(TableItem::Airport(a.clone())))
                .collect(),
            DisplayMode::Statistics | DisplayMode::Other => Vec::new(), // Handled elsewhere
        };
        self.update_filtered_items();
    }

    /// Regenerates routes when selections change.
    fn regenerate_routes_for_selection_change(&mut self) {
        let departure_icao = self
            .state
            .selected_departure_airport
            .as_ref()
            .map(|a| a.ICAO.as_str());
        let should_update = match self.services.popup.display_mode(&self.view_state) {
            DisplayMode::RandomRoutes | DisplayMode::SpecificAircraftRoutes => {
                if let Some(aircraft) = &self.state.selected_aircraft {
                    self.services
                        .app
                        .regenerate_routes_for_aircraft(aircraft, departure_icao);
                } else {
                    self.services.app.regenerate_random_routes(departure_icao);
                }
                true
            }
            DisplayMode::NotFlownRoutes => {
                self.services
                    .app
                    .regenerate_not_flown_routes(departure_icao);
                true
            }
            _ => false,
        };
        if should_update {
            self.update_displayed_items();
        }
    }

    /// Loads more routes for infinite scrolling.
    fn load_more_routes_if_needed(&mut self) {
        if self.state.is_loading_more_routes || !self.is_route_mode() {
            return;
        }
        self.state.is_loading_more_routes = true;
        let departure_icao = self
            .state
            .selected_departure_airport
            .as_ref()
            .map(|a| a.ICAO.as_str());
        match self.services.popup.display_mode(&self.view_state) {
            DisplayMode::RandomRoutes | DisplayMode::SpecificAircraftRoutes => {
                if let Some(aircraft) = &self.state.selected_aircraft {
                    self.services
                        .app
                        .append_routes_for_aircraft(aircraft, departure_icao);
                } else {
                    self.services.app.append_random_routes(departure_icao);
                }
            }
            DisplayMode::NotFlownRoutes => {
                self.services.app.append_not_flown_routes(departure_icao)
            }
            _ => unreachable!(),
        }
        self.update_displayed_items();
        self.state.is_loading_more_routes = false;
    }

    // --- Helper methods for state management ---

    fn set_all_items(&mut self, items: Vec<Arc<TableItem>>) {
        self.state.all_items = items;
        self.update_filtered_items();
    }

    fn update_filtered_items(&mut self) {
        SearchService::filter_items(&mut self.view_state, &self.state.all_items);
    }

    fn is_route_mode(&self) -> bool {
        matches!(
            self.services.popup.display_mode(&self.view_state),
            DisplayMode::RandomRoutes
                | DisplayMode::NotFlownRoutes
                | DisplayMode::SpecificAircraftRoutes
        )
    }

    fn get_appropriate_route_mode(&self) -> DisplayMode {
        if self.state.selected_aircraft.is_some() {
            DisplayMode::SpecificAircraftRoutes
        } else {
            DisplayMode::RandomRoutes
        }
    }

    fn maybe_switch_to_route_mode(&mut self, selection_being_made: bool) {
        if selection_being_made && !self.is_route_mode() {
            let new_mode = self.get_appropriate_route_mode();
            self.services
                .popup
                .set_display_mode(&mut self.view_state, new_mode);
        }
    }

    fn handle_route_mode_transition(&mut self) {
        if self.is_route_mode() {
            let new_mode = self.get_appropriate_route_mode();
            if new_mode != *self.services.popup.display_mode(&self.view_state) {
                self.services
                    .popup
                    .set_display_mode(&mut self.view_state, new_mode);
            }
        }
    }

    fn refresh_aircraft_items_if_needed(&mut self) {
        if matches!(
            self.services.popup.display_mode(&self.view_state),
            DisplayMode::Other
        ) {
            let aircraft_items: Vec<_> = self
                .services
                .app
                .aircraft()
                .iter()
                .map(ListItemAircraft::new)
                .collect();
            let table_items: Vec<Arc<TableItem>> = aircraft_items
                .into_iter()
                .map(|item| Arc::new(TableItem::Aircraft(item)))
                .collect();
            self.set_all_items(table_items);
        }
    }

    pub fn mark_route_as_flown(
        &mut self,
        route: &crate::gui::data::ListItemRoute,
    ) -> Result<(), Box<dyn Error>> {
        self.services.app.mark_route_as_flown(route)
    }
}

impl eframe::App for Gui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut events = Vec::new();

        // Handle popup input first, as it might block other interactions.
        if self.view_state.show_route_popup {
            RoutePopup::render(&mut self.view_state, &self.state, ctx, |event| {
                events.push(event);
            });
        }

        let vm = ViewModel::new(
            &self.state,
            &self.view_state,
            self.services.app.aircraft(),
            self.services.app.airports(),
        );

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_enabled_ui(!vm.view_state.show_route_popup, |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                    // --- Left Panel ---
                    ui.allocate_ui_with_layout(
                        egui::Vec2::new(250.0, ui.available_height()),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            // Selection controls
                            SelectionControls::render(
                                &vm,
                                &mut self.state.departure_search,
                                &mut self.state.aircraft_search,
                                &mut self.state.departure_dropdown_open,
                                &mut self.state.aircraft_dropdown_open,
                                &mut self.state.departure_display_count,
                                &mut self.state.aircraft_display_count,
                                ui,
                                &mut events,
                            );
                            ui.separator();

                            // Action buttons
                            ActionButtons::render(&vm, ui, &mut events);
                        },
                    );

                    ui.separator();

                    // --- Right Panel ---
                    ui.vertical(|ui| {
                        // Search controls
                        SearchControls::render(&mut self.view_state.table_search, ui, &mut events);
                        ui.add_space(10.0);
                        ui.separator();

                        // Table display
                        TableDisplay::render(&vm, ui, &mut events);
                    });
                });
            });
        });

        // Handle all collected events efficiently
        self.handle_events(events);
    }
}
