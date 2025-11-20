use crate::database::DatabasePool;
use crate::gui::components::{
    action_buttons::{ActionButtons, ActionButtonsViewModel},
    add_history_popup::AddHistoryPopup,
    route_popup::RoutePopup,
    search_controls::{SearchControls, SearchControlsViewModel},
    selection_controls::{SelectionControls, SelectionControlsViewModel},
    settings_popup::{SettingsPopup, SettingsPopupViewModel},
    table_display::{TableDisplay, TableDisplayViewModel},
};
use crate::gui::data::{ListItemAircraft, ListItemRoute, TableItem};
use crate::gui::events::Event;
use crate::gui::services::popup_service::DisplayMode;
use crate::gui::services::{AppService, SearchService, Services};
use crate::gui::state::{AddHistoryState, ApplicationState};
use crate::models::weather::{Metar, WeatherError};
use diesel_migrations::MigrationHarness;
use eframe::egui::{self};
use log;
use rayon::prelude::*;
use std::collections::HashSet;
use std::error::Error;
use std::sync::{Arc, mpsc};

// UI Constants
const RANDOM_AIRPORTS_COUNT: usize = 50;
/// Query length threshold for instant search without debouncing
const INSTANT_SEARCH_MIN_QUERY_LEN: usize = 2;

/// Defines the type of action to be taken when updating routes.
#[derive(Clone, Copy)]
pub enum RouteUpdateAction {
    /// Replace the existing routes with a new set.
    Regenerate,
    /// Append new routes to the existing list.
    Append,
}

/// The main struct for the Flight Planner GUI.
///
/// This struct holds the application's state, services, and communication
/// channels for background tasks. It is responsible for rendering the UI and
/// handling user events.
pub struct Gui {
    /// The unified state of the application, containing all UI-related data.
    pub state: ApplicationState,
    /// A container for all business logic and application services.
    /// This is `None` during initialization.
    pub services: Option<Services>,
    /// Receiver for the initialized services from the background thread.
    pub startup_receiver: Option<mpsc::Receiver<Result<Services, String>>>,
    /// Error message if initialization fails.
    pub startup_error: Option<String>,
    /// The sender part of a channel for sending route generation results from a background thread.
    pub route_sender: mpsc::Sender<Vec<ListItemRoute>>,
    /// The receiver part of a channel for receiving route generation results.
    pub route_receiver: mpsc::Receiver<Vec<ListItemRoute>>,
    /// The sender part of a channel for sending search results from a background thread.
    pub search_sender: mpsc::Sender<Vec<Arc<TableItem>>>,
    /// The receiver part of a channel for receiving search results.
    pub search_receiver: mpsc::Receiver<Vec<Arc<TableItem>>>,
    /// The sender part of a channel for sending weather results.
    pub weather_sender: mpsc::Sender<(String, Result<Metar, WeatherError>)>,
    /// The receiver part of a channel for receiving weather results.
    pub weather_receiver: mpsc::Receiver<(String, Result<Metar, WeatherError>)>,
    /// Stores a pending request to update routes, used to trigger background generation.
    pub route_update_request: Option<RouteUpdateAction>,
}

