use crate::database::DatabasePool;
use crate::gui::components::{
    action_buttons::{ActionButtons, ActionButtonsViewModel},
    route_popup::RoutePopup,
    search_controls::{SearchControls, SearchControlsViewModel},
    selection_controls::{SelectionControls, SelectionControlsViewModel},
    table_display::{TableDisplay, TableDisplayViewModel},
};
use crate::gui::data::{ListItemAircraft, TableItem};
use crate::gui::events::Event;
use crate::gui::services::popup_service::DisplayMode;
use crate::gui::services::{AppService, SearchService, Services};
use crate::gui::state::ApplicationState;
use eframe::egui::{self};
use log;
use std::error::Error;
use std::sync::Arc;

// UI Constants
const RANDOM_AIRPORTS_COUNT: usize = 50;

enum RouteUpdateAction {
    Regenerate,
    Append,
}

/// Simplified GUI application using unified state and services.
/// Much cleaner than having many small ViewModels!
pub struct Gui {
    /// All UI state in one organized place.
    pub state: ApplicationState,
    /// All business logic services in one container.
    pub services: Services,
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

        Ok(Gui { services, state })
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
                self.state.selected_aircraft = aircraft;
                self.maybe_switch_to_route_mode(aircraft_being_selected);
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
                self.services.popup.set_selected_route(Some(route))
            }
            Event::SetShowPopup(show) => self.services.popup.set_alert_visibility(show),
            Event::ToggleAircraftFlownStatus(aircraft_id) => {
                if let Err(e) = self.services.app.toggle_aircraft_flown_status(aircraft_id) {
                    log::error!("Failed to toggle aircraft flown status: {e}");
                } else {
                    self.refresh_aircraft_items_if_needed();
                }
            }
            Event::LoadMoreRoutes => self.load_more_routes_if_needed(),

            // --- SearchControls Events ---
            Event::SearchQueryChanged => {
                self.services.search.set_search_pending(true); // Flag for debouncing
            }
            Event::ClearSearch => {
                // The mutable borrow in the VM clears the text.
                // We just need to update the filtered items.
                self.update_filtered_items();
            }

            // --- RoutePopup Events ---
            Event::MarkRouteAsFlown(route) => {
                if let Err(e) = self.mark_route_as_flown(&route) {
                    log::error!("Failed to mark route as flown: {e}");
                } else {
                    // Refresh the UI after successfully marking the route as flown
                    self.regenerate_routes_for_selection_change();
                }
            }
            Event::ClosePopup => {
                self.services.popup.set_alert_visibility(false);
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
        self.services.popup.set_display_mode(mode.clone());
        match mode {
            DisplayMode::RandomAirports => {
                let random_airports = self.services.app.get_random_airports(RANDOM_AIRPORTS_COUNT);
                let airport_items: Vec<_> = random_airports
                    .iter()
                    .map(|airport| self.services.app.create_list_item_for_airport(airport))
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
                // Clear the table and search query
                self.state.all_items.clear();
                self.services.search.clear_query();
                // Calculate and set the flight statistics
                let stats_result = self.services.app.get_flight_statistics();
                self.state.statistics = Some(stats_result);
            }
            _ => self.update_displayed_items(),
        }
    }

    /// Updates the displayed items based on the current mode.
    pub fn update_displayed_items(&mut self) {
        self.state.all_items = match self.services.popup.display_mode() {
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
        self.update_routes(RouteUpdateAction::Regenerate);
    }

    /// Loads more routes for infinite scrolling.
    fn load_more_routes_if_needed(&mut self) {
        if self.state.is_loading_more_routes || !self.is_route_mode() {
            return;
        }
        self.state.is_loading_more_routes = true;
        self.update_routes(RouteUpdateAction::Append);
        self.state.is_loading_more_routes = false;
    }

    fn update_routes(&mut self, action: RouteUpdateAction) {
        let departure_icao = self.services.app.get_selected_airport_icao(&self.state.selected_departure_airport);
        let display_mode = self.services.popup.display_mode();

        let should_update = match display_mode {
            DisplayMode::RandomRoutes | DisplayMode::SpecificAircraftRoutes => {
                if let Some(aircraft) = &self.state.selected_aircraft {
                    match action {
                        RouteUpdateAction::Regenerate => self.services.app.regenerate_routes_for_aircraft(aircraft, departure_icao.as_deref()),
                        RouteUpdateAction::Append => self.services.app.append_routes_for_aircraft(aircraft, departure_icao.as_deref()),
                    }
                } else {
                    match action {
                        RouteUpdateAction::Regenerate => self.services.app.regenerate_random_routes(departure_icao.as_deref()),
                        RouteUpdateAction::Append => self.services.app.append_random_routes(departure_icao.as_deref()),
                    }
                }
                true
            }
            DisplayMode::NotFlownRoutes => {
                match action {
                    RouteUpdateAction::Regenerate => self.services.app.regenerate_not_flown_routes(departure_icao.as_deref()),
                    RouteUpdateAction::Append => self.services.app.append_not_flown_routes(departure_icao.as_deref()),
                }
                true
            }
            _ => false,
        };

        if should_update {
            self.update_displayed_items();
        }
    }

    // --- Helper methods for state management ---

    fn get_displayed_items(&self) -> &[Arc<TableItem>] {
        if self.services.search.query().trim().is_empty() {
            &self.state.all_items
        } else {
            self.services.search.filtered_items()
        }
    }

    fn set_all_items(&mut self, items: Vec<Arc<TableItem>>) {
        self.state.all_items = items;
        self.update_filtered_items();
    }

    fn update_filtered_items(&mut self) {
        let query = self.services.search.query();
        if query.trim().is_empty() {
            self.services.search.set_filtered_items(Vec::new());
        } else {
            let filtered_items = SearchService::filter_items_static(&self.state.all_items, query);
            self.services.search.set_filtered_items(filtered_items);
        }
    }

    fn is_route_mode(&self) -> bool {
        matches!(
            self.services.popup.display_mode(),
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
            self.services.popup.set_display_mode(new_mode);
        }
    }

    fn handle_route_mode_transition(&mut self) {
        if self.is_route_mode() {
            let new_mode = self.get_appropriate_route_mode();
            if new_mode != *self.services.popup.display_mode() {
                self.services.popup.set_display_mode(new_mode);
            }
        }
    }

    fn refresh_aircraft_items_if_needed(&mut self) {
        if matches!(self.services.popup.display_mode(), DisplayMode::Other) {
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

        // Check if a debounced search should be executed
        if self.services.search.should_execute_search() {
            self.update_filtered_items();
        }

        // Handle route popup
        if self.services.popup.is_alert_visible() {
            let route_popup_vm = crate::gui::components::route_popup::RoutePopupViewModel {
                is_alert_visible: self.services.popup.is_alert_visible(),
                selected_route: self.services.popup.selected_route(),
                display_mode: self.services.popup.display_mode(),
            };
            events.extend(RoutePopup::render(&route_popup_vm, ctx));
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_enabled_ui(!self.services.popup.is_alert_visible(), |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                    // --- Left Panel ---
                    ui.allocate_ui_with_layout(
                        egui::Vec2::new(250.0, ui.available_height()),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            let mut selection_vm = SelectionControlsViewModel {
                                selected_departure_airport: &self.state.selected_departure_airport,
                                selected_aircraft: &self.state.selected_aircraft,
                                departure_dropdown_open: &mut self.state.departure_dropdown_open,
                                aircraft_dropdown_open: &mut self.state.aircraft_dropdown_open,
                                departure_airport_search: &mut self.state.departure_search,
                                aircraft_search: &mut self.state.aircraft_search,
                                departure_display_count: &mut self.state.departure_display_count,
                                aircraft_display_count: &mut self.state.aircraft_display_count,
                                available_airports: self.services.app.airports(),
                                all_aircraft: self.services.app.aircraft(),
                            };
                            events.extend(SelectionControls::render(&mut selection_vm, ui));
                            ui.separator();

                            let action_vm = ActionButtonsViewModel {
                                departure_airport_valid: true, // Always valid - no departure selection means random departure
                            };
                            events.extend(ActionButtons::render(&action_vm, ui));
                        },
                    );

                    ui.separator();

                    // --- Right Panel ---
                    ui.vertical(|ui| {
                        let mut search_vm = SearchControlsViewModel {
                            query: self.services.search.query_mut(),
                        };
                        events.extend(SearchControls::render(&mut search_vm, ui));
                        ui.add_space(10.0);
                        ui.separator();

                        let table_vm = TableDisplayViewModel {
                            items: self.get_displayed_items(),
                            display_mode: self.services.popup.display_mode(),
                            is_loading_more_routes: self.state.is_loading_more_routes,
                            statistics: &self.state.statistics,
                        };
                        events.extend(TableDisplay::render(&table_vm, ui));
                    });
                });
            });
        });

        self.handle_events(events);
    }
}
