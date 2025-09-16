use crate::database::DatabasePool;
use crate::gui::components::{
    action_buttons::{ActionButtons, ActionButtonsViewModel},
    route_popup::RoutePopup,
    search_controls::{SearchControls, SearchControlsViewModel},
    selection_controls::{SelectionControls, SelectionControlsViewModel},
    table_display::{TableDisplay, TableDisplayViewModel},
};
use crate::gui::data::{ListItemAircraft, ListItemRoute, TableItem};
use crate::gui::events::Event;
use crate::gui::services::popup_service::DisplayMode;
use crate::gui::services::{AppService, SearchService, Services};
use crate::gui::state::ApplicationState;
use crate::modules::data_operations::DataOperations;
use eframe::egui::{self};
use log;
use std::error::Error;
use std::sync::{Arc, mpsc};

// UI Constants
const RANDOM_AIRPORTS_COUNT: usize = 50;

#[derive(Clone, Copy)]
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
    /// Sender for route generation results.
    route_sender: mpsc::Sender<Vec<ListItemRoute>>,
    /// Receiver for route generation results.
    route_receiver: mpsc::Receiver<Vec<ListItemRoute>>,
    /// Sender for search results.
    search_sender: mpsc::Sender<Vec<Arc<TableItem>>>,
    /// Receiver for search results.
    search_receiver: mpsc::Receiver<Vec<Arc<TableItem>>>,
    /// A place to store a route update request
    route_update_request: Option<RouteUpdateAction>,
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

        // Create channels for background tasks
        let (route_sender, route_receiver) = mpsc::channel();
        let (search_sender, search_receiver) = mpsc::channel();

        Ok(Gui {
            services,
            state,
            route_sender,
            route_receiver,
            search_sender,
            search_receiver,
            route_update_request: None,
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
        self.update_routes(RouteUpdateAction::Append);
    }

    fn update_routes(&mut self, action: RouteUpdateAction) {
        if self.state.is_loading_more_routes {
            return; // Don't stack requests
        }
        self.route_update_request = Some(action);
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

    fn spawn_route_generation_thread(&mut self, ctx: Option<&egui::Context>) {
        if let Some(_action) = self.route_update_request
            && !self.state.is_loading_more_routes {
                self.state.is_loading_more_routes = true;

                let sender = self.route_sender.clone();
                let ctx_clone = ctx.cloned();
                let departure_icao = self
                    .services
                    .app
                    .get_selected_airport_icao(&self.state.selected_departure_airport);
                let display_mode = self.services.popup.display_mode().clone();
                let selected_aircraft = self.state.selected_aircraft.clone();
                let route_generator = self.services.app.route_generator().clone();
                let all_aircraft = self.services.app.aircraft().to_vec();

                std::thread::spawn(move || {
                    let routes = match (display_mode, &selected_aircraft) {
                        (
                            DisplayMode::RandomRoutes | DisplayMode::SpecificAircraftRoutes,
                            Some(aircraft),
                        ) => DataOperations::generate_routes_for_aircraft(
                            &route_generator,
                            aircraft,
                            departure_icao.as_deref(),
                        ),
                        (
                            DisplayMode::RandomRoutes | DisplayMode::SpecificAircraftRoutes,
                            None,
                        ) => DataOperations::generate_random_routes(
                            &route_generator,
                            &all_aircraft,
                            departure_icao.as_deref(),
                        ),
                        (DisplayMode::NotFlownRoutes, _) => {
                            DataOperations::generate_not_flown_routes(
                                &route_generator,
                                &all_aircraft,
                                departure_icao.as_deref(),
                            )
                        }
                        _ => Vec::new(),
                    };

                    if sender.send(routes).is_ok()
                        && let Some(ctx) = ctx_clone {
                            ctx.request_repaint();
                        }
                });
            }
    }

    fn spawn_search_thread(&mut self, ctx: Option<&egui::Context>) {
        if self.services.search.should_execute_search() && !self.state.is_searching {
            self.state.is_searching = true;
            let sender = self.search_sender.clone();
            let ctx_clone = ctx.cloned();
            let all_items = self.state.all_items.clone();
            let query = self.services.search.query().to_string();

            std::thread::spawn(move || {
                let filtered_items = SearchService::filter_items_static(&all_items, &query);
                if sender.send(filtered_items).is_ok()
                    && let Some(ctx) = ctx_clone {
                        ctx.request_repaint();
                    }
            });
        }
    }
}

impl eframe::App for Gui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut events = Vec::new();

        // --- Background Task Results ---
        // Check for results from the route generation thread
        if let Ok(new_routes) = self.route_receiver.try_recv() {
            if let Some(RouteUpdateAction::Append) = self.route_update_request {
                let mut current_routes = self.services.app.route_items().to_vec();
                current_routes.extend(new_routes);
                self.services.app.set_route_items(current_routes);
            } else {
                self.services.app.set_route_items(new_routes);
            }
            self.update_displayed_items();
            self.state.is_loading_more_routes = false;
            self.route_update_request = None; // Clear the request
        }

        // Check for results from the search thread
        if let Ok(filtered_items) = self.search_receiver.try_recv() {
            self.services.search.set_filtered_items(filtered_items);
            self.state.is_searching = false;
        }

        // --- Background Task Spawning ---
        self.spawn_route_generation_thread(Some(ctx));
        self.spawn_search_thread(Some(ctx));

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::DatabasePool;
    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
    use std::time::Duration;

    const AIRCRAFT_MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
    const AIRPORT_MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations_airport_database");

    fn setup_gui() -> Gui {
        use crate::models::{Airport, NewAircraft, Runway};
        use crate::schema::{aircraft as aircraft_schema, Airports as airports_schema, Runways as runways_schema};
        use diesel::RunQueryDsl;
        use std::sync::atomic::{AtomicUsize, Ordering};

        static DB_COUNTER: AtomicUsize = AtomicUsize::new(0);
        let aircraft_db_name = format!("file:aircraft_test_db_{}?mode=memory&cache=shared", DB_COUNTER.fetch_add(1, Ordering::SeqCst));
        let airport_db_name = format!("file:airport_test_db_{}?mode=memory&cache=shared", DB_COUNTER.fetch_add(1, Ordering::SeqCst));

        // In-memory database for testing
        let db_pool = DatabasePool::new(Some(&aircraft_db_name), Some(&airport_db_name)).unwrap();
        let mut aircraft_conn = db_pool.aircraft_pool.get().unwrap();
        let mut airport_conn = db_pool.airport_pool.get().unwrap();

        aircraft_conn
            .run_pending_migrations(AIRCRAFT_MIGRATIONS)
            .expect("Failed to run aircraft migrations in test");
        airport_conn
            .run_pending_migrations(AIRPORT_MIGRATIONS)
            .expect("Failed to run airport migrations in test");

        // Insert test data
        let test_aircraft = NewAircraft {
            manufacturer: "TestAir".to_string(),
            variant: "T-1".to_string(),
            icao_code: "TEST".to_string(),
            flown: 0,
            aircraft_range: 1000,
            category: "A".to_string(),
            cruise_speed: 400,
            date_flown: None,
            takeoff_distance: Some(4000),
        };
        diesel::insert_into(aircraft_schema::table)
            .values(&test_aircraft)
            .execute(&mut aircraft_conn)
            .unwrap();

        let test_airport1 = Airport {
            ID: 1, Name: "Airport A".to_string(), ICAO: "AAAA".to_string(), PrimaryID: None,
            Latitude: 0.0, Longtitude: 0.0, Elevation: 0, TransitionAltitude: None, TransitionLevel: None, SpeedLimit: None, SpeedLimitAltitude: None,
        };
        let test_airport2 = Airport {
            ID: 2, Name: "Airport B".to_string(), ICAO: "BBBB".to_string(), PrimaryID: None,
            Latitude: 1.0, Longtitude: 1.0, Elevation: 0, TransitionAltitude: None, TransitionLevel: None, SpeedLimit: None, SpeedLimitAltitude: None,
        };
        diesel::insert_into(airports_schema::table)
            .values(&vec![test_airport1, test_airport2])
            .execute(&mut airport_conn)
            .unwrap();

        let test_runway1 = Runway {
            ID: 1, AirportID: 1, Length: 14000, Width: 150, Surface: "ASPH".to_string(),
            Latitude: 0.0, Longtitude: 0.0, Elevation: 0, Ident: "09/27".to_string(), TrueHeading: 90.0,
        };
        let test_runway2 = Runway {
            ID: 2, AirportID: 2, Length: 14000, Width: 150, Surface: "ASPH".to_string(),
            Latitude: 1.0, Longtitude: 1.0, Elevation: 0, Ident: "01/19".to_string(), TrueHeading: 10.0,
        };
        diesel::insert_into(runways_schema::table)
            .values(&vec![test_runway1, test_runway2])
            .execute(&mut airport_conn)
            .unwrap();


        Gui::new(
            &eframe::CreationContext::_new_kittest(egui::Context::default()),
            db_pool,
        )
        .unwrap()
    }

    #[test]
    fn test_background_route_generation_sends_results() {
        let mut gui = setup_gui();
        gui.update_routes(RouteUpdateAction::Regenerate);
        gui.spawn_route_generation_thread(None);

        let result = gui
            .route_receiver
            .recv_timeout(Duration::from_secs(5))
            .unwrap();

        assert!(!result.is_empty(), "Should receive some routes");
        let first_route = &result[0];
        assert_ne!(
            first_route.departure.ICAO, first_route.destination.ICAO,
            "Departure and destination should be different"
        );
    }

    #[test]
    fn test_background_search_sends_filtered_results() {
        let mut gui = setup_gui();

        // Populate all_items with some test data
        let item1 = Arc::new(TableItem::Airport(crate::gui::data::ListItemAirport::new(
            "Airport A".to_string(),
            "AAAA".to_string(),
            "10000ft".to_string(),
        )));
        let item2 = Arc::new(TableItem::Airport(crate::gui::data::ListItemAirport::new(
            "Airport B".to_string(),
            "BBBB".to_string(),
            "12000ft".to_string(),
        )));
        gui.state.all_items = vec![item1.clone(), item2.clone()];
        gui.services.search.set_query("Airport A".to_string());
        gui.services.search.force_search_pending();

        // Manually trigger the search thread
        gui.spawn_search_thread(None);

        let result = gui
            .search_receiver
            .recv_timeout(Duration::from_secs(5))
            .unwrap();

        assert_eq!(result.len(), 1, "Should find exactly one match");
        assert_eq!(
            result[0].as_ref(),
            item1.as_ref(),
            "The found item should be 'Airport A'"
        );
    }
}