impl Gui {
    /// Creates a new `Gui` instance.
    ///
    /// This function initializes the application services, state, and communication
    /// channels required for the GUI to operate.
    ///
    /// # Arguments
    ///
    /// * `_cc` - The eframe creation context, which is not used in this case.
    /// * `database_pool` - The database connection pool for application services.
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `Gui` instance on success, or an error if
    /// initialization fails.
    pub fn new(cc: &eframe::CreationContext) -> Result<Self, Box<dyn Error>> {
        log::info!("Gui::new: Called.");
        // Create channels for background tasks
        let (route_sender, route_receiver) = mpsc::channel();
        let (search_sender, search_receiver) = mpsc::channel();
        let (weather_sender, weather_receiver) = mpsc::channel();
        let (startup_sender, startup_receiver) = mpsc::channel();

        let ctx = cc.egui_ctx.clone();

        // Spawn background initialization
        std::thread::spawn(move || {
            log::info!("Gui::new: Background thread started.");
            let start = std::time::Instant::now();
            let result = (|| -> Result<Services, String> {
                // Initialize DatabasePool
                let database_pool = DatabasePool::new(None, None).map_err(|e| e.to_string())?;

                // Run migrations
                database_pool
                    .aircraft_pool
                    .get()
                    .map_err(|e| e.to_string())?
                    .run_pending_migrations(crate::MIGRATIONS)
                    .map_err(|e| e.to_string())?;

                database_pool
                    .airport_pool
                    .get()
                    .map_err(|e| e.to_string())?
                    .run_pending_migrations(crate::MIGRATIONS)
                    .map_err(|e| e.to_string())?;

                // Import aircraft CSV if empty
                if let Some(csv_path) = crate::modules::aircraft::find_aircraft_csv_path() {
                    if let Ok(mut conn) = database_pool.aircraft_pool.get() {
                        let _ = crate::modules::aircraft::import_aircraft_from_csv_if_empty(
                            &mut conn, &csv_path,
                        );
                    }
                }

                let mut app_service = AppService::new(database_pool).map_err(|e| e.to_string())?;
                let api_key = app_service
                    .get_api_key()
                    .map_err(|e| e.to_string())?
                    .unwrap_or_else(|| std::env::var("AVWX_API_KEY").unwrap_or_default());
                Ok(Services::new(app_service, api_key))
            })();
            log::info!(
                "Gui::new: Background thread finished in {:?}",
                start.elapsed()
            );
            let _ = startup_sender.send(result);
            ctx.request_repaint();
        });

        Ok(Gui {
            services: None,
            startup_receiver: Some(startup_receiver),
            startup_error: None,
            state: ApplicationState::new(),
            route_sender,
            route_receiver,
            search_sender,
            search_receiver,
            weather_sender,
            weather_receiver,
            route_update_request: None,
        })
    }

    /// Handles a single UI event, updating the state accordingly.
    fn handle_event(&mut self, event: Event, ctx: &egui::Context) {
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
                if let Some(services) = &mut self.services {
                    services.popup.set_selected_route(Some(route.clone()));
                    services.popup.show_alert();

                    let departure = route.departure.ICAO.clone();
                    let destination = route.destination.ICAO.clone();
                    let weather_service = services.weather.clone();
                    let sender = self.weather_sender.clone();
                    let ctx_clone = ctx.clone();

                    std::thread::spawn(move || {
                        let res = weather_service.fetch_metar(&departure);
                        send_and_repaint(&sender, (departure, res), Some(ctx_clone));
                    });

                    let weather_service = services.weather.clone();
                    let sender = self.weather_sender.clone();
                    let ctx_clone = ctx.clone();

                    std::thread::spawn(move || {
                        let res = weather_service.fetch_metar(&destination);
                        send_and_repaint(&sender, (destination, res), Some(ctx_clone));
                    });
                }
            }
            Event::SetShowPopup(show) => {
                if let Some(services) = &mut self.services {
                    services.popup.set_alert_visibility(show)
                }
            }
            Event::ToggleAircraftFlownStatus(aircraft_id) => {
                if let Some(services) = &mut self.services {
                    if let Err(e) = services.app.toggle_aircraft_flown_status(aircraft_id) {
                        log::error!("Failed to toggle aircraft flown status: {e}");
                    } else {
                        self.refresh_aircraft_items_if_needed();
                    }
                }
            }
            Event::LoadMoreRoutes => self.load_more_routes_if_needed(),

