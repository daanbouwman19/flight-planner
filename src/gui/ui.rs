use crate::database::DatabasePool;
use crate::gui::components::{
    action_buttons::{ActionButtons, ActionButtonsViewModel},
    route_popup::RoutePopup,
    search_controls::{SearchControls, SearchControlsViewModel},
    selection_controls::{SelectionControls, SelectionControlsViewModel},
    table_display::{TableDisplay, TableDisplayViewModel},
};
use crate::gui::data::{ListItemAircraft, ListItemAirport, TableItem};
use crate::gui::events::Event;
use crate::gui::state::{
    app_state::AppState,
    popup_state::{DisplayMode, PopupState},
    search_state::SearchState,
};
use crate::models::{Aircraft, Airport};
use crate::modules::data_operations::FlightStatistics;
use eframe::egui::{self};
use log;
use rstar::{AABB, RTreeObject};
use std::error::Error;
use std::sync::Arc;

/// The main GUI application using a view-model-driven approach.
pub struct Gui {
    /// The main application state (business logic and data).
    app_state: AppState,
    /// State for search functionality.
    search_state: SearchState,
    /// State for popup dialogs.
    popup_state: PopupState,

    // --- State moved from former ViewState ---
    /// Currently selected departure airport.
    selected_departure_airport: Option<Arc<Airport>>,
    /// Currently selected aircraft.
    selected_aircraft: Option<Arc<Aircraft>>,
    /// Whether the aircraft dropdown is open.
    aircraft_dropdown_open: bool,
    /// Whether the departure airport dropdown is open.
    departure_dropdown_open: bool,
    /// Dropdown display counts for pagination.
    aircraft_display_count: usize,
    departure_display_count: usize,
    /// All items for the current view.
    all_items: Vec<Arc<TableItem>>,
    /// Infinite scrolling state.
    is_loading_more_routes: bool,
    /// Separate search state for aircraft dropdown.
    aircraft_search: String,
    /// Separate search state for departure airport dropdown.
    departure_airport_search: String,
    /// Cached flight statistics.
    flight_statistics: Option<Result<FlightStatistics, Box<dyn Error + Send + Sync>>>,
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
    /// Creates a new GUI instance.
    pub fn new(
        _cc: &eframe::CreationContext,
        database_pool: DatabasePool,
    ) -> Result<Self, Box<dyn Error>> {
        let app_state = AppState::new(database_pool)?;
        Ok(Gui {
            app_state,
            search_state: SearchState::default(),
            popup_state: PopupState::default(),
            selected_departure_airport: None,
            selected_aircraft: None,
            aircraft_dropdown_open: false,
            departure_dropdown_open: false,
            aircraft_display_count: 50,
            departure_display_count: 50,
            all_items: Vec::new(),
            is_loading_more_routes: false,
            aircraft_search: String::new(),
            departure_airport_search: String::new(),
            flight_statistics: None,
        })
    }

