#[cfg(not(target_arch = "wasm32"))]
use crate::database::DatabasePool;
use crate::gui::components::common::IconButton;
use crate::gui::components::{
    action_buttons::{ActionButtons, ActionButtonsViewModel},
    add_history_popup::AddHistoryPopup,
    route_popup::RoutePopup,
    search_controls::{SearchControls, SearchControlsViewModel},
    selection_controls::{SelectionControls, SelectionControlsViewModel},
    settings_popup::{SettingsPopup, SettingsPopupViewModel},
    table_display::{TableDisplay, TableDisplayViewModel},
    toast::ToastKind,
};
use crate::gui::data::{ListItemAircraft, ListItemRoute, TableItem};
use crate::gui::events::{AppEvent, DataEvent, SelectionEvent, UiEvent};
use crate::gui::icons;
use crate::gui::services::SearchService;
use crate::gui::services::popup_service::DisplayMode;
#[cfg(not(target_arch = "wasm32"))]
use crate::gui::services::{AppService, Services};
use crate::gui::state::{AddHistoryState, ApplicationState};
use crate::models::weather::{Metar, WeatherError};
use eframe::egui::{self};
use log;
#[cfg(all(feature = "gui", not(target_arch = "wasm32")))]
use rayon::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use std::collections::HashSet;
use std::error::Error;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, mpsc};

#[cfg(not(target_arch = "wasm32"))]
type GServices = Services;
#[cfg(target_arch = "wasm32")]
type GServices = crate::web::services::WebServices;

/// WASM-only async channels and UI state that would be unused on native targets.
///
/// Collected here to avoid scattering `#[cfg(target_arch = "wasm32")]` attributes
/// across every field of `Gui`.
#[cfg(target_arch = "wasm32")]
pub struct WasmState {
    pub airport_search_sender: mpsc::Sender<(u64, Vec<crate::models::Airport>)>,
    pub airport_search_receiver: mpsc::Receiver<(u64, Vec<crate::models::Airport>)>,
    pub route_popup_sender: mpsc::Sender<ListItemRoute>,
    pub route_popup_receiver: mpsc::Receiver<ListItemRoute>,
    pub statistics_sender: mpsc::Sender<crate::models::FlightStatistics>,
    pub statistics_receiver: mpsc::Receiver<crate::models::FlightStatistics>,
    pub toast_sender: mpsc::Sender<(String, ToastKind)>,
    pub toast_receiver: mpsc::Receiver<(String, ToastKind)>,
    /// Delivers paginated history pages from the server.
    pub history_page_sender: mpsc::Sender<crate::models::HistoryPageResponse>,
    pub history_page_receiver: mpsc::Receiver<crate::models::HistoryPageResponse>,
    /// Delivers paginated airport-browse pages — `(airports, has_more, is_append)`.
    pub airports_page_sender: mpsc::Sender<(Vec<crate::models::Airport>, bool, bool)>,
    pub airports_page_receiver: mpsc::Receiver<(Vec<crate::models::Airport>, bool, bool)>,
    pub is_loading_more_history: bool,
    pub is_loading_more_airports: bool,
    /// Last observed search texts for each airport-driven dropdown (debounce tracking).
    pub last_main_departure_search: String,
    pub last_add_history_departure_search: String,
    pub last_add_history_destination_search: String,
    pub airport_search_debounce_at: Option<web_time::Instant>,
    pub pending_airport_query: Option<String>,
}

// UI Constants
const RANDOM_AIRPORTS_COUNT: usize = 50;
/// Query length threshold for instant search without debouncing
const INSTANT_SEARCH_MIN_QUERY_LEN: usize = 2;
const ADD_TO_HISTORY_TOOLTIP: &str = "Manually add a flown route to your history";
const SETTINGS_TOOLTIP: &str = "Configure API keys and preferences";

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
    pub services: Option<GServices>,
    /// Receiver for the initialized services from the background thread.
    pub startup_receiver: Option<mpsc::Receiver<Result<GServices, String>>>,
    /// Error message if initialization fails.
    pub startup_error: Option<String>,
    /// The sender part of a channel for sending route generation results from a background thread.
    pub route_sender: mpsc::Sender<(u64, RouteUpdateAction, Vec<ListItemRoute>)>,
    /// The receiver part of a channel for receiving route generation results.
    pub route_receiver: mpsc::Receiver<(u64, RouteUpdateAction, Vec<ListItemRoute>)>,
    /// The sender part of a channel for sending search results from a background thread.
    pub search_sender: mpsc::Sender<Vec<Arc<TableItem>>>,
    /// The receiver part of a channel for receiving search results.
    pub search_receiver: mpsc::Receiver<Vec<Arc<TableItem>>>,
    /// The sender part of a channel for sending weather results.
    pub weather_sender: mpsc::Sender<(String, Result<Metar, WeatherError>)>,
    /// The receiver part of a channel for receiving weather results.
    pub weather_receiver: mpsc::Receiver<(String, Result<Metar, WeatherError>)>,
    /// Receiver for airport item generation results.
    pub airport_items_receiver: mpsc::Receiver<Vec<Arc<TableItem>>>,
    /// Sender for airport item generation results.
    pub airport_items_sender: mpsc::Sender<Vec<Arc<TableItem>>>,
    /// Stores a pending request to update routes, used to trigger background generation.
    pub route_update_request: Option<RouteUpdateAction>,
    /// Flag to indicate if airport items are currently loading
    pub is_loading_airport_items: bool,
    /// The current route generation ID, used to invalidate old requests and cancel background tasks.
    pub current_route_generation_id: Arc<AtomicU64>,
    /// A flag indicating whether the main table should scroll to the top on the next frame.
    pub scroll_to_top: bool,
    /// WASM-only async channels and debounce state.
    /// All WASM-specific fields live here so the `Gui` struct stays clean.
    #[cfg(target_arch = "wasm32")]
    pub wasm: WasmState,
}