            // --- SearchControls Events ---
            Event::SearchQueryChanged => {
                if let Some(services) = &mut self.services {
                    // The query has already been updated via the mutable reference in the view model
                    let query = services.search.query();

                    // For very short queries (1-2 characters), search instantly
                    // For longer queries, use debouncing to avoid excessive searches
                    if query.len() <= INSTANT_SEARCH_MIN_QUERY_LEN {
                        services.search.force_search_pending();
                    } else {
                        // Standard debouncing for longer queries
                        services.search.set_search_pending(true);
                        services
                            .search
                            .set_last_search_request(Some(std::time::Instant::now()));
                    }
                }
            }
            Event::ClearSearch => {
                if let Some(services) = &mut self.services {
                    // The mutable borrow in the VM has already cleared the text.
                    // Now we need to clear the search service state as well.
                    services.search.clear_query();
                }
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
                if let Some(services) = &mut self.services {
                    services.popup.set_alert_visibility(false);
                }
            }

            // --- AddHistoryPopup Events ---
            Event::ShowAddHistoryPopup => {
                self.state.add_history.show_popup = true;
            }
            Event::CloseAddHistoryPopup => {
                // Reset the entire popup state in one go.
                self.state.add_history = AddHistoryState::new();
            }
            Event::AddHistoryEntry {
                aircraft,
                departure,
                destination,
            } => {
                if let Some(services) = &mut self.services {
                    if let Err(e) =
                        services
                            .app
                            .add_history_entry(&aircraft, &departure, &destination)
                    {
                        log::error!("Failed to add history entry: {e}");
                    } else {
                        // Refresh history view if it's active
                        if services.popup.display_mode() == &DisplayMode::History {
                            self.update_displayed_items();
                        }
                        // Close the popup
                        self.handle_event(Event::CloseAddHistoryPopup, ctx);
                    }
                }
            }
            Event::ToggleAddHistoryAircraftDropdown => {
                self.state.add_history.aircraft_dropdown_open =
                    !self.state.add_history.aircraft_dropdown_open;
                self.state.add_history.aircraft_search_autofocus =
                    self.state.add_history.aircraft_dropdown_open;
                self.state.add_history.departure_dropdown_open = false;
                self.state.add_history.destination_dropdown_open = false;
            }
            Event::ToggleAddHistoryDepartureDropdown => {
                self.state.add_history.departure_dropdown_open =
                    !self.state.add_history.departure_dropdown_open;
                self.state.add_history.departure_search_autofocus =
                    self.state.add_history.departure_dropdown_open;
                self.state.add_history.aircraft_dropdown_open = false;
                self.state.add_history.destination_dropdown_open = false;
            }
            Event::ToggleAddHistoryDestinationDropdown => {
                self.state.add_history.destination_dropdown_open =
                    !self.state.add_history.destination_dropdown_open;
                self.state.add_history.destination_search_autofocus =
                    self.state.add_history.destination_dropdown_open;
                self.state.add_history.aircraft_dropdown_open = false;
                self.state.add_history.departure_dropdown_open = false;
            }

