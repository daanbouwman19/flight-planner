use crate::database::DatabasePool;
use crate::gui::data::{ListItemRoute, TableItem};
use crate::gui::services::{RouteService, SearchService};
use crate::gui::state::AppState;
use crate::models::{Aircraft, Airport, Runway};
use crate::modules::routes::RouteGenerator;
use crate::traits::{AircraftOperations, AirportOperations};
use eframe::egui::{self};
use log;
use rstar::{RTree, RTreeObject, AABB};
use std::collections::HashMap;
use std::sync::Arc;

/// The main GUI application.
pub struct Gui<'a> {
    /// The database pool for accessing data.
    database_pool: &'a mut DatabasePool,
    /// The main application state.
    app_state: AppState,
}

/// A spatial index object for airports.
pub struct SpatialAirport {
    pub airport: Arc<Airport>,
}

/// Implement the `RTreeObject` trait for `SpatialAirport` for the `RTree` index.
impl RTreeObject for SpatialAirport {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        let point = [self.airport.Latitude, self.airport.Longtitude];
        AABB::from_point(point)
    }
}

impl<'a> Gui<'a> {
    /// Creates a new GUI instance.
    ///
    /// # Arguments
    ///
    /// * `_cc` - The creation context.
    /// * `database_pool` - A mutable reference to the database pool.
    pub fn new(_cc: &eframe::CreationContext, database_pool: &'a mut DatabasePool) -> Self {
        let all_aircraft = database_pool
            .get_all_aircraft()
            .expect("Failed to load aircraft");
        let all_airports = database_pool
            .get_airports()
            .expect("Failed to load airports");
        let runway_data = database_pool.get_runways().expect("Failed to load runways");

        let mut runway_map: HashMap<i32, Vec<Runway>> = HashMap::new();
        for runway in runway_data {
            runway_map.entry(runway.AirportID).or_default().push(runway);
        }

        let all_runways: HashMap<i32, Arc<Vec<Runway>>> = runway_map
            .into_iter()
            .map(|(id, runways)| (id, Arc::new(runways)))
            .collect();

        let all_aircraft: Vec<Arc<Aircraft>> = all_aircraft.into_iter().map(Arc::new).collect();
        let all_airports: Vec<Arc<Airport>> = all_airports.into_iter().map(Arc::new).collect();

        let spatial_airports = RTree::bulk_load(
            all_airports
                .iter()
                .map(|airport| SpatialAirport {
                    airport: Arc::clone(airport),
                })
                .collect(),
        );

        let route_generator = RouteGenerator {
            all_airports,
            all_runways,
            spatial_airports,
        };

        let app_state = AppState::new(all_aircraft, route_generator);

        Gui {
            database_pool,
            app_state,
        }
    }

    // Getter methods for better encapsulation using the new state structure
    /// Gets the current departure airport ICAO code.
    pub fn get_departure_airport_icao(&self) -> &str {
        self.app_state.get_ui_state().get_departure_airport_icao()
    }

    /// Gets the departure airport validation result if available.
    pub const fn get_departure_airport_validation(&self) -> Option<bool> {
        self.app_state
            .get_ui_state()
            .get_departure_airport_validation()
    }

    /// Gets a reference to the available airports.
    pub fn get_available_airports(&self) -> &[Arc<Airport>] {
        self.app_state.get_available_airports()
    }

    /// Gets a reference to all aircraft.
    pub fn get_all_aircraft(&self) -> &[Arc<Aircraft>] {
        self.app_state.get_all_aircraft()
    }

    /// Gets the current selected aircraft.
    pub const fn get_selected_aircraft(&self) -> Option<&Arc<Aircraft>> {
        self.app_state.get_ui_state().get_selected_aircraft()
    }

    /// Gets the current aircraft search text.
    pub fn get_aircraft_search(&self) -> &str {
        self.app_state.get_ui_state().get_aircraft_search()
    }

    /// Gets whether the aircraft dropdown is open.
    pub const fn is_aircraft_dropdown_open(&self) -> bool {
        self.app_state.get_ui_state().is_aircraft_dropdown_open()
    }

    /// Gets the current search query.
    pub fn get_search_query(&self) -> &str {
        self.app_state.get_search_state().get_query()
    }

    /// Gets the filtered items from search.
    pub fn get_filtered_items(&self) -> &[Arc<TableItem>] {
        self.app_state.get_search_state().get_filtered_items()
    }

    // State management methods for better encapsulation
    /// Clears the departure airport validation cache.
    pub fn clear_departure_validation_cache(&mut self) {
        self.app_state
            .get_ui_state_mut()
            .clear_departure_validation_cache();
    }

    /// Sets the departure airport validation result and updates the cache.
    pub fn set_departure_validation(&mut self, icao: &str, is_valid: bool) {
        self.app_state
            .get_ui_state_mut()
            .set_departure_validation(icao, is_valid);
    }