impl Gui {
    /// Creates a new `Gui` instance.
    ///
    /// This constructor initializes the GUI state and spawns a background thread
    /// to perform heavy initialization tasks (database setup, migrations, data loading).
    ///
    /// # Arguments
    ///
    /// * `cc` - The eframe creation context.
    /// * `database_pool` - Optional database pool. If `None`, a new pool will be created.
    ///   For testing, pass `Some(pool)` to use an in-memory database.
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `Gui` instance on success, or an error if
    /// initialization fails.
    #[cfg(not(target_arch = "wasm32"))]
    #[cfg(not(tarpaulin_include))]
    pub fn new(
        cc: &eframe::CreationContext,
        database_pool: Option<DatabasePool>,
    ) -> Result<Self, Box<dyn Error>> {
        log::info!("Gui::new: Called.");
        // Create channels for background tasks
        let (route_sender, route_receiver) = mpsc::channel();
        let (search_sender, search_receiver) = mpsc::channel();
        let (weather_sender, weather_receiver) = mpsc::channel();
        let (startup_sender, startup_receiver) = mpsc::channel();
        let (airport_items_sender, airport_items_receiver) = mpsc::channel();

        let ctx = cc.egui_ctx.clone();

        // Spawn background initialization
        std::thread::spawn(move || {
            log::info!("Gui::new: Background thread started.");
            let start = web_time::Instant::now();
            let result = (|| -> Result<Services, String> {
                // Initialize DatabasePool (or use provided one for tests)
                let database_pool = match database_pool {
                    Some(pool) => pool,
                    None => DatabasePool::new(None, None).map_err(|e| e.to_string())?,
                };

                // Run migrations
                crate::run_database_migrations(&database_pool).map_err(|e| e.to_string())?;

                // Apply database optimizations (indexes)
                crate::database::apply_database_optimizations(&database_pool)
                    .map_err(|e| e.to_string())?;

                // Import aircraft CSV if empty (only for production, not tests)
                crate::import_aircraft_csv_if_empty(&database_pool);

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
            airport_items_sender,
            airport_items_receiver,
            route_update_request: None,
            is_loading_airport_items: false,
            current_route_generation_id: Arc::new(AtomicU64::new(0)),
            scroll_to_top: false,
        })
    }

    /// Creates a new `Gui` instance for the WASM web target.
    ///
    /// Fetches the small startup data (aircraft, history, settings, initial
    /// random routes/statistics, plus a small random sample of airports) and
    /// builds the `WebServices`. The global airport list is never downloaded;
    /// dropdowns drive `/api/airports/search` queries as the user types.
    #[cfg(target_arch = "wasm32")]
    pub fn new_web(cc: &eframe::CreationContext) -> Result<Self, Box<dyn Error>> {
        Self::spawn_web_startup(cc.egui_ctx.clone())
    }

    /// Builds the channels for a WASM session and spawns the startup task.
    /// Factored out so the error screen can re-run startup on retry.
    #[cfg(target_arch = "wasm32")]
    fn spawn_web_startup(ctx: egui::Context) -> Result<Self, Box<dyn Error>> {
        let (route_sender, route_receiver) = mpsc::channel();
        let (search_sender, search_receiver) = mpsc::channel();
        let (weather_sender, weather_receiver) = mpsc::channel();
        let (startup_sender, startup_receiver) = mpsc::channel();
        let (airport_items_sender, airport_items_receiver) = mpsc::channel();
        let (airport_search_sender, airport_search_receiver) = mpsc::channel();
        let (route_popup_sender, route_popup_receiver) = mpsc::channel();
        let (statistics_sender, statistics_receiver) = mpsc::channel();
        let (toast_sender, toast_receiver) = mpsc::channel();
        let (history_page_sender, history_page_receiver) = mpsc::channel();
        let (airports_page_sender, airports_page_receiver): (
            mpsc::Sender<(Vec<crate::models::Airport>, bool, bool)>,
            mpsc::Receiver<(Vec<crate::models::Airport>, bool, bool)>,
        ) = mpsc::channel();

        let startup_toast = toast_sender.clone();
        let ctx_bg = ctx.clone();

        wasm_bindgen_futures::spawn_local(async move {
            let result = (|| async {
                let api_client = crate::web::api_client::ApiClient::new();

                let aircraft = api_client
                    .fetch_aircraft()
                    .await
                    .map_err(|e| e.to_string())?;
                // Fetch only the first 50 history items; user can load more on demand.
                let history_page = api_client
                    .fetch_history(50, 0)
                    .await
                    .map_err(|e| e.to_string())?;
                let settings = api_client
                    .fetch_settings()
                    .await
                    .map_err(|e| e.to_string())?;
                let initial_routes = api_client
                    .generate_routes("random", None, None)
                    .await
                    .unwrap_or_default();
                let initial_statistics = api_client.fetch_statistics().await.ok();
                let initial_airports = api_client.random_airports(50).await.unwrap_or_default();

                let weather_service =
                    crate::web::weather_service::WebWeatherService::new(api_client.clone());

                let mut app = crate::web::app_service::WebAppService::new(
                    aircraft,
                    history_page,
                    settings,
                    api_client,
                    initial_routes,
                    initial_statistics,
                    initial_airports,
                );
                app.set_toast_sender(startup_toast);

                Ok::<crate::web::services::WebServices, String>(
                    crate::web::services::WebServices::new(app, weather_service),
                )
            })()
            .await;

            let _ = startup_sender.send(result);
            ctx_bg.request_repaint();
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
            airport_items_sender,
            airport_items_receiver,
            route_update_request: None,
            is_loading_airport_items: false,
            current_route_generation_id: Arc::new(AtomicU64::new(0)),
            scroll_to_top: false,
            wasm: WasmState {
                airport_search_sender,
                airport_search_receiver,
                route_popup_sender,
                route_popup_receiver,
                statistics_sender,
                statistics_receiver,
                toast_sender,
                toast_receiver,
                history_page_sender,
                history_page_receiver,
                airports_page_sender,
                airports_page_receiver,
                is_loading_more_history: false,
                is_loading_more_airports: false,
                last_main_departure_search: String::new(),
                last_add_history_departure_search: String::new(),
                last_add_history_destination_search: String::new(),
                airport_search_debounce_at: None,
                pending_airport_query: None,
            },
        })
    }

    /// Handles a single UI event, updating the state accordingly.
    #[cfg(not(tarpaulin_include))]
    fn handle_event(&mut self, event: AppEvent, ctx: &egui::Context) {
        match event {
            AppEvent::Selection(e) => match e {
                SelectionEvent::DepartureAirportSelected(airport) => {
                    self.maybe_switch_to_route_mode(airport.is_some());
                    self.state.selected_departure_airport = airport;
                    self.state.departure_dropdown_open = false;
                    if self.state.selected_departure_airport.is_some() {
                        self.state.departure_search.clear();
                    }
                }
                SelectionEvent::AircraftSelected(aircraft) => {
                    let aircraft_being_selected = aircraft.is_some();
                    self.state.selected_aircraft = aircraft;
                    self.maybe_switch_to_route_mode(aircraft_being_selected);
                    self.handle_route_mode_transition();
                    self.state.aircraft_dropdown_open = false;
                    if self.state.selected_aircraft.is_some() {
                        self.state.aircraft_search.clear();
                    }
                }
                SelectionEvent::ToggleDepartureAirportDropdown => {
                    self.state.departure_dropdown_open = !self.state.departure_dropdown_open;
                    self.state.departure_search_autofocus = self.state.departure_dropdown_open;
                    if self.state.departure_dropdown_open {
                        self.state.aircraft_dropdown_open = false;
                    }
                }
                SelectionEvent::ToggleAircraftDropdown => {
                    self.state.aircraft_dropdown_open = !self.state.aircraft_dropdown_open;
                    self.state.aircraft_search_autofocus = self.state.aircraft_dropdown_open;
                    if self.state.aircraft_dropdown_open {
                        self.state.departure_dropdown_open = false;
                    }
                }
                SelectionEvent::ToggleAddHistoryAircraftDropdown => {
                    self.state.add_history.aircraft_dropdown_open =
                        !self.state.add_history.aircraft_dropdown_open;
                    self.state.add_history.aircraft_search_autofocus =
                        self.state.add_history.aircraft_dropdown_open;
                    self.state.add_history.departure_dropdown_open = false;
                    self.state.add_history.destination_dropdown_open = false;
                }
                SelectionEvent::ToggleAddHistoryDepartureDropdown => {
                    self.state.add_history.departure_dropdown_open =
                        !self.state.add_history.departure_dropdown_open;
                    self.state.add_history.departure_search_autofocus =
                        self.state.add_history.departure_dropdown_open;
                    self.state.add_history.aircraft_dropdown_open = false;
                    self.state.add_history.destination_dropdown_open = false;
                }
                SelectionEvent::ToggleAddHistoryDestinationDropdown => {
                    self.state.add_history.destination_dropdown_open =
                        !self.state.add_history.destination_dropdown_open;
                    self.state.add_history.destination_search_autofocus =
                        self.state.add_history.destination_dropdown_open;
                    self.state.add_history.aircraft_dropdown_open = false;
                    self.state.add_history.departure_dropdown_open = false;
                }
                SelectionEvent::ClearAllSelections => {
                    self.state.selected_departure_airport = None;
                    self.state.selected_aircraft = None;
                    self.state.departure_search.clear();
                    self.state.aircraft_search.clear();
                    self.state.departure_dropdown_open = false;
                    self.state.aircraft_dropdown_open = false;
                    self.regenerate_routes_for_selection_change();
                    self.scroll_to_top = true;
                }
            },
            AppEvent::Ui(e) => match e {
                UiEvent::SetDisplayMode(mode) => {
                    #[cfg(target_arch = "wasm32")]
                    if matches!(mode, DisplayMode::Statistics)
                        && let Some(services) = self.services.as_ref()
                    {
                        let sender = self.wasm.statistics_sender.clone();
                        let ctx_clone = ctx.clone();
                        services.app.refresh_statistics(move |result| match result {
                            Ok(stats) => {
                                let _ = sender.send(stats);
                                ctx_clone.request_repaint();
                            }
                            Err(e) => log::error!("Statistics refresh failed: {e}"),
                        });
                    }
                    #[cfg(target_arch = "wasm32")]
                    if matches!(mode, DisplayMode::RandomAirports)
                        && let Some(services) = self.services.as_mut()
                    {
                        let generation = services.app.begin_airport_search();
                        let sender = self.wasm.airport_search_sender.clone();
                        let ctx_clone = ctx.clone();
                        services.app.spawn_airport_search(
                            generation,
                            String::new(),
                            RANDOM_AIRPORTS_COUNT,
                            move |g, airports| {
                                let _ = sender.send((g, airports));
                                ctx_clone.request_repaint();
                            },
                        );
                    }
                    // Airports browse view uses paginated endpoint, not random.
                    #[cfg(target_arch = "wasm32")]
                    if matches!(mode, DisplayMode::Airports)
                        && let Some(services) = self.services.as_mut()
                    {
                        self.wasm.is_loading_more_airports = false;
                        let sender = self.wasm.airports_page_sender.clone();
                        let ctx_clone = ctx.clone();
                        services
                            .app
                            .spawn_airport_browse_page(200, move |airports, has_more| {
                                // is_append=false: this is a fresh page-1 load
                                let _ = sender.send((airports, has_more, false));
                                ctx_clone.request_repaint();
                            });
                    }
                    self.process_display_mode_change(mode);
                }
                UiEvent::SearchQueryChanged => {
                    if let Some(services) = &mut self.services {
                        let query = services.search.query();
                        if query.len() <= INSTANT_SEARCH_MIN_QUERY_LEN {
                            services.search.force_search_pending();
                        } else {
                            services.search.set_search_pending(true);
                            services
                                .search
                                .set_last_search_request(Some(web_time::Instant::now()));
                        }
                    }
                }
                UiEvent::ClearSearch => {
                    if let Some(services) = &mut self.services {
                        services.search.clear_query();
                    }
                }
                UiEvent::ColumnResized {
                    mode,
                    index,
                    delta,
                    total_width,
                } => {
                    TableDisplay::handle_column_resize(
                        &mut self.state.column_widths,
                        mode,
                        index,
                        delta,
                        total_width,
                    );
                }
                UiEvent::ScrollTableToTop => self.scroll_to_top = true,
                UiEvent::SetShowPopup(show) => {
                    if let Some(services) = &mut self.services {
                        services.popup.set_alert_visibility(show)
                    }
                }
                UiEvent::ClosePopup => {
                    if let Some(services) = &mut self.services {
                        services.popup.set_alert_visibility(false);
                    }
                }
                UiEvent::ShowAddHistoryPopup => {
                    self.state.add_history.show_popup = true;
                }
                UiEvent::CloseAddHistoryPopup => {
                    self.state.add_history = AddHistoryState::new();
                }
                UiEvent::ShowSettingsPopup => {
                    self.state.show_settings_popup = true;
                }
                UiEvent::CloseSettingsPopup => {
                    self.state.show_settings_popup = false;
                }
                UiEvent::ShowToast(message, kind) => {
                    self.state.toast_manager.add(message, kind);
                }
            },
            AppEvent::Data(e) => match e {
                DataEvent::RegenerateRoutesForSelectionChange => {
                    self.regenerate_routes_for_selection_change()
                }
                DataEvent::RouteSelectedForPopup(route) => {
                    if let Some(services) = &mut self.services {
                        services.popup.set_selected_route(Some(route.clone()));
                        services.popup.show_alert();

                        let departure = route.departure.ICAO.clone();
                        let destination = route.destination.ICAO.clone();

                        #[cfg(not(target_arch = "wasm32"))]
                        {
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

                        #[cfg(target_arch = "wasm32")]
                        {
                            let weather_service = services.weather.clone();
                            let sender = self.weather_sender.clone();
                            let ctx_clone = ctx.clone();
                            let dep_key = departure.clone();
                            weather_service.fetch_metar_async(&dep_key, move |res| {
                                send_and_repaint(&sender, (departure, res), Some(ctx_clone));
                            });

                            let weather_service = services.weather.clone();
                            let sender = self.weather_sender.clone();
                            let ctx_clone = ctx.clone();
                            let dst_key = destination.clone();
                            weather_service.fetch_metar_async(&dst_key, move |res| {
                                send_and_repaint(&sender, (destination, res), Some(ctx_clone));
                            });
                        }
                    }
                }
                DataEvent::HistoryItemSelected(history) => {
                    #[cfg(not(target_arch = "wasm32"))]
                    if let Some(route) = self
                        .services
                        .as_ref()
                        .and_then(|s| s.app.get_route_from_history(&history))
                    {
                        self.handle_event(
                            AppEvent::Data(DataEvent::RouteSelectedForPopup(route)),
                            ctx,
                        );
                    }
                    #[cfg(target_arch = "wasm32")]
                    if let Some(services) = self.services.as_ref() {
                        let sender = self.wasm.route_popup_sender.clone();
                        let ctx_clone = ctx.clone();
                        services
                            .app
                            .spawn_route_from_history(&history, move |route| {
                                if let Some(r) = route {
                                    let _ = sender.send(r);
                                    ctx_clone.request_repaint();
                                }
                            });
                    }
                }
                DataEvent::ToggleAircraftFlownStatus(aircraft_id) => {
                    if let Some(services) = &mut self.services {
                        if let Err(e) = services.app.toggle_aircraft_flown_status(aircraft_id) {
                            log::error!("Failed to toggle aircraft flown status: {e}");
                        } else {
                            self.refresh_aircraft_items_if_needed();
                        }
                    }
                }
                DataEvent::MarkAllAircraftAsNotFlown => {
                    if let Some(services) = &mut self.services {
                        if let Err(e) = services.app.mark_all_aircraft_as_not_flown() {
                            log::error!("Failed to mark all aircraft as not flown: {e}");
                        } else {
                            self.refresh_aircraft_items_if_needed();
                        }
                    }
                }
                DataEvent::LoadMoreRoutes => self.load_more_routes_if_needed(),
                DataEvent::LoadMoreHistory => {
                    #[cfg(target_arch = "wasm32")]
                    self.load_more_history(ctx);
                }
                DataEvent::LoadMoreAirports => {
                    #[cfg(target_arch = "wasm32")]
                    self.load_more_airports(ctx);
                }
                DataEvent::MarkRouteAsFlown(route) => {
                    if let Err(e) = self.mark_route_as_flown(&route) {
                        log::error!("Failed to mark route as flown: {e}");
                        self.handle_event(
                            AppEvent::Ui(UiEvent::ShowToast(
                                "Failed to mark route as flown".to_string(),
                                ToastKind::Error,
                            )),
                            ctx,
                        );
                    } else {
                        self.regenerate_routes_for_selection_change();
                        if let Some(services) = &mut self.services {
                            services.popup.mark_selected_route_as_flown();
                        }
                        self.handle_event(
                            AppEvent::Ui(UiEvent::ShowToast(
                                "Route marked as flown".to_string(),
                                ToastKind::Success,
                            )),
                            ctx,
                        );
                    }
                }
                DataEvent::AddHistoryEntry {
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
                            self.handle_event(
                                AppEvent::Ui(UiEvent::ShowToast(
                                    format!("Failed to add history entry: {}", e),
                                    ToastKind::Error,
                                )),
                                ctx,
                            );
                        } else {
                            if services.popup.display_mode() == &DisplayMode::History {
                                self.update_displayed_items();
                            }
                            self.handle_event(AppEvent::Ui(UiEvent::CloseAddHistoryPopup), ctx);
                            self.handle_event(
                                AppEvent::Ui(UiEvent::ShowToast(
                                    "Flight added to history".to_string(),
                                    ToastKind::Success,
                                )),
                                ctx,
                            );
                        }
                    }
                }
                DataEvent::SaveSettings => {
                    if let Some(services) = &mut self.services {
                        if let Err(e) = services.app.set_api_key(&self.state.api_key) {
                            log::error!("Failed to save API key: {e}");
                            self.handle_event(
                                AppEvent::Ui(UiEvent::ShowToast(
                                    "Failed to save settings".to_string(),
                                    ToastKind::Error,
                                )),
                                ctx,
                            );
                        } else {
                            services.weather.update_api_key(self.state.api_key.clone());
                            self.handle_event(
                                AppEvent::Ui(UiEvent::ShowToast(
                                    "Settings saved".to_string(),
                                    ToastKind::Success,
                                )),
                                ctx,
                            );
                        }
                    }
                    self.state.show_settings_popup = false;
                }
            },
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
    #[cfg(not(tarpaulin_include))]
    pub fn handle_events(&mut self, events: Vec<AppEvent>, ctx: &egui::Context) {
        for event in events {
            self.handle_event(event, ctx);
        }
    }

    /// Central logic for processing a display mode change.
    pub fn process_display_mode_change(&mut self, mode: DisplayMode) {
        self.state.reset_confirm_mode = false;
        let services = match &mut self.services {
            Some(s) => s,
            None => return,
        };

        services.popup.set_display_mode(mode.clone());
        self.scroll_to_top = true;
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
                // Calculate and set the flight statistics. On WASM the refresh
                // is triggered from the SetDisplayMode handler (which has ctx).
                let stats_result = services.app.get_flight_statistics();
                self.state.statistics = Some(stats_result);
            }
            DisplayMode::Airports => {
                // Show loading
                self.is_loading_airport_items = true;
                self.state.all_items.clear();
                services.search.clear_query();

                let sender = self.airport_items_sender.clone();
                let app = services.app.clone();

                #[cfg(not(target_arch = "wasm32"))]
                std::thread::spawn(move || {
                    let items = app.generate_airport_items();
                    let table_items: Vec<Arc<TableItem>> = items
                        .into_iter()
                        .map(|item| Arc::new(TableItem::Airport(item)))
                        .collect();
                    let _ = sender.send(table_items);
                });

                #[cfg(target_arch = "wasm32")]
                {
                    let items = app.generate_airport_items();
                    let table_items: Vec<Arc<TableItem>> = items
                        .into_iter()
                        .map(|item| Arc::new(TableItem::Airport(item)))
                        .collect();
                    let _ = sender.send(table_items);
                }
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
            DisplayMode::RandomAirports => {
                // Re-generate current random set if needed, but usually handled by process_display_mode_change
                // If we are refreshing, we might want a new set or keep old.
                // For consistency with other views, if we are here, we regenerate based on current random set logic?
                // Actually, AppService generates random airports on demand. It doesn't cache "current random airports" in a field exposed like route_items.
                // So if we call update_displayed_items on RandomAirports, we generate a NEW set.
                let random_airports = services.app.get_random_airports(RANDOM_AIRPORTS_COUNT);
                random_airports
                    .iter()
                    .map(|airport| services.app.create_list_item_for_airport(airport))
                    .map(|item| Arc::new(TableItem::Airport(item)))
                    .collect()
            }
            DisplayMode::Airports => {
                // Handled in process_display_mode_change async
                // If we are here, it might be a refresh. Ideally we should cache it or re-trigger.
                // For now, return empty to avoid blocking, assuming process_display_mode_change is the entry.
                Vec::new()
            }
            DisplayMode::Statistics | DisplayMode::Other => Vec::new(), // Handled elsewhere
        };
        self.update_filtered_items();
    }

    /// Regenerates routes when selections change.
    #[cfg(not(tarpaulin_include))]
    fn regenerate_routes_for_selection_change(&mut self) {
        self.update_routes(RouteUpdateAction::Regenerate);
    }

    /// Loads more routes for infinite scrolling.
    #[cfg(not(tarpaulin_include))]
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
    /// an update is already in progress, unless it's a regeneration request which
    /// should override the current loading state.
    ///
    /// # Arguments
    ///
    /// * `action` - The `RouteUpdateAction` to perform (e.g., `Regenerate` or `Append`).
    #[cfg(not(tarpaulin_include))]
    pub fn update_routes(&mut self, action: RouteUpdateAction) {
        // Prevent stacking of Append requests
        if let RouteUpdateAction::Append = action {
            if self.state.is_loading_more_routes {
                return;
            }
        } else {
            // New generation request: increment ID to invalidate old results
            self.current_route_generation_id
                .fetch_add(1, Ordering::SeqCst);
        }
        // Force update for Regenerate even if loading
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

    pub fn set_all_items(&mut self, items: Vec<Arc<TableItem>>) {
        self.state.all_items = items;
        self.update_filtered_items();
    }

    pub fn update_filtered_items(&mut self) {
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

    pub fn is_route_mode(&self) -> bool {
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

    pub fn get_appropriate_route_mode(&self) -> DisplayMode {
        if self.state.selected_aircraft.is_some() {
            DisplayMode::SpecificAircraftRoutes
        } else {
            DisplayMode::RandomRoutes
        }
    }

    pub fn maybe_switch_to_route_mode(&mut self, selection_being_made: bool) {
        if selection_being_made && !self.is_route_mode() {
            let new_mode = self.get_appropriate_route_mode();
            if let Some(services) = &mut self.services {
                services.popup.set_display_mode(new_mode);
            }
        }
    }

    #[cfg(not(tarpaulin_include))]
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

    #[cfg(not(tarpaulin_include))]
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
    #[cfg(not(tarpaulin_include))]
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
    #[cfg(not(tarpaulin_include))]
    pub fn handle_background_task_results(&mut self, ctx: &egui::Context) {
        // Check for results from the route generation thread
        match self.route_receiver.try_recv() {
            Ok((id, action, new_routes)) => {
                // Discard results from old generation requests
                let current_id = self.current_route_generation_id.load(Ordering::SeqCst);
                if id != current_id {
                    log::info!(
                        "Discarding route generation result ID: {} (current: {})",
                        id,
                        current_id
                    );
                    return;
                }
                log::info!("Processing route generation result ID: {}", id);

                // Extract ICAOs for background fetching before moving new_routes (native only)
                #[cfg(not(target_arch = "wasm32"))]
                let icaos: HashSet<String> = new_routes
                    .iter()
                    .flat_map(|r| vec![r.departure.ICAO.clone(), r.destination.ICAO.clone()])
                    .collect();

                if let Some(services) = &mut self.services {
                    match action {
                        RouteUpdateAction::Append => {
                            services.app.append_route_items(new_routes);
                        }
                        RouteUpdateAction::Regenerate => {
                            services.app.set_route_items(new_routes);
                        }
                    }
                }
                self.update_displayed_items();
                self.state.is_loading_more_routes = false;
                // No need to clear local request as it's cleared on spawn

                // Spawn background fetch for all airports in the routes (native only)
                #[cfg(not(target_arch = "wasm32"))]
                if !icaos.is_empty()
                    && let Some(services) = &self.services
                {
                    let weather_service = services.weather.clone();
                    let ctx_clone = ctx.clone();
                    let generation_id = self.current_route_generation_id.clone();
                    let my_id = current_id;

                    std::thread::spawn(move || {
                        let icaos_vec: Vec<String> = icaos.into_iter().collect();
                        log::info!(
                            "Starting background weather fetch for {} stations. Global ID: {}, My ID: {}",
                            icaos_vec.len(),
                            generation_id.load(Ordering::SeqCst),
                            my_id
                        );

                        icaos_vec.par_iter().for_each(|icao| {
                            if generation_id.load(Ordering::SeqCst) != my_id {
                                return;
                            }
                            let _ = weather_service.fetch_metar(icao);
                        });
                        if generation_id.load(Ordering::SeqCst) == my_id {
                            ctx_clone.request_repaint();
                        } else {
                            log::info!("Weather fetch cancelled/aborted (ID changed).");
                        }
                    });
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
                    services.search.decrement_active_searches();
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
                        let error_msg: String = e.to_string();
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

        // Check for airport items results
        match self.airport_items_receiver.try_recv() {
            Ok(items) => {
                self.set_all_items(items);
                self.is_loading_airport_items = false;
                ctx.request_repaint();
            }
            Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => {
                self.is_loading_airport_items = false;
                log::error!("Airport items thread disconnected unexpectedly.");
            }
        }

        #[cfg(target_arch = "wasm32")]
        self.handle_wasm_background_results(ctx);
    }

    /// WASM-only: drain channels for airport searches, route-from-history popups,
    /// stats refreshes, and async error toasts; trigger debounced airport searches
    /// when any of the airport-driven dropdown text fields change.
    #[cfg(target_arch = "wasm32")]
    #[cfg(not(tarpaulin_include))]
    fn handle_wasm_background_results(&mut self, ctx: &egui::Context) {
        // Apply airport search results (latest wins via generation counter).
        let mut latest_airports: Option<(u64, Vec<crate::models::Airport>)> = None;
        while let Ok((gen_id, airports)) = self.wasm.airport_search_receiver.try_recv() {
            let keep_existing = latest_airports.as_ref().is_some_and(|(g, _)| *g > gen_id);
            if !keep_existing {
                latest_airports = Some((gen_id, airports));
            }
        }
        if let Some((gen_id, airports)) = latest_airports {
            if let Some(services) = &mut self.services {
                services.app.apply_airport_search_result(gen_id, airports);
            }
            // Refresh the displayed airport table if we're currently viewing one.
            if let Some(services) = &self.services {
                let mode = services.popup.display_mode().clone();
                if matches!(mode, DisplayMode::RandomAirports | DisplayMode::Airports) {
                    self.refresh_airport_table_items();
                }
            }
            ctx.request_repaint();
        }

        // Deliver routes built from history into the popup flow.
        let mut route_popups = Vec::new();
        while let Ok(route) = self.wasm.route_popup_receiver.try_recv() {
            route_popups.push(route);
        }
        for route in route_popups {
            self.handle_event(
                AppEvent::Data(crate::gui::events::DataEvent::RouteSelectedForPopup(route)),
                ctx,
            );
        }

        // Apply refreshed statistics.
        while let Ok(stats) = self.wasm.statistics_receiver.try_recv() {
            if let Some(services) = &mut self.services {
                services.app.set_cached_statistics(stats);
                self.state.statistics = Some(services.app.get_flight_statistics());
            }
            ctx.request_repaint();
        }

        // Surface async errors as toasts.
        while let Ok((message, kind)) = self.wasm.toast_receiver.try_recv() {
            self.state.toast_manager.add(message, kind);
            ctx.request_repaint();
        }

        // Deliver paginated history pages.
        let mut history_pages: Vec<crate::models::HistoryPageResponse> = Vec::new();
        while let Ok(page) = self.wasm.history_page_receiver.try_recv() {
            history_pages.push(page);
        }
        if !history_pages.is_empty() {
            self.wasm.is_loading_more_history = false;
            for page in history_pages {
                if let Some(services) = &mut self.services {
                    services.app.extend_history_items(page);
                }
            }
            let is_history = self
                .services
                .as_ref()
                .is_some_and(|s| matches!(s.popup.display_mode(), DisplayMode::History));
            if is_history {
                self.update_displayed_items();
            }
            ctx.request_repaint();
        }

        // Deliver paginated airport browse pages.
        while let Ok((airports, has_more, is_append)) = self.wasm.airports_page_receiver.try_recv()
        {
            self.wasm.is_loading_more_airports = false;
            if let Some(services) = &mut self.services {
                if is_append {
                    services.app.append_airport_browse_page(airports, has_more);
                } else {
                    services.app.apply_airport_browse_page(airports, has_more);
                }
                if matches!(services.popup.display_mode(), DisplayMode::Airports) {
                    self.refresh_airport_table_items();
                }
            }
            ctx.request_repaint();
        }

        self.maybe_trigger_airport_search(ctx);
    }

    /// WASM-only: load the next page of history items from the server.
    #[cfg(target_arch = "wasm32")]
    #[cfg(not(tarpaulin_include))]
    fn load_more_history(&mut self, ctx: &egui::Context) {
        if self.wasm.is_loading_more_history {
            return;
        }
        if let Some(services) = &self.services {
            if !services.app.history_has_more() {
                return;
            }
            self.wasm.is_loading_more_history = true;
            let sender = self.wasm.history_page_sender.clone();
            let ctx_clone = ctx.clone();
            services.app.spawn_load_more_history(move |page| {
                let _ = sender.send(page);
                ctx_clone.request_repaint();
            });
        }
    }

    /// WASM-only: load the next page of airports (browse view or departure dropdown).
    /// Works for both the initial load (replaces the random seed) and subsequent pages.
    #[cfg(target_arch = "wasm32")]
    #[cfg(not(tarpaulin_include))]
    fn load_more_airports(&mut self, ctx: &egui::Context) {
        if self.wasm.is_loading_more_airports {
            return;
        }
        if let Some(services) = &self.services {
            if !services.app.can_load_more_for_dropdown() {
                return;
            }
            self.wasm.is_loading_more_airports = true;
            let sender = self.wasm.airports_page_sender.clone();
            let ctx_clone = ctx.clone();
            services
                .app
                .spawn_load_more_airports(move |airports, has_more, is_append| {
                    let _ = sender.send((airports, has_more, is_append));
                    ctx_clone.request_repaint();
                });
        }
    }

    /// WASM-only: rebuild the airport table from the current cache (used after
    /// an async airport fetch completes while the user is viewing an airport mode).
    #[cfg(target_arch = "wasm32")]
    #[cfg(not(tarpaulin_include))]
    fn refresh_airport_table_items(&mut self) {
        let services = match self.services.as_ref() {
            Some(s) => s,
            None => return,
        };
        let items: Vec<Arc<TableItem>> = match services.popup.display_mode() {
            DisplayMode::RandomAirports => services
                .app
                .get_random_airports(RANDOM_AIRPORTS_COUNT)
                .iter()
                .map(|a| services.app.create_list_item_for_airport(a))
                .map(|item| Arc::new(TableItem::Airport(item)))
                .collect(),
            DisplayMode::Airports => services
                .app
                .generate_airport_items()
                .into_iter()
                .map(|item| Arc::new(TableItem::Airport(item)))
                .collect(),
            _ => return,
        };
        self.is_loading_airport_items = false;
        self.set_all_items(items);
    }

    /// WASM-only: detect changes in dropdown search texts and trigger a
    /// debounced server-side airport search.
    #[cfg(target_arch = "wasm32")]
    #[cfg(not(tarpaulin_include))]
    fn maybe_trigger_airport_search(&mut self, ctx: &egui::Context) {
        let mut new_query: Option<String> = None;
        if self.state.departure_search != self.wasm.last_main_departure_search {
            self.wasm.last_main_departure_search = self.state.departure_search.clone();
            new_query = Some(self.state.departure_search.clone());
        } else if self.state.add_history.departure_search
            != self.wasm.last_add_history_departure_search
        {
            self.wasm.last_add_history_departure_search =
                self.state.add_history.departure_search.clone();
            new_query = Some(self.state.add_history.departure_search.clone());
        } else if self.state.add_history.destination_search
            != self.wasm.last_add_history_destination_search
        {
            self.wasm.last_add_history_destination_search =
                self.state.add_history.destination_search.clone();
            new_query = Some(self.state.add_history.destination_search.clone());
        }

        if let Some(q) = new_query {
            self.wasm.pending_airport_query = Some(q);
            self.wasm.airport_search_debounce_at =
                Some(web_time::Instant::now() + std::time::Duration::from_millis(150));
            ctx.request_repaint_after(std::time::Duration::from_millis(160));
        }

        if let (Some(deadline), Some(query)) = (
            self.wasm.airport_search_debounce_at,
            self.wasm.pending_airport_query.clone(),
        ) && web_time::Instant::now() >= deadline
        {
            self.wasm.airport_search_debounce_at = None;
            self.wasm.pending_airport_query = None;
            if let Some(services) = &mut self.services {
                let generation = services.app.begin_airport_search();
                let sender = self.wasm.airport_search_sender.clone();
                let ctx_clone = ctx.clone();
                services
                    .app
                    .spawn_airport_search(generation, query, 50, move |g, airports| {
                        let _ = sender.send((g, airports));
                        ctx_clone.request_repaint();
                    });
            }
        }
    }

    /// Spawns background tasks when needed (route generation and search).
    #[cfg(not(tarpaulin_include))]
    fn spawn_background_tasks(&mut self, ctx: &egui::Context) {
        let services = match &mut self.services {
            Some(s) => s,
            None => return,
        };

        // Spawn route generation task if requested
        if let Some(action) = self.route_update_request {
            let should_spawn = match action {
                RouteUpdateAction::Regenerate => true,
                RouteUpdateAction::Append => !self.state.is_loading_more_routes,
            };

            if should_spawn {
                self.route_update_request = None;
                self.state.is_loading_more_routes = true;

                // If regenerating, clear existing items immediately to show loading state
                if let RouteUpdateAction::Regenerate = action {
                    services.app.clear_route_items();
                    self.state.all_items.clear(); // Clear the view cache so UI updates immediately
                }

                let sender = self.route_sender.clone();
                let ctx_clone = ctx.clone();
                let departure_icao = services
                    .app
                    .get_selected_airport_icao(&self.state.selected_departure_airport);
                let display_mode = services.popup.display_mode().clone();
                let selected_aircraft = self.state.selected_aircraft.clone();
                let generation_id = self.current_route_generation_id.load(Ordering::SeqCst);
                log::info!(
                    "Spawning route generation thread with ID: {}",
                    generation_id
                );

                services.app.spawn_route_generation_thread(
                    display_mode,
                    selected_aircraft,
                    departure_icao,
                    move |routes| {
                        send_and_repaint(&sender, (generation_id, action, routes), Some(ctx_clone));
                    },
                );
            }
        }

        // Spawn search task if needed
        if services.search.should_execute_search() {
            let sender = self.search_sender.clone();
            let ctx_clone = ctx.clone();
            let all_items = self.state.all_items.clone();

            services.search.increment_active_searches();

            services
                .search
                .spawn_search_thread(all_items, move |filtered_items| {
                    send_and_repaint(&sender, filtered_items, Some(ctx_clone));
                });
        }
    }
}

#[cfg(not(tarpaulin_include))]
fn send_and_repaint<T: Send>(sender: &mpsc::Sender<T>, data: T, ctx: Option<egui::Context>) {
    if sender.send(data).is_ok()
        && let Some(ctx) = ctx
    {
        ctx.request_repaint();
    }
}

#[cfg(not(tarpaulin_include))]
impl eframe::App for Gui {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Handle startup
        if self.services.is_none() {
            if let Some(receiver) = &self.startup_receiver
                && let Ok(result) = receiver.try_recv()
            {
                match result {
                    Ok(services) => {
                        log::info!("Gui::ui: Services received from background thread.");
                        self.services = Some(services);
                        self.startup_receiver = None; // Done loading

                        // Initialize data
                        self.update_displayed_items();

                        // Trigger background weather prefetch for initial routes (native only)
                        #[cfg(not(target_arch = "wasm32"))]
                        if let Some(services) = &mut self.services {
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
                                let ctx_clone = ui.ctx().clone();
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
                    Err(e) => {
                        self.startup_error = Some(e);
                        self.startup_receiver = None;
                    }
                }
            }

            // Show loading or error screen
            #[cfg(target_arch = "wasm32")]
            let mut retry_clicked = false;
            egui::CentralPanel::default().show_inside(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() / 2.0 - 50.0);
                    if let Some(error) = &self.startup_error {
                        ui.heading("❌ Failed to start application");
                        ui.add_space(10.0);
                        ui.label(error);
                        #[cfg(target_arch = "wasm32")]
                        {
                            ui.add_space(15.0);
                            if ui.button("Retry").clicked() {
                                retry_clicked = true;
                            }
                        }
                    } else {
                        ui.spinner();
                        ui.add_space(10.0);
                        ui.heading("Loading application data...");
                    }
                });
            });
            #[cfg(target_arch = "wasm32")]
            if retry_clicked && let Ok(fresh) = Self::spawn_web_startup(ui.ctx().clone()) {
                *self = fresh;
            }
            return;
        }

        let mut events = Vec::new();

        // Handle results from background tasks
        self.handle_background_task_results(ui.ctx());

        // Spawn new background tasks if needed
        self.spawn_background_tasks(ui.ctx());

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
                is_flown: services.popup.is_selected_route_flown(),
            };
            events.extend(RoutePopup::render(&route_popup_vm, ui.ctx()));
        }

        // Handle "Add History" popup
        if self.state.add_history.show_popup
            && let Some(services) = &self.services
        {
            events.extend(AddHistoryPopup::render(
                services.app.aircraft(),
                services.app.airports(),
                &mut self.state.add_history,
                ui.ctx(),
            ));
        }

        // Handle "Settings" popup
        if self.state.show_settings_popup {
            let mut vm = SettingsPopupViewModel {
                api_key: &mut self.state.api_key,
            };
            events.extend(SettingsPopup::render(&mut vm, ui.ctx()));
        }

        egui::CentralPanel::default().show_inside(ui, |ui| {
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
                                    departure_search_autofocus: &mut self
                                        .state
                                        .departure_search_autofocus,
                                    aircraft_search_autofocus: &mut self
                                        .state
                                        .aircraft_search_autofocus,
                                    departure_airport_search: &mut self.state.departure_search,
                                    aircraft_search: &mut self.state.aircraft_search,
                                    departure_display_count: &mut self
                                        .state
                                        .departure_display_count,
                                    aircraft_display_count: &mut self.state.aircraft_display_count,
                                    available_airports: services.app.airports(),
                                    all_aircraft: services.app.aircraft(),
                                };
                                SelectionControls::render(&mut selection_vm, ui, &mut events);
                            }
                            ui.separator();

                            let current_mode = if let Some(services) = &self.services {
                                services.popup.display_mode().clone()
                            } else {
                                DisplayMode::default()
                            };

                            let action_vm = ActionButtonsViewModel::new(
                                true, // Always valid - no departure selection means random departure
                                self.state.is_loading_more_routes,
                                current_mode,
                            );
                            ActionButtons::render(&action_vm, ui, &mut events);
                        },
                    );

                    ui.separator();

                    // --- Right Panel ---
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            if let Some(services) = &mut self.services {
                                if services.popup.display_mode() == &DisplayMode::History
                                    && ui
                                        .add(IconButton::new(icons::ICON_PLUS, "Add to History"))
                                        .on_hover_text(ADD_TO_HISTORY_TOOLTIP)
                                        .clicked()
                                {
                                    events.push(AppEvent::Ui(UiEvent::ShowAddHistoryPopup));
                                }
                                if services.popup.display_mode() == &DisplayMode::Other {
                                    if self.state.reset_confirm_mode {
                                        let mut confirmed = false;
                                        let mut cancelled = false;
                                        ui.horizontal(|ui| {
                                            ui.label("Are you sure?");
                                            if ui
                                                .add(IconButton::new(icons::ICON_CHECK, "Yes"))
                                                .on_hover_text("Confirm reset")
                                                .clicked()
                                            {
                                                confirmed = true;
                                            }
                                            if ui
                                                .add(IconButton::new(icons::ICON_CLOSE, "No"))
                                                .on_hover_text("Cancel reset")
                                                .clicked()
                                            {
                                                cancelled = true;
                                            }
                                        });

                                        if confirmed {
                                            events.push(AppEvent::Data(
                                                DataEvent::MarkAllAircraftAsNotFlown,
                                            ));
                                        }

                                        if confirmed || cancelled {
                                            self.state.reset_confirm_mode = false;
                                        }
                                    } else if ui
                                        .add(IconButton::new(
                                            icons::ICON_RESET,
                                            "Reset all aircraft status",
                                        ))
                                        .on_hover_text("Mark all aircraft as not flown")
                                        .clicked()
                                    {
                                        self.state.reset_confirm_mode = true;
                                    }
                                }
                                if ui
                                    .button(icons::ICON_SETTINGS)
                                    .on_hover_text(SETTINGS_TOOLTIP)
                                    .clicked()
                                {
                                    events.push(AppEvent::Ui(UiEvent::ShowSettingsPopup));
                                }
                                let is_loading = services.search.is_searching();
                                let mut search_vm = SearchControlsViewModel {
                                    query: services.search.query_mut(),
                                    is_loading,
                                };
                                SearchControls::render(&mut search_vm, ui, &mut events);
                            }
                        });

                        ui.add_space(10.0);
                        ui.separator();

                        if self.is_loading_airport_items {
                            ui.horizontal(|ui| {
                                ui.spinner();
                                ui.label("Loading airports...");
                            });
                        } else if let Some(services) = &self.services {
                            let weather_service = &services.weather;
                            let table_vm = TableDisplayViewModel {
                                items: self.get_displayed_items(),
                                display_mode: services.popup.display_mode(),
                                is_loading_more_routes: self.state.is_loading_more_routes,
                                has_more_history: services.app.history_has_more(),
                                has_more_airports: services.app.airports_browse_has_more(),
                                statistics: &self.state.statistics,
                                flight_rules_lookup: Some(&|icao| {
                                    weather_service.get_cached_flight_rules(icao)
                                }),
                                column_widths: &self.state.column_widths,
                                has_active_search: !services.search.query().is_empty(),
                            };
                            let mut scroll = self.scroll_to_top;
                            TableDisplay::render(&table_vm, &mut scroll, ui, &mut events);
                            self.scroll_to_top = scroll;
                        }
                    });
                });
            });
        });

        self.handle_events(events, ui.ctx());

        // Render toasts overlay
        self.state.toast_manager.render(ui.ctx());
    }
}