    /// Handles a single UI event, updating the state accordingly.
    fn handle_event(&mut self, event: Event) {
        match event {
            // --- SelectionControls Events ---
            Event::DepartureAirportSelected(airport) => {
                self.maybe_switch_to_route_mode(airport.is_some());
                self.selected_departure_airport = airport;
                self.departure_dropdown_open = false;
            }
            Event::AircraftSelected(aircraft) => {
                let aircraft_being_selected = aircraft.is_some();
                self.maybe_switch_to_route_mode(aircraft_being_selected);
                self.selected_aircraft = aircraft;
                self.aircraft_dropdown_open = false;
                self.handle_route_mode_transition();
            }
            Event::ToggleDepartureAirportDropdown => {
                self.departure_dropdown_open = !self.departure_dropdown_open;
                if self.departure_dropdown_open {
                    self.aircraft_dropdown_open = false;
                }
            }
            Event::ToggleAircraftDropdown => {
                self.aircraft_dropdown_open = !self.aircraft_dropdown_open;
                if self.aircraft_dropdown_open {
                    self.departure_dropdown_open = false;
                }
            }

            // --- ActionButtons Events ---
            Event::SetDisplayMode(mode) => self.process_display_mode_change(mode),
            Event::RegenerateRoutesForSelectionChange => self.regenerate_routes_for_selection_change(),

            // --- TableDisplay Events ---
            Event::RouteSelectedForPopup(route) => self.popup_state.set_selected_route(Some(route)),
            Event::SetShowPopup(show) => self.popup_state.set_show_alert(show),
            Event::ToggleAircraftFlownStatus(aircraft_id) => {
                if let Err(e) = self.app_state.toggle_aircraft_flown_status(aircraft_id) {
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

    /// Central logic for processing a display mode change.
    fn process_display_mode_change(&mut self, mode: DisplayMode) {
        self.popup_state.set_display_mode(mode.clone());
        match mode {
            DisplayMode::RandomAirports => {
                let random_airports = self.app_state.get_random_airports(50);
                let airport_items: Vec<_> = random_airports
                    .iter()
                    .map(|airport| {
                        let runways = self.app_state.get_runways_for_airport(airport);
                        let runway_length = runways
                            .iter()
                            .max_by_key(|r| r.Length)
                            .map_or("No runways".to_string(), |r| format!("{}ft", r.Length));
                        ListItemAirport::new(airport.Name.clone(), airport.ICAO.clone(), runway_length)
                    })
                    .collect();
                let table_items: Vec<Arc<TableItem>> = airport_items
                    .into_iter()
                    .map(|item| Arc::new(TableItem::Airport(item)))
                    .collect();
                self.set_all_items(table_items);
            }
            DisplayMode::Other => { // Currently "List all aircraft"
                let aircraft_items: Vec<_> = self.app_state.aircraft().iter().map(ListItemAircraft::new).collect();
                let table_items: Vec<Arc<TableItem>> = aircraft_items
                    .into_iter()
                    .map(|item| Arc::new(TableItem::Aircraft(item)))
                    .collect();
                self.set_all_items(table_items);
            }
            DisplayMode::Statistics => {
                self.flight_statistics = Some(self.app_state.get_flight_statistics());
                self.all_items.clear();
                self.search_state.clear_query();
            }
            _ => self.update_displayed_items(),
        }
    }

    /// Updates the displayed items based on the current mode.
    pub fn update_displayed_items(&mut self) {
        self.all_items = match self.popup_state.get_display_mode() {
            DisplayMode::RandomRoutes | DisplayMode::NotFlownRoutes | DisplayMode::SpecificAircraftRoutes => {
                self.app_state.route_items().iter().map(|r| Arc::new(TableItem::Route(r.clone()))).collect()
            }
            DisplayMode::History => self.app_state.history_items().iter().map(|h| Arc::new(TableItem::History(h.clone()))).collect(),
            DisplayMode::Airports | DisplayMode::RandomAirports => self.app_state.airport_items().iter().map(|a| Arc::new(TableItem::Airport(a.clone()))).collect(),
            DisplayMode::Statistics | DisplayMode::Other => Vec::new(), // Handled elsewhere
        };
        self.update_filtered_items();
    }

    /// Regenerates routes when selections change.
    fn regenerate_routes_for_selection_change(&mut self) {
        let departure_icao = self.selected_departure_airport.as_ref().map(|a| a.ICAO.as_str());
        let should_update = match self.popup_state.get_display_mode() {
            DisplayMode::RandomRoutes | DisplayMode::SpecificAircraftRoutes => {
                if let Some(aircraft) = &self.selected_aircraft {
                    self.app_state.regenerate_routes_for_aircraft(aircraft, departure_icao);
                } else {
                    self.app_state.regenerate_random_routes(departure_icao);
                }
                true
            }
            DisplayMode::NotFlownRoutes => {
                self.app_state.regenerate_not_flown_routes(departure_icao);
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
        if self.is_loading_more_routes || !self.is_route_mode() {
            return;
        }
        self.is_loading_more_routes = true;
        let departure_icao = self.selected_departure_airport.as_ref().map(|a| a.ICAO.as_str());
        match self.popup_state.get_display_mode() {
            DisplayMode::RandomRoutes | DisplayMode::SpecificAircraftRoutes => {
                if let Some(aircraft) = &self.selected_aircraft {
                    self.app_state.append_routes_for_aircraft(aircraft, departure_icao);
                } else {
                    self.app_state.append_random_routes(departure_icao);
                }
            }
            DisplayMode::NotFlownRoutes => self.app_state.append_not_flown_routes(departure_icao),
            _ => unreachable!(),
        }
        self.update_displayed_items();
        self.is_loading_more_routes = false;
    }

    // --- Helper methods for state management ---

    fn get_displayed_items(&self) -> &[Arc<TableItem>] {
        if self.search_state.get_query().is_empty() {
            &self.all_items
        } else {
            self.search_state.get_filtered_items()
        }
    }

    fn set_all_items(&mut self, items: Vec<Arc<TableItem>>) {
        self.all_items = items;
        self.update_filtered_items();
    }

    fn update_filtered_items(&mut self) {
        use crate::gui::services::SearchService;
        let query = self.search_state.get_query();
        let filtered_items = SearchService::filter_items(&self.all_items, query);
        self.search_state.set_filtered_items(filtered_items);
    }

    fn is_route_mode(&self) -> bool {
        matches!(
            self.popup_state.get_display_mode(),
            DisplayMode::RandomRoutes | DisplayMode::NotFlownRoutes | DisplayMode::SpecificAircraftRoutes
        )
    }

    fn get_appropriate_route_mode(&self) -> DisplayMode {
        if self.selected_aircraft.is_some() {
            DisplayMode::SpecificAircraftRoutes
        } else {
            DisplayMode::RandomRoutes
        }
    }

    fn maybe_switch_to_route_mode(&mut self, selection_being_made: bool) {
        if selection_being_made && !self.is_route_mode() {
            let new_mode = self.get_appropriate_route_mode();
            self.popup_state.set_display_mode(new_mode);
        }
    }

    fn handle_route_mode_transition(&mut self) {
        if self.is_route_mode() {
            let new_mode = self.get_appropriate_route_mode();
            if new_mode != *self.popup_state.get_display_mode() {
                self.popup_state.set_display_mode(new_mode);
            }
        }
    }

    fn refresh_aircraft_items_if_needed(&mut self) {
        if matches!(self.popup_state.get_display_mode(), DisplayMode::Other) {
            let aircraft_items: Vec<_> = self.app_state.aircraft().iter().map(ListItemAircraft::new).collect();
            let table_items: Vec<Arc<TableItem>> = aircraft_items.into_iter().map(|item| Arc::new(TableItem::Aircraft(item))).collect();
            self.set_all_items(table_items);
        }
    }

    // --- Methods for RoutePopup compatibility ---

    pub fn show_alert(&self) -> bool {
        self.popup_state.show_alert()
    }

    pub fn get_selected_route(&self) -> Option<&crate::gui::data::ListItemRoute> {
        self.popup_state.get_selected_route()
    }

    pub fn routes_from_not_flown(&self) -> bool {
        self.popup_state.routes_from_not_flown()
    }

    pub fn set_show_alert(&mut self, show: bool) {
        self.popup_state.set_show_alert(show);
    }

    pub fn mark_route_as_flown(
        &mut self,
        route: &crate::gui::data::ListItemRoute,
    ) -> Result<(), Box<dyn Error>> {
        self.app_state.mark_route_as_flown(route)
    }
}

impl eframe::App for Gui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle popup input first, as it might block other interactions.
        if self.popup_state.show_alert() {
            RoutePopup::render(self, ctx);
        }

        let mut events = Vec::new();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_enabled_ui(!self.popup_state.show_alert(), |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                    // --- Left Panel ---
                    ui.allocate_ui_with_layout(
                        egui::Vec2::new(250.0, ui.available_height()),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            // Selection controls
                            let mut sc_vm = SelectionControlsViewModel {
                                selected_departure_airport: &self.selected_departure_airport,
                                selected_aircraft: &self.selected_aircraft,
                                departure_dropdown_open: self.departure_dropdown_open,
                                aircraft_dropdown_open: self.aircraft_dropdown_open,
                                departure_airport_search: &mut self.departure_airport_search,
                                aircraft_search: &mut self.aircraft_search,
                                departure_display_count: &mut self.departure_display_count,
                                aircraft_display_count: &mut self.aircraft_display_count,
                                available_airports: self.app_state.airports(),
                                all_aircraft: self.app_state.aircraft(),
                            };
                            events.extend(SelectionControls::render(&mut sc_vm, ui));
                            ui.separator();

                            // Action buttons
                            let ab_vm = ActionButtonsViewModel {
                                departure_airport_valid: self.selected_departure_airport.is_some()
                                    || self.departure_airport_search.is_empty(),
                            };
                            events.extend(ActionButtons::render(&ab_vm, ui));
                        },
                    );

                    ui.separator();

                    // --- Right Panel ---
                    ui.vertical(|ui| {
                        // Search controls
                        let mut search_vm = SearchControlsViewModel {
                            query: self.search_state.get_query_mut(),
                        };
                        events.extend(SearchControls::render(&mut search_vm, ui));
                        ui.add_space(10.0);
                        ui.separator();

                        // Table display
                        let td_vm = TableDisplayViewModel {
                            items: self.get_displayed_items(),
                            display_mode: self.popup_state.get_display_mode(),
                            is_loading_more_routes: self.is_loading_more_routes,
                            statistics: &self.flight_statistics,
                        };
                        events.extend(TableDisplay::render(&td_vm, ui));
                    });
                });
            });
        });

        // Handle all collected events
        for event in events {
            self.handle_event(event);
        }
    }
}