    /// Checks if departure airport validation needs to be refreshed.
    pub fn needs_departure_validation_refresh(&self) -> bool {
        self.app_state
            .get_ui_state()
            .needs_departure_validation_refresh()
    }

    /// Sets the selected aircraft.
    pub fn set_selected_aircraft(&mut self, aircraft: Option<Arc<Aircraft>>) {
        self.app_state
            .get_ui_state_mut()
            .set_selected_aircraft(aircraft);
    }

    /// Sets the aircraft search text.
    pub fn set_aircraft_search(&mut self, search: String) {
        self.app_state
            .get_ui_state_mut()
            .set_aircraft_search(search);
    }

    /// Sets whether the aircraft dropdown is open.
    pub const fn set_aircraft_dropdown_open(&mut self, open: bool) {
        self.app_state
            .get_ui_state_mut()
            .set_aircraft_dropdown_open(open);
    }

    /// Updates the search query and triggers search if changed.
    pub fn update_search_query(&mut self, query: String) {
        self.app_state.get_search_state_mut().update_query(query);
    }

    /// Checks if debounced search should be executed.
    pub fn should_execute_search(&self) -> bool {
        self.app_state.get_search_state().should_execute_search()
    }

    /// Sets the filtered items.
    pub fn set_filtered_items(&mut self, items: Vec<Arc<TableItem>>) {
        self.app_state
            .get_search_state_mut()
            .set_filtered_items(items);
    }

    /// Sets whether a search is pending.
    pub const fn set_search_pending(&mut self, pending: bool) {
        self.app_state
            .get_search_state_mut()
            .set_search_pending(pending);
    }

    // Additional getter/setter methods for better encapsulation

    /// Gets a mutable reference to the database pool.
    pub const fn get_database_pool(&mut self) -> &mut DatabasePool {
        self.database_pool
    }

    /// Gets a reference to the displayed items.
    pub fn get_displayed_items(&self) -> &[Arc<TableItem>] {
        self.app_state.get_displayed_items()
    }

    /// Sets the displayed items.
    pub fn set_displayed_items(&mut self, items: Vec<Arc<TableItem>>) {
        self.app_state.set_displayed_items(items);
    }

    /// Clears the displayed items.
    pub fn clear_displayed_items(&mut self) {
        self.app_state.clear_displayed_items();
    }

    /// Extends the displayed items with additional items.
    pub fn extend_displayed_items(&mut self, items: impl IntoIterator<Item = Arc<TableItem>>) {
        self.app_state.extend_displayed_items(items);
    }

    /// Gets whether to show the alert popup.
    pub const fn show_alert(&self) -> bool {
        self.app_state.get_popup_state().show_alert()
    }

    /// Gets the currently selected route.
    pub const fn get_selected_route(&self) -> Option<&ListItemRoute> {
        self.app_state.get_popup_state().get_selected_route()
    }

    /// Gets whether routes are from not flown aircraft.
    pub const fn routes_from_not_flown(&self) -> bool {
        self.app_state.get_popup_state().routes_from_not_flown()
    }

    /// Gets whether routes are for a specific aircraft.
    pub const fn routes_for_specific_aircraft(&self) -> bool {
        self.app_state
            .get_popup_state()
            .routes_for_specific_aircraft()
    }

    /// Gets whether the current items are random airports.
    pub const fn airports_random(&self) -> bool {
        self.app_state.get_popup_state().airports_random()
    }

    /// Sets whether to show the alert popup.
    pub const fn set_show_alert(&mut self, show: bool) {
        self.app_state.get_popup_state_mut().set_show_alert(show);
    }

    /// Sets the selected route in the popup state.
    pub fn set_selected_route(&mut self, route: Option<ListItemRoute>) {
        self.app_state
            .get_popup_state_mut()
            .set_selected_route(route);
    }

    /// Sets whether routes are from not flown aircraft.
    pub const fn set_routes_from_not_flown(&mut self, from_not_flown: bool) {
        self.app_state
            .get_popup_state_mut()
            .set_routes_from_not_flown(from_not_flown);
    }

    /// Sets whether routes are for a specific aircraft.
    pub const fn set_routes_for_specific_aircraft(&mut self, for_specific: bool) {
        self.app_state
            .get_popup_state_mut()
            .set_routes_for_specific_aircraft(for_specific);
    }

    /// Sets whether the current items are random airports.
    pub const fn set_airports_random(&mut self, airports_random: bool) {
        self.app_state
            .get_popup_state_mut()
            .set_airports_random(airports_random);
    }

    /// Gets a reference to the route generator.
    pub const fn get_route_service(&self) -> &RouteService {
        self.app_state.get_route_service()
    }

