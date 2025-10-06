use crate::database::DatabasePool;
use crate::gui::components::{
    action_buttons::{ActionButtons, ActionButtonsViewModel},
    add_history_popup::AddHistoryPopup,
    route_popup::RoutePopup,
    search_controls::{SearchControls, SearchControlsViewModel},
    selection_controls::{SelectionControls, SelectionControlsViewModel},
    table_display::{TableDisplay, TableDisplayViewModel},
    weather_controls::{WeatherControls, WeatherControlsViewModel},
};
use crate::gui::data::{ListItemAircraft, ListItemRoute, TableItem};
use crate::gui::events::Event;
use crate::gui::services::route_popup_service::WeatherState;
use crate::gui::services::view_mode_service::DisplayMode;
use crate::gui::services::{AppService, SearchService, Services};
use crate::modules::weather;
use reqwest::Client;
use tokio::runtime::Runtime;
use crate::gui::state::{AddHistoryState, ApplicationState};
use eframe::egui::{self};
use log;
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
    pub services: Services,
    /// The sender part of a channel for sending route generation results from a background thread.
    pub route_sender: mpsc::Sender<Vec<ListItemRoute>>,
    /// The receiver part of a channel for receiving route generation results.
    pub route_receiver: mpsc::Receiver<Vec<ListItemRoute>>,
    /// The sender part of a channel for sending search results from a background thread.
    pub search_sender: mpsc::Sender<Vec<Arc<TableItem>>>,
    /// The receiver part of a channel for receiving search results.
    pub search_receiver: mpsc::Receiver<Vec<Arc<TableItem>>>,
    /// The sender part of a channel for sending weather data results.
    pub weather_sender: mpsc::Sender<(String, WeatherState)>,
    /// The receiver part of a channel for receiving weather data results.
    pub weather_receiver: mpsc::Receiver<(String, WeatherState)>,
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
        let (weather_sender, weather_receiver) = mpsc::channel();

        Ok(Gui {
            services,
            state,
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
                let route_arc = Arc::new(route);
                self.services
                    .route_popup
                    .set_selected_route(Some(route_arc.clone()));
                self.spawn_weather_fetch_thread(
                    &route_arc.departure.ICAO,
                    &route_arc.destination.ICAO,
                );
            }
            Event::SetShowPopup(show) => self.services.route_popup.set_visibility(show),
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
                // The query has already been updated via the mutable reference in the view model
                let query = self.services.search.query();

                // For very short queries (1-2 characters), search instantly
                // For longer queries, use debouncing to avoid excessive searches
                if query.len() <= INSTANT_SEARCH_MIN_QUERY_LEN {
                    self.services.search.force_search_pending();
                } else {
                    // Standard debouncing for longer queries
                    self.services.search.set_search_pending(true);
                    self.services
                        .search
                        .set_last_search_request(Some(std::time::Instant::now()));
                }
            }
            Event::ClearSearch => {
                // The mutable borrow in the VM has already cleared the text.
                // Now we need to clear the search service state as well.
                self.services.search.clear_query();
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
                self.services.route_popup.set_visibility(false);
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
                if let Err(e) =
                    self.services
                        .app
                        .add_history_entry(&aircraft, &departure, &destination)
                {
                    log::error!("Failed to add history entry: {e}");
                } else {
                    // Refresh history view if it's active
                    if self.services.view_mode.display_mode() == &DisplayMode::History {
                        self.update_displayed_items();
                    }
                    // Close the popup
                    self.handle_event(Event::CloseAddHistoryPopup);
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
    pub fn handle_events(&mut self, events: Vec<Event>) {
        for event in events {
            self.handle_event(event);
        }
    }

    /// Central logic for processing a display mode change.
    fn process_display_mode_change(&mut self, mode: DisplayMode) {
        self.services.view_mode.set_display_mode(mode.clone());
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

    /// Updates the list of items to be displayed in the main table based on the current `DisplayMode`.
    ///
    /// This function is called when the display mode changes or when the underlying
    /// data for the current mode needs to be refreshed. It populates `state.all_items`
    /// with the appropriate data from the `AppService`.
    pub fn update_displayed_items(&mut self) {
        self.state.all_items = match self.services.view_mode.display_mode() {
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
            self.services.view_mode.display_mode(),
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
            self.services.view_mode.set_display_mode(new_mode);
        }
    }

    fn handle_route_mode_transition(&mut self) {
        if self.is_route_mode() {
            let new_mode = self.get_appropriate_route_mode();
            if new_mode != *self.services.view_mode.display_mode() {
                self.services.view_mode.set_display_mode(new_mode);
            }
        }
    }

    fn refresh_aircraft_items_if_needed(&mut self) {
        if matches!(self.services.view_mode.display_mode(), DisplayMode::Other) {
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
        self.services.app.mark_route_as_flown(route)
    }

    /// Handles results from background tasks (route generation and search).
    fn handle_background_task_results(&mut self) {
        // Check for results from the route generation thread
        match self.route_receiver.try_recv() {
            Ok(new_routes) => {
                if let Some(RouteUpdateAction::Append) = self.route_update_request {
                    self.services.app.append_route_items(new_routes);
                } else {
                    self.services.app.set_route_items(new_routes);
                }
                self.update_displayed_items();
                self.state.is_loading_more_routes = false;
                self.route_update_request = None; // Clear the request
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
                self.services.search.set_filtered_items(filtered_items);
            }
            Err(mpsc::TryRecvError::Empty) => {
                // No message yet
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                log::error!("Search thread disconnected unexpectedly.");
            }
        }

        // Check for results from the weather fetch thread
        match self.weather_receiver.try_recv() {
            Ok((icao, weather_state)) => {
                if let Some(route) = self.services.route_popup.selected_route() {
                    if route.departure.ICAO == icao {
                        self.services.route_popup.state_mut().departure_weather = weather_state;
                    } else if route.destination.ICAO == icao {
                        self.services.route_popup.state_mut().destination_weather = weather_state;
                    }
                }
            }
            Err(mpsc::TryRecvError::Empty) => {
                // No message yet
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                log::error!("Weather fetch thread disconnected unexpectedly.");
            }
        }
    }

    /// Spawns background tasks when needed (route generation and search).
    fn spawn_background_tasks(&mut self, ctx: &egui::Context) {
        // Spawn route generation task if requested
        if self.route_update_request.is_some() && !self.state.is_loading_more_routes {
            self.state.is_loading_more_routes = true;
            let sender = self.route_sender.clone();
            let ctx_clone = ctx.clone();
            let departure_icao = self
                .services
                .app
                .get_selected_airport_icao(&self.state.selected_departure_airport);
            let display_mode = self.services.view_mode.display_mode().clone();
            let selected_aircraft = self.state.selected_aircraft.clone();
            let weather_filter = self.state.weather_filter.clone();

            self.services.app.spawn_route_generation_thread(
                display_mode,
                selected_aircraft,
                departure_icao,
                weather_filter,
                move |routes| {
                    send_and_repaint(&sender, routes, Some(ctx_clone));
                },
            );
        }

        // Spawn search task if needed
        if self.services.search.should_execute_search() {
            let sender = self.search_sender.clone();
            let ctx_clone = ctx.clone();
            let all_items = self.state.all_items.clone();

            self.services
                .search
                .spawn_search_thread(all_items, move |filtered_items| {
                    send_and_repaint(&sender, filtered_items, Some(ctx_clone));
                });
        }
    }

    /// Spawns a background thread to fetch weather data for the given airports.
    ///
    /// This method first checks the cache for existing, valid weather data. If
    /// data is not found or is expired, it spawns a thread to fetch it from the
    /// AVWX API.
    fn spawn_weather_fetch_thread(&mut self, departure_icao: &str, destination_icao: &str) {
        let dep_icao = departure_icao.to_string();
        let dest_icao = destination_icao.to_string();

        // Check cache for departure weather
        if let Some(cached_weather) = self.services.route_popup.get_cached_weather(&dep_icao) {
            self.services.route_popup.state_mut().departure_weather =
                WeatherState::Success(cached_weather.clone());
        } else {
            // Fetch departure weather
            self.fetch_weather_for_icao(dep_icao);
        }

        // Check cache for destination weather
        if let Some(cached_weather) = self.services.route_popup.get_cached_weather(&dest_icao) {
            self.services.route_popup.state_mut().destination_weather =
                WeatherState::Success(cached_weather.clone());
        } else {
            // Fetch destination weather
            self.fetch_weather_for_icao(dest_icao);
        }
    }

    /// Helper to spawn a thread for fetching weather for a single ICAO.
    fn fetch_weather_for_icao(&self, icao: String) {
        let sender = self.weather_sender.clone();
        std::thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                let api_key = match std::env::var("AVWX_API_KEY") {
                    Ok(key) => key,
                    Err(_) => {
                        log::error!("AVWX_API_KEY not set");
                        let _ = sender.send((
                            icao,
                            WeatherState::Error("API Key not set".to_string()),
                        ));
                        return;
                    }
                };
                let client = Client::new();
                let result =
                    weather::get_weather_data(weather::AVWX_API_URL, &icao, &client, &api_key).await;

                let state = match result {
                    Ok(metar) => WeatherState::Success(metar),
                    Err(e) => WeatherState::Error(e.to_string()),
                };

                let _ = sender.send((icao, state));
            });
        });
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
        let mut events = Vec::new();

        // Handle results from background tasks
        self.handle_background_task_results();

        // Spawn new background tasks if needed
        self.spawn_background_tasks(ctx);

        // Handle route popup
        if self.services.route_popup.is_visible() {
            let route_popup_vm = crate::gui::components::route_popup::RoutePopupViewModel {
                popup_state: self.services.route_popup.state(),
                display_mode: self.services.view_mode.display_mode(),
            };
            events.extend(RoutePopup::render(&route_popup_vm, ctx));
        }

        // Handle "Add History" popup
        if self.state.add_history.show_popup {
            events.extend(AddHistoryPopup::render(
                self.services.app.aircraft(),
                self.services.app.airports(),
                &mut self.state.add_history,
                ctx,
            ));
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let main_ui_enabled =
                !self.services.route_popup.is_visible() && !self.state.add_history.show_popup;
            ui.add_enabled_ui(main_ui_enabled, |ui| {
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

                            ui.separator();
                            let mut weather_vm = WeatherControlsViewModel {
                                weather_filter: &mut self.state.weather_filter,
                            };
                            if WeatherControls::render(&mut weather_vm, ui) {
                                events.push(Event::RegenerateRoutesForSelectionChange);
                            }
                        },
                    );

                    ui.separator();

                    // --- Right Panel ---
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            if self.services.view_mode.display_mode() == &DisplayMode::History
                                && ui.button("Add to History").clicked()
                            {
                                events.push(Event::ShowAddHistoryPopup);
                            }
                            let mut search_vm = SearchControlsViewModel {
                                query: self.services.search.query_mut(),
                            };
                            events.extend(SearchControls::render(&mut search_vm, ui));
                        });

                        ui.add_space(10.0);
                        ui.separator();

                        let table_vm = TableDisplayViewModel {
                            items: self.get_displayed_items(),
                            display_mode: self.services.view_mode.display_mode(),
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