            // --- SettingsPopup Events ---
            Event::ShowSettingsPopup => {
                self.state.show_settings_popup = true;
            }
            Event::CloseSettingsPopup => {
                self.state.show_settings_popup = false;
            }
            Event::SaveSettings => {
                if let Some(services) = &mut self.services {
                    if let Err(e) = services.app.set_api_key(&self.state.api_key) {
                        log::error!("Failed to save API key: {e}");
                    } else {
                        services.weather.update_api_key(self.state.api_key.clone());
                    }
                }
                self.state.show_settings_popup = false;
            }
        }
    }

    /// Handles a vector of events in sequence.
    ///
    /// This method iterates through a list of `Event`s and calls `handle_event`
    /// for each one, allowing for batch processing of UI events.
    ///
    /// # Arguments
    ///
    /// * `events` - A `Vec<Event>` to be processed.
    /// * `events` - A `Vec<Event>` to be processed.
    /// * `ctx` - The `egui::Context` for repainting.
    pub fn handle_events(&mut self, events: Vec<Event>, ctx: &egui::Context) {
        for event in events {
            self.handle_event(event, ctx);
        }
    }

    /// Central logic for processing a display mode change.
    fn process_display_mode_change(&mut self, mode: DisplayMode) {
        let services = match &mut self.services {
            Some(s) => s,
            None => return,
        };

        services.popup.set_display_mode(mode.clone());
        match mode {
            DisplayMode::RandomAirports => {
                let random_airports = services.app.get_random_airports(RANDOM_AIRPORTS_COUNT);
                let airport_items: Vec<_> = random_airports
                    .iter()
                    .map(|airport| services.app.create_list_item_for_airport(airport))
                    .collect();
                let table_items: Vec<Arc<TableItem>> = airport_items
                    .into_iter()
                    .map(|item| Arc::new(TableItem::Airport(item)))
                    .collect();
                self.set_all_items(table_items);
            }
            DisplayMode::Other => {
                // Currently "List all aircraft"
                let aircraft_items: Vec<_> = services
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
                services.search.clear_query();
                // Calculate and set the flight statistics
                let stats_result = services.app.get_flight_statistics();
                self.state.statistics = Some(stats_result);
            }
            _ => self.update_displayed_items(),
        }
    }

    /// Updates the list of items to be displayed in the main table based on the current `DisplayMode`.
    ///
    /// This function is called when the display mode changes or when the underlying
    /// data for the current mode needs to be refreshed. It populates `state.all_items`
    /// with the appropriate data from the `AppService`.
    pub fn update_displayed_items(&mut self) {
        let services = match &mut self.services {
            Some(s) => s,
            None => return,
        };

        self.state.all_items = match services.popup.display_mode() {
            DisplayMode::RandomRoutes
            | DisplayMode::NotFlownRoutes
            | DisplayMode::SpecificAircraftRoutes => services
                .app
                .route_items()
                .iter()
                .map(|r| Arc::new(TableItem::Route(r.clone())))
                .collect(),
            DisplayMode::History => services
                .app
                .history_items()
                .iter()
                .map(|h| Arc::new(TableItem::History(h.clone())))
                .collect(),
            DisplayMode::Airports | DisplayMode::RandomAirports => services
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

    /// Initiates an update of the route list.
    ///
    /// This method sets a request to update the routes, which is then handled
    /// by the background task spawner. It avoids stacking multiple requests if
    /// an update is already in progress.
    ///
    /// # Arguments
    ///
    /// * `action` - The `RouteUpdateAction` to perform (e.g., `Regenerate` or `Append`).
    pub fn update_routes(&mut self, action: RouteUpdateAction) {
        if self.state.is_loading_more_routes {
            return; // Don't stack requests
        }
        self.route_update_request = Some(action);
    }

    // --- Helper methods for state management ---

    /// Returns a slice of the items that should be currently displayed in the table.
    ///
    /// If there is an active search query, this returns the filtered items.
    /// Otherwise, it returns all items for the current view.
    ///
    /// # Returns
    ///
    /// A slice of `Arc<TableItem>` to be displayed.
    pub fn get_displayed_items(&self) -> &[Arc<TableItem>] {
        if let Some(services) = &self.services {
            if services.search.query().trim().is_empty() {
                &self.state.all_items
            } else {
                services.search.filtered_items()
            }
        } else {
            &self.state.all_items
        }
    }

    fn set_all_items(&mut self, items: Vec<Arc<TableItem>>) {
        self.state.all_items = items;
        self.update_filtered_items();
    }

    fn update_filtered_items(&mut self) {
        if let Some(services) = &mut self.services {
            let query = services.search.query();
            if query.trim().is_empty() {
                services.search.set_filtered_items(Vec::new());
            } else {
                let filtered_items =
                    SearchService::filter_items_static(&self.state.all_items, query);
                services.search.set_filtered_items(filtered_items);
            }
        }
    }

    fn is_route_mode(&self) -> bool {
        if let Some(services) = &self.services {
            matches!(
                services.popup.display_mode(),
                DisplayMode::RandomRoutes
                    | DisplayMode::NotFlownRoutes
                    | DisplayMode::SpecificAircraftRoutes
            )
        } else {
            false
        }
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
            if let Some(services) = &mut self.services {
                services.popup.set_display_mode(new_mode);
            }
        }
    }

    fn handle_route_mode_transition(&mut self) {
        if self.is_route_mode() {
            let new_mode = self.get_appropriate_route_mode();
            if let Some(services) = &mut self.services
                && new_mode != *services.popup.display_mode()
            {
                services.popup.set_display_mode(new_mode);
            }
        }
    }

    fn refresh_aircraft_items_if_needed(&mut self) {
        if let Some(services) = &self.services
            && matches!(services.popup.display_mode(), DisplayMode::Other)
        {
            let aircraft_items: Vec<_> = services
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

    /// Marks a given route as flown.
    ///
    /// This method delegates the action to the `AppService`, which handles
    /// adding the flight to history and updating the aircraft's flown status.
    ///
    /// # Arguments
    ///
    /// * `route` - The `ListItemRoute` to be marked as flown.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or an error if the operation fails.
    pub fn mark_route_as_flown(
        &mut self,
        route: &crate::gui::data::ListItemRoute,
    ) -> Result<(), Box<dyn Error>> {
        if let Some(services) = &mut self.services {
            services.app.mark_route_as_flown(route)
        } else {
            Ok(())
        }
    }

    /// Handles results from background tasks (route generation and search).
    fn handle_background_task_results(&mut self, ctx: &egui::Context) {
        // Check for results from the route generation thread
        match self.route_receiver.try_recv() {
            Ok(new_routes) => {
                // Extract ICAOs for background fetching before moving new_routes
                let icaos: HashSet<String> = new_routes
                    .iter()
                    .flat_map(|r| vec![r.departure.ICAO.clone(), r.destination.ICAO.clone()])
                    .collect();

                if let Some(RouteUpdateAction::Append) = self.route_update_request {
                    if let Some(services) = &mut self.services {
                        services.app.append_route_items(new_routes);
                    }
                } else if let Some(services) = &mut self.services {
                    services.app.set_route_items(new_routes);
                }
                self.update_displayed_items();
                self.state.is_loading_more_routes = false;
                self.route_update_request = None; // Clear the request

                // Spawn background fetch for all airports in the routes
                if !icaos.is_empty() {
                    // Fetch all METARs in parallel (caching happens automatically in the database)
                    if let Some(services) = &self.services {
                        let weather_service = services.weather.clone();
                        let ctx_clone = ctx.clone();
                        std::thread::spawn(move || {
                            let icaos_vec: Vec<String> = icaos.into_iter().collect();
                            icaos_vec.par_iter().for_each(|icao| {
                                let _ = weather_service.fetch_metar(icao);
                            });
                            ctx_clone.request_repaint();
                        });
                    }
                }
            }
            Err(mpsc::TryRecvError::Empty) => {
                // No message yet
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                log::error!("Route generation thread disconnected unexpectedly.");
                self.state.is_loading_more_routes = false;
                self.route_update_request = None;
            }
        }

        // Check for results from the search thread
        match self.search_receiver.try_recv() {
            Ok(filtered_items) => {
                if let Some(services) = &mut self.services {
                    services.search.set_filtered_items(filtered_items);
                }
            }
            Err(mpsc::TryRecvError::Empty) => {
                // No message yet
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                log::error!("Search thread disconnected unexpectedly.");
            }
        }

        // Check for results from the weather thread
        while let Ok((station, result)) = self.weather_receiver.try_recv() {
            if let Some(services) = &mut self.services
                && let Some(route) = services.popup.selected_route()
            {
                match result {
                    Ok(metar) => {
                        if route.departure.ICAO == station {
                            services.popup.set_departure_metar(Some(metar));
                        } else if route.destination.ICAO == station {
                            services.popup.set_destination_metar(Some(metar));
                        }
                    }
                    Err(e) => {
                        // Convert WeatherError to string for display
                        let error_msg = e.to_string();
                        log::error!("Failed to fetch weather for {}: {}", station, error_msg);
                        if route.departure.ICAO == station {
                            services.popup.set_departure_weather_error(Some(error_msg));
                        } else if route.destination.ICAO == station {
                            services
                                .popup
                                .set_destination_weather_error(Some(error_msg));
                        }
                    }
                }
            }
        }
    }

    /// Spawns background tasks when needed (route generation and search).
    fn spawn_background_tasks(&mut self, ctx: &egui::Context) {
        let services = match &mut self.services {
            Some(s) => s,
            None => return,
        };

        // Spawn route generation task if requested
        if self.route_update_request.is_some() && !self.state.is_loading_more_routes {
            self.state.is_loading_more_routes = true;
            let sender = self.route_sender.clone();
            let ctx_clone = ctx.clone();
            let departure_icao = services
                .app
                .get_selected_airport_icao(&self.state.selected_departure_airport);
            let display_mode = services.popup.display_mode().clone();
            let selected_aircraft = self.state.selected_aircraft.clone();

            services.app.spawn_route_generation_thread(
                display_mode,
                selected_aircraft,
                departure_icao,
                move |routes| {
                    send_and_repaint(&sender, routes, Some(ctx_clone));
                },
            );
        }

        // Spawn search task if needed
        if services.search.should_execute_search() {
            let sender = self.search_sender.clone();
            let ctx_clone = ctx.clone();
            let all_items = self.state.all_items.clone();

            services
                .search
                .spawn_search_thread(all_items, move |filtered_items| {
                    send_and_repaint(&sender, filtered_items, Some(ctx_clone));
                });
        }
    }
}

fn send_and_repaint<T: Send>(sender: &mpsc::Sender<T>, data: T, ctx: Option<egui::Context>) {
    if sender.send(data).is_ok()
        && let Some(ctx) = ctx
    {
        ctx.request_repaint();
    }
}

impl eframe::App for Gui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle startup
        if self.services.is_none() {
            if let Some(receiver) = &self.startup_receiver
                && let Ok(result) = receiver.try_recv()
            {
                match result {
                    Ok(services) => {
                        log::info!("Gui::update: Services received from background thread.");
                        self.services = Some(services);
                        self.startup_receiver = None; // Done loading

                        // Initialize data
                        if let Some(services) = &mut self.services {
                            // Trigger batch fetch for the initial routes
                            let icaos: HashSet<String> = self
                                .state
                                .all_items
                                .iter()
                                .filter_map(|item| {
                                    if let TableItem::Route(r) = &**item {
                                        Some(vec![
                                            r.departure.ICAO.clone(),
                                            r.destination.ICAO.clone(),
                                        ])
                                    } else {
                                        None
                                    }
                                })
                                .flatten()
                                .collect();

                            if !icaos.is_empty() {
                                let weather_service = services.weather.clone();
                                let ctx_clone = ctx.clone();
                                std::thread::spawn(move || {
                                    let icaos_vec: Vec<String> = icaos.into_iter().collect();
                                    icaos_vec.par_iter().for_each(|icao| {
                                        let _ = weather_service.fetch_metar(icao);
                                    });
                                    ctx_clone.request_repaint();
                                });
                            }
                        }
                        self.update_displayed_items();
                    }
                    Err(e) => {
                        self.startup_error = Some(e);
                        self.startup_receiver = None;
                    }
                }
            }

            // Show loading or error screen
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() / 2.0 - 50.0);
                    if let Some(error) = &self.startup_error {
                        ui.heading("❌ Failed to start application");
                        ui.add_space(10.0);
                        ui.label(error);
                    } else {
                        ui.spinner();
                        ui.add_space(10.0);
                        ui.heading("Loading application data...");
                    }
                });
            });
            return;
        }

        let mut events = Vec::new();

        // Handle results from background tasks
        self.handle_background_task_results(ctx);

        // Spawn new background tasks if needed
        self.spawn_background_tasks(ctx);

        // Handle route popup
        if let Some(services) = &self.services
            && services.popup.is_alert_visible()
        {
            let route_popup_vm = crate::gui::components::route_popup::RoutePopupViewModel {
                is_alert_visible: services.popup.is_alert_visible(),
                selected_route: services.popup.selected_route(),
                display_mode: services.popup.display_mode(),
                departure_metar: services.popup.departure_metar(),
                destination_metar: services.popup.destination_metar(),
                departure_weather_error: services.popup.departure_weather_error(),
                destination_weather_error: services.popup.destination_weather_error(),
            };
            events.extend(RoutePopup::render(&route_popup_vm, ctx));
        }

        // Handle "Add History" popup
        if self.state.add_history.show_popup
            && let Some(services) = &self.services
        {
            events.extend(AddHistoryPopup::render(
                services.app.aircraft(),
                services.app.airports(),
                &mut self.state.add_history,
                ctx,
            ));
        }

        // Handle "Settings" popup
        if self.state.show_settings_popup {
            let mut vm = SettingsPopupViewModel {
                api_key: &mut self.state.api_key,
            };
            events.extend(SettingsPopup::render(&mut vm, ctx));
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let main_ui_enabled = if let Some(services) = &self.services {
                !services.popup.is_alert_visible()
                    && !self.state.add_history.show_popup
                    && !self.state.show_settings_popup
            } else {
                false
            };
            ui.add_enabled_ui(main_ui_enabled, |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                    // --- Left Panel ---
                    ui.allocate_ui_with_layout(
                        egui::Vec2::new(250.0, ui.available_height()),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            if let Some(services) = &self.services {
                                let mut selection_vm = SelectionControlsViewModel {
                                    selected_departure_airport: &self
                                        .state
                                        .selected_departure_airport,
                                    selected_aircraft: &self.state.selected_aircraft,
                                    departure_dropdown_open: &mut self
                                        .state
                                        .departure_dropdown_open,
                                    aircraft_dropdown_open: &mut self.state.aircraft_dropdown_open,
                                    departure_airport_search: &mut self.state.departure_search,
                                    aircraft_search: &mut self.state.aircraft_search,
                                    departure_display_count: &mut self
                                        .state
                                        .departure_display_count,
                                    aircraft_display_count: &mut self.state.aircraft_display_count,
                                    available_airports: services.app.airports(),
                                    all_aircraft: services.app.aircraft(),
                                };
                                events.extend(SelectionControls::render(&mut selection_vm, ui));
                            }
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
                        ui.horizontal(|ui| {
                            if let Some(services) = &mut self.services {
                                if services.popup.display_mode() == &DisplayMode::History
                                    && ui.button("Add to History").clicked()
                                {
                                    events.push(Event::ShowAddHistoryPopup);
                                }
                                if ui.button("Settings").clicked() {
                                    events.push(Event::ShowSettingsPopup);
                                }
                                let mut search_vm = SearchControlsViewModel {
                                    query: services.search.query_mut(),
                                };
                                events.extend(SearchControls::render(&mut search_vm, ui));
                            }
                        });

                        ui.add_space(10.0);
                        ui.separator();

                        if let Some(services) = &self.services {
                            let weather_service = &services.weather;
                            let table_vm = TableDisplayViewModel {
                                items: self.get_displayed_items(),
                                display_mode: services.popup.display_mode(),
                                is_loading_more_routes: self.state.is_loading_more_routes,
                                statistics: &self.state.statistics,
                                flight_rules_lookup: Some(&|icao| {
                                    weather_service.get_cached_flight_rules(icao)
                                }),
                            };
                            events.extend(TableDisplay::render(&table_vm, ui));
                        }
                    });
                });
            });
        });

        self.handle_events(events, ctx);
    }
}