    /// Gets a mutable reference to the departure airport ICAO code.
    pub const fn get_departure_airport_icao_mut(&mut self) -> &mut String {
        self.app_state
            .get_ui_state_mut()
            .get_departure_airport_icao_mut()
    }

    /// Handles the action when the "Mark as flown" button is pressed.
    ///
    /// # Arguments
    ///
    /// * `route` - The route to mark as flown.
    pub fn handle_mark_flown_button(&mut self, route: &ListItemRoute) {
        self.set_show_alert(false);

        if let Err(e) = RouteService::mark_route_as_flown(self.database_pool, route) {
            log::error!("Failed to mark route as flown: {e}");
            return;
        }

        // Refresh aircraft data
        let all_aircraft = self
            .database_pool
            .get_all_aircraft()
            .expect("Failed to load aircraft");

        self.app_state
            .set_all_aircraft(all_aircraft.into_iter().map(Arc::new).collect());
    }

    /// Handles user input and updates state.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The egui context.
    pub fn handle_input(&mut self, ctx: &egui::Context) {
        if self.show_alert() {
            self.show_modal_popup(ctx);
        }
    }

    /// Filters the displayed items based on the search query.
    pub fn handle_search(&mut self) {
        let query = self.get_search_query().to_string();
        let displayed_items = self.get_displayed_items().to_vec();
        let filtered_items = SearchService::filter_items(&displayed_items, &query);
        self.set_filtered_items(filtered_items);
    }

    /// Resets the UI state to a clean state and triggers search refresh.
    /// This method now uses encapsulated state management for better maintainability.
    ///
    /// # Arguments
    ///
    /// * `clear_search_query` - Whether to clear the search query or keep it
    pub fn reset_ui_state_and_refresh(&mut self, clear_search_query: bool) {
        self.app_state.reset_state(clear_search_query, true);
        self.handle_search();
    }

    /// Loads more routes if needed using encapsulated state access.
    pub fn load_more_routes_if_needed(&mut self) {
        // Only load more items if we're not searching
        if !self.get_search_query().is_empty() {
            return;
        }

        // Check what type of items we have and if infinite scrolling applies
        if let Some(first_item) = self.get_displayed_items().first() {
            match first_item.as_ref() {
                TableItem::Route(_) => {
                    // Handle route infinite scrolling
                    self.load_more_routes();
                }
                TableItem::Airport(_) => {
                    // Handle airport infinite scrolling
                    if self.airports_random() {
                        self.load_more_airports();
                    }
                }
                _ => {
                    // Other item types don't support infinite scrolling
                }
            }
        }
    }

    /// Loads more routes for infinite scrolling.
    fn load_more_routes(&mut self) {
        let departure_icao = if self.get_departure_airport_icao().is_empty() {
            None
        } else {
            Some(self.get_departure_airport_icao())
        };

        let all_aircraft = self.get_all_aircraft().to_vec(); // Clone to avoid borrowing conflicts
        let route_service = self.get_route_service();

        let routes = if self.routes_from_not_flown() {
            route_service.generate_not_flown_routes(&all_aircraft, departure_icao)
        } else if self.routes_for_specific_aircraft() {
            // If we're generating routes for a specific aircraft, use the selected aircraft
            self.get_selected_aircraft().map_or_else(
                || {
                    // If somehow there's no selected aircraft, fall back to all aircraft
                    log::warn!(
                        "No selected aircraft when generating routes for a specific aircraft"
                    );
                    route_service.generate_random_routes(&all_aircraft, departure_icao)
                },
                |selected_aircraft| {
                    route_service.generate_routes_for_aircraft(selected_aircraft, departure_icao)
                },
            )
        } else {
            // Otherwise generate random routes for all aircraft
            route_service.generate_random_routes(&all_aircraft, departure_icao)
        };

        self.extend_displayed_items(routes);

        // Update filtered items after adding more routes
        self.handle_search();
    }

    /// Loads more airports for infinite scrolling.
    fn load_more_airports(&mut self) {
        let route_service = self.get_route_service();
        let airports = route_service.generate_random_airports(50);

        self.extend_displayed_items(airports);

        // Update filtered items after adding more airports
        self.handle_search();
    }
}

impl eframe::App for Gui<'_> {
    /// Updates the application state and the user interface.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The egui context.
    /// * `_frame` - The eframe frame.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_input(ctx);
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_enabled_ui(!self.show_alert(), |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                    // Left panel with buttons - fixed width
                    ui.allocate_ui_with_layout(
                        egui::Vec2::new(250.0, ui.available_height()),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            self.update_buttons(ui);
                        },
                    );

                    ui.separator();

                    // Right panel with search and table - takes remaining space
                    ui.vertical(|ui| {
                        self.update_search_bar(ui);
                        ui.add_space(10.0);
                        ui.separator();
                        self.update_table(ui);
                    });
                });
            });
        });
    }
}
