use crate::database::DatabasePool;
use crate::gui::components::{
    action_buttons::ActionButtons, route_popup::RoutePopup, search_controls::SearchControls,
    selection_controls::SelectionControls, table_display::TableDisplay,
};
use crate::gui::data::{ListItemRoute, TableItem};
use crate::gui::state::{popup_state::DisplayMode, AppState};
use crate::models::{Aircraft, Airport};
use crate::modules::routes::RouteGenerator;
use eframe::egui::{self};
use log;
use rstar::{RTreeObject, AABB};
use std::sync::Arc;

/// The main GUI application using clean architecture.
pub struct Gui {
    /// The main application state with clean architecture.
    app_state: AppState,
    /// UI-specific state that doesn't belong in the clean architecture
    ui_state: UiState,
}

/// UI-specific state that complements the clean AppState
pub struct UiState {
    /// Currently selected departure airport
    selected_departure_airport: Option<Arc<Airport>>,
    /// Currently selected aircraft
    selected_aircraft: Option<Arc<Aircraft>>,
    /// Current search query
    search_query: String,
    /// Whether dropdowns are open
    aircraft_dropdown_open: bool,
    departure_dropdown_open: bool,
    /// Dropdown display counts for pagination
    aircraft_display_count: usize,
    departure_display_count: usize,
    /// Current display mode for items
    display_mode: DisplayMode,
    /// Popup state
    show_popup: bool,
    selected_route_for_popup: Option<ListItemRoute>,
    /// Search-related state
    search_should_execute: bool,
    filtered_items: Vec<TableItem>,
    all_items: Vec<TableItem>,
    /// Infinite scrolling state
    is_loading_more_routes: bool,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            selected_departure_airport: None,
            selected_aircraft: None,
            search_query: String::new(),
            aircraft_dropdown_open: false,
            departure_dropdown_open: false,
            aircraft_display_count: 50,
            departure_display_count: 50,
            display_mode: DisplayMode::RandomRoutes,
            show_popup: false,
            selected_route_for_popup: None,
            search_should_execute: false,
            filtered_items: Vec::new(),
            all_items: Vec::new(),
            is_loading_more_routes: false,
        }
    }
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

impl Gui {
    /// Creates a new GUI instance using clean architecture.
    ///
    /// # Arguments
    ///
    /// * `_cc` - The creation context.
    /// * `database_pool` - A mutable reference to the database pool.
    pub fn new(
        _cc: &eframe::CreationContext,
        database_pool: DatabasePool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Create the clean AppState - this handles all data and business logic
        let app_state = AppState::new(database_pool)?;

        // Create UI-specific state
        let ui_state = UiState::default();

        Ok(Gui {
            app_state,
            ui_state,
        })
    }

    // Clean architecture getter methods - these delegate to AppState

    /// Gets the current departure airport.
    pub fn get_departure_airport(&self) -> Option<&Arc<Airport>> {
        self.ui_state.selected_departure_airport.as_ref()
    }

    /// Gets the current departure airport ICAO code.
    pub fn get_departure_airport_icao(&self) -> &str {
        self.ui_state
            .selected_departure_airport
            .as_ref()
            .map(|airport| airport.ICAO.as_str())
            .unwrap_or("")
    }

    /// Gets the departure airport validation result if available.
    pub fn get_departure_airport_validation(&self) -> Option<bool> {
        // In clean architecture, validation happens when needed
        self.ui_state
            .selected_departure_airport
            .as_ref()
            .map(|_| true) // If we have an airport, it's valid
    }

    /// Gets a reference to the available airports using clean architecture.
    pub fn get_available_airports(&self) -> &[Arc<Airport>] {
        self.app_state.airports()
    }

    /// Gets runway data for an airport.
    pub fn get_runways_for_airport(&self, airport: &Airport) -> Vec<Arc<crate::models::Runway>> {
        self.app_state.get_runways_for_airport(airport)
    }

    /// Gets a reference to all aircraft using clean architecture.
    pub fn get_all_aircraft(&self) -> &[Arc<Aircraft>] {
        self.app_state.aircraft()
    }

    /// Gets the current selected aircraft.
    pub fn get_selected_aircraft(&self) -> Option<&Arc<Aircraft>> {
        self.ui_state.selected_aircraft.as_ref()
    }

    /// Gets the current aircraft search query.
    pub fn get_aircraft_search(&self) -> &str {
        &self.ui_state.search_query
    }

    /// Gets whether the aircraft dropdown is open.
    pub fn is_aircraft_dropdown_open(&self) -> bool {
        self.ui_state.aircraft_dropdown_open
    }

    /// Gets the current departure airport search query.
    pub fn get_departure_airport_search(&self) -> &str {
        "" // Clean architecture handles this differently
    }

    /// Gets whether the departure airport dropdown is open.
    pub fn is_departure_airport_dropdown_open(&self) -> bool {
        self.ui_state.departure_dropdown_open
    }

    /// Sets whether the departure airport dropdown is open.
    pub fn set_departure_airport_dropdown_open(&mut self, open: bool) {
        self.ui_state.departure_dropdown_open = open;
    }

    /// Gets a mutable reference to the aircraft dropdown display count.
    pub fn get_aircraft_dropdown_display_count_mut(&mut self) -> &mut usize {
        &mut self.ui_state.aircraft_display_count
    }

    /// Gets a mutable reference to the departure airport dropdown display count.
    pub fn get_departure_airport_dropdown_display_count_mut(&mut self) -> &mut usize {
        &mut self.ui_state.departure_display_count
    }

    /// Sets the aircraft search text.
    pub fn set_aircraft_search(&mut self, search: String) {
        self.ui_state.search_query = search;
    }

    /// Sets the departure airport search text.
    pub fn set_departure_airport_search(&mut self, _search: String) {
        // Clean architecture handles this differently
    }

    /// Updates the departure validation state.
    pub fn update_departure_validation_state(&mut self) {
        // Clean architecture handles validation differently
    }

    /// Regenerates routes for departure change.
    pub fn regenerate_routes_for_departure_change(&mut self) {
        if let Some(ref airport) = self.ui_state.selected_departure_airport {
            let icao = airport.ICAO.clone();
            self.regenerate_random_routes_for_departure(&icao);
        } else {
            self.regenerate_random_routes_for_departure("");
        }
    }

    /// Sets the aircraft dropdown open state.
    pub fn set_aircraft_dropdown_open(&mut self, open: bool) {
        self.ui_state.aircraft_dropdown_open = open;
    }

    /// Checks if the popup/alert should be shown.
    pub fn show_popup(&self) -> bool {
        self.ui_state.show_popup
    }

    /// Handles user input and modal popups.
    pub fn handle_input(&mut self, ctx: &egui::Context) {
        if self.show_popup() {
            self.show_modal_popup(ctx);
        }
    }

    /// Shows the route details modal popup.
    pub fn show_modal_popup(&mut self, ctx: &egui::Context) {
        RoutePopup::render(self, ctx);
    }

    /// Sets whether to show the popup.
    pub fn set_show_popup(&mut self, show: bool) {
        self.ui_state.show_popup = show;
    }

    /// Sets the selected route for popup.
    pub fn set_selected_route_for_popup(&mut self, route: Option<ListItemRoute>) {
        self.ui_state.selected_route_for_popup = route;
    }

    /// Gets the selected route for popup.
    pub fn get_selected_route_for_popup(&self) -> Option<&ListItemRoute> {
        self.ui_state.selected_route_for_popup.as_ref()
    }

    /// Gets the current search query.
    pub fn get_search_query(&self) -> &str {
        &self.ui_state.search_query
    }

    /// Gets the filtered items.
    pub fn get_filtered_items(&self) -> &[TableItem] {
        &self.ui_state.filtered_items
    }

    // UI state mutation methods

    /// Sets the selected departure airport.
    pub fn set_departure_airport(&mut self, airport: Option<Arc<Airport>>) {
        self.ui_state.selected_departure_airport = airport;
        // When departure changes, regenerate routes
        if let Some(ref airport) = self.ui_state.selected_departure_airport {
            let icao = airport.ICAO.clone(); // Clone the ICAO to avoid borrow issues
            self.regenerate_random_routes_for_departure(&icao);
        }
    }

    /// Sets the selected aircraft.
    pub fn set_selected_aircraft(&mut self, aircraft: Option<Arc<Aircraft>>) {
        self.ui_state.selected_aircraft = aircraft;
        // When aircraft changes, regenerate routes
        self.regenerate_random_routes();
    }

    /// Opens the aircraft dropdown.
    pub fn open_aircraft_dropdown(&mut self) {
        self.ui_state.aircraft_dropdown_open = true;
    }

    /// Closes the aircraft dropdown.
    pub fn close_aircraft_dropdown(&mut self) {
        self.ui_state.aircraft_dropdown_open = false;
    }

    /// Opens the departure airport dropdown.
    pub fn open_departure_dropdown(&mut self) {
        self.ui_state.departure_dropdown_open = true;
    }

    /// Closes the departure airport dropdown.
    pub fn close_departure_dropdown(&mut self) {
        self.ui_state.departure_dropdown_open = false;
    }

    /// Updates the search query.
    pub fn update_search_query(&mut self, query: String) {
        self.ui_state.search_query = query;
        self.ui_state.search_should_execute = true;
    }

    /// Checks if search should be executed.
    pub fn should_execute_search(&self) -> bool {
        self.ui_state.search_should_execute
    }

    /// Marks search as executed.
    pub fn mark_search_executed(&mut self) {
        self.ui_state.search_should_execute = false;
    }

    /// Updates filtered items based on current display mode and search.
    pub fn update_filtered_items(&mut self) {
        use crate::gui::services::{
            aircraft_service, airport_service, history_service, route_service,
        };

        let query = &self.ui_state.search_query;

        // Filter items based on current display mode using clean services
        self.ui_state.filtered_items = match self.ui_state.display_mode {
            DisplayMode::RandomAirports | DisplayMode::Airports => {
                let filtered = airport_service::filter_items(self.app_state.airport_items(), query);
                filtered.into_iter().map(TableItem::Airport).collect()
            }
            DisplayMode::RandomRoutes
            | DisplayMode::NotFlownRoutes
            | DisplayMode::SpecificAircraftRoutes => {
                let filtered = route_service::filter_items(self.app_state.route_items(), query);
                filtered.into_iter().map(TableItem::Route).collect()
            }
            DisplayMode::History => {
                let filtered = history_service::filter_items(self.app_state.history_items(), query);
                filtered.into_iter().map(TableItem::History).collect()
            }
            DisplayMode::Other => {
                // Check if we're showing aircraft based on the current items
                if let Some(first_item) = self.ui_state.all_items.first() {
                    match first_item {
                        TableItem::Aircraft(_) => {
                            // Extract aircraft items from all_items
                            let aircraft_items: Vec<_> = self
                                .ui_state
                                .all_items
                                .iter()
                                .filter_map(|item| {
                                    if let TableItem::Aircraft(aircraft) = item {
                                        Some(aircraft.clone())
                                    } else {
                                        None
                                    }
                                })
                                .collect();

                            // Filter using aircraft service
                            let filtered = aircraft_service::filter_items(&aircraft_items, query);
                            filtered.into_iter().map(TableItem::Aircraft).collect()
                        }
                        _ => {
                            // For other types, just return all current items (no filtering for now)
                            self.ui_state.all_items.clone()
                        }
                    }
                } else {
                    // No items to filter
                    Vec::new()
                }
            }
        };
    }

    /// Gets the displayed items for the current view.
    pub fn get_displayed_items(&self) -> &[TableItem] {
        if self.ui_state.search_query.is_empty() {
            &self.ui_state.all_items
        } else {
            &self.ui_state.filtered_items
        }
    }

    /// Sets all items for the current display mode.
    pub fn set_all_items(&mut self, items: Vec<TableItem>) {
        self.ui_state.all_items = items;
    }

    /// Sets items with a specific display mode.
    pub fn set_items_with_display_mode(
        &mut self,
        items: Vec<TableItem>,
        display_mode: DisplayMode,
    ) {
        self.ui_state.display_mode = display_mode;
        self.ui_state.all_items = items;
        self.update_filtered_items();
    }

    /// Clears all items and sets display mode.
    pub fn clear_and_set_display_mode(&mut self, display_mode: DisplayMode) {
        self.ui_state.display_mode = display_mode;
        self.ui_state.all_items.clear();
        self.ui_state.filtered_items.clear();
    }

    /// Clears all items.
    pub fn clear_all_items(&mut self) {
        self.ui_state.all_items.clear();
        self.ui_state.filtered_items.clear();
    }

    /// Extends all items.
    pub fn extend_all_items(&mut self, items: Vec<TableItem>) {
        self.ui_state.all_items.extend(items);
        if !self.ui_state.search_query.is_empty() {
            self.update_filtered_items();
        }
    }

    /// Gets all items.
    pub fn get_all_items(&self) -> &[TableItem] {
        &self.ui_state.all_items
    }

    // Popup-related methods

    /// Shows the popup alert.
    pub fn show_alert(&self) -> bool {
        self.ui_state.show_popup
    }

    /// Gets the selected route for popup.
    pub fn get_selected_route(&self) -> Option<&ListItemRoute> {
        self.ui_state.selected_route_for_popup.as_ref()
    }

    /// Sets whether to show alert.
    pub fn set_show_alert(&mut self, show: bool) {
        self.ui_state.show_popup = show;
    }

    /// Sets the selected route for popup.
    pub fn set_selected_route(&mut self, route: Option<ListItemRoute>) {
        self.ui_state.selected_route_for_popup = route;
    }

    // Business logic methods that delegate to clean AppState

    /// Regenerates random routes using clean architecture.
    pub fn regenerate_random_routes(&mut self) {
        let departure_icao = self
            .ui_state
            .selected_departure_airport
            .as_ref()
            .map(|airport| airport.ICAO.as_str());

        self.app_state.regenerate_random_routes(departure_icao);
        self.update_all_items_for_current_mode();
    }

    /// Regenerates random routes for a specific departure.
    pub fn regenerate_random_routes_for_departure(&mut self, departure_icao: &str) {
        self.app_state
            .regenerate_random_routes(Some(departure_icao));
        self.update_all_items_for_current_mode();
    }

    /// Updates all items based on current display mode.
    fn update_all_items_for_current_mode(&mut self) {
        let items: Vec<TableItem> = match self.ui_state.display_mode {
            DisplayMode::RandomAirports | DisplayMode::Airports => self
                .app_state
                .airport_items()
                .iter()
                .map(|item| TableItem::Airport(item.clone()))
                .collect(),
            DisplayMode::RandomRoutes
            | DisplayMode::NotFlownRoutes
            | DisplayMode::SpecificAircraftRoutes => self
                .app_state
                .route_items()
                .iter()
                .map(|item| TableItem::Route(item.clone()))
                .collect(),
            DisplayMode::History => self
                .app_state
                .history_items()
                .iter()
                .map(|item| TableItem::History(item.clone()))
                .collect(),
            DisplayMode::Other => {
                // For "Other" mode, we could be showing aircraft, history, or other items
                // This will be set explicitly by the calling code
                Vec::new()
            }
        };

        self.ui_state.all_items = items;
        if !self.ui_state.search_query.is_empty() {
            self.update_filtered_items();
        }
    }

    /// Resets the UI state.
    pub fn reset_state(&mut self, clear_search_query: bool, update_items: bool) {
        if clear_search_query {
            self.ui_state.search_query.clear();
            self.ui_state.search_should_execute = false;
        }

        self.ui_state.aircraft_dropdown_open = false;
        self.ui_state.departure_dropdown_open = false;

        if update_items {
            self.update_all_items_for_current_mode();
        }
    }

    /// Gets random airports using clean architecture.
    pub fn get_random_airports(&self, amount: usize) -> Vec<Arc<Airport>> {
        self.app_state.get_random_airports(amount)
    }

    /// Marks a route as flown using clean architecture.
    pub fn mark_route_as_flown(
        &mut self,
        route: &ListItemRoute,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.app_state.mark_route_as_flown(route)
    }

    /// Toggles the flown status of an aircraft using clean architecture.
    pub fn toggle_aircraft_flown_status(
        &mut self,
        aircraft_id: i32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.app_state.toggle_aircraft_flown_status(aircraft_id)?;

        // If we're currently displaying aircraft, refresh the display
        if matches!(self.ui_state.display_mode, DisplayMode::Other) {
            let aircraft_items: Vec<_> = self
                .get_all_aircraft()
                .iter()
                .map(crate::gui::data::ListItemAircraft::new)
                .collect();
            let table_items: Vec<TableItem> = aircraft_items
                .into_iter()
                .map(TableItem::Aircraft)
                .collect();
            self.set_all_items(table_items);

            // Also update filtered items if there's a search query
            if !self.ui_state.search_query.is_empty() {
                self.update_filtered_items();
            }
        }

        Ok(())
    }

    // Additional methods expected by the UI components

    /// Gets whether routes are from not flown aircraft.
    pub fn routes_from_not_flown(&self) -> bool {
        matches!(self.ui_state.display_mode, DisplayMode::NotFlownRoutes)
    }

    /// Gets whether airports are random.
    pub fn airports_random(&self) -> bool {
        matches!(self.ui_state.display_mode, DisplayMode::RandomAirports)
    }

    /// Sets all aircraft (for compatibility).
    pub fn set_all_aircraft(&mut self, _aircraft: Vec<Arc<Aircraft>>) {
        // In clean architecture, aircraft data is managed by AppState
        // This method is for UI compatibility - the actual aircraft data
        // comes from the database through AppState
        log::warn!("set_all_aircraft called on clean UI - aircraft should be managed by AppState");
    }

    /// Gets access to the route generator (for compatibility).
    pub fn get_route_service(&self) -> &RouteGenerator {
        self.app_state.route_generator()
    }
}

impl eframe::App for Gui {
    /// Updates the application state and the user interface.
    ///
    /// # Arguments
    ///
    /// The main update method that renders the entire UI using clean architecture.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The egui context.
    /// * `_frame` - The eframe frame.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle popup state first
        self.handle_input(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_enabled_ui(!self.show_popup(), |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                    // Left panel with selection controls and buttons - fixed width (same as original)
                    ui.allocate_ui_with_layout(
                        egui::Vec2::new(250.0, ui.available_height()),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            // Selection controls at top of left panel
                            SelectionControls::render(self, ui);
                            ui.separator();

                            // Action buttons below selection controls
                            ActionButtons::render(self, ui);
                        },
                    );

                    ui.separator();

                    // Right panel with search and table - takes remaining space (same as original)
                    ui.vertical(|ui| {
                        SearchControls::render(self, ui);
                        ui.add_space(10.0);
                        ui.separator();
                        TableDisplay::render(self, ui);
                    });
                });
            });
        });

        // Popup handling - use component
        RoutePopup::render(self, ctx);
    }
}

impl Gui {
    /// Gets the current display mode.
    pub fn get_display_mode(&self) -> &DisplayMode {
        &self.ui_state.display_mode
    }

    /// Sets the display mode.
    pub fn set_display_mode(&mut self, mode: DisplayMode) {
        self.ui_state.display_mode = mode;
    }

    /// Updates the displayed items based on current mode.
    pub fn update_displayed_items(&mut self) {
        self.ui_state.all_items = match self.ui_state.display_mode {
            DisplayMode::RandomRoutes
            | DisplayMode::NotFlownRoutes
            | DisplayMode::SpecificAircraftRoutes => self
                .app_state
                .route_items()
                .iter()
                .map(|r| TableItem::Route(r.clone()))
                .collect(),
            DisplayMode::History => self
                .app_state
                .history_items()
                .iter()
                .map(|h| TableItem::History(h.clone()))
                .collect(),
            DisplayMode::Airports | DisplayMode::RandomAirports => self
                .app_state
                .airport_items()
                .iter()
                .map(|a| TableItem::Airport(a.clone()))
                .collect(),
            DisplayMode::Other => {
                Vec::new() // Empty for now
            }
        };

        // Update filtered items as well
        self.update_filtered_items();
    }

    /// Regenerates routes when selections change using clean architecture.
    pub fn regenerate_routes_for_selection_change(&mut self) {
        let departure_icao = self
            .ui_state
            .selected_departure_airport
            .as_ref()
            .map(|airport| airport.ICAO.as_str());

        let should_update = match self.ui_state.display_mode {
            DisplayMode::RandomRoutes | DisplayMode::SpecificAircraftRoutes => {
                if let Some(aircraft) = &self.ui_state.selected_aircraft {
                    self.app_state
                        .regenerate_routes_for_aircraft(aircraft, departure_icao);
                } else {
                    self.app_state.regenerate_random_routes(departure_icao);
                }
                true
            }
            DisplayMode::NotFlownRoutes => {
                self.app_state.regenerate_not_flown_routes(departure_icao);
                true
            }
            _ => {
                // For other modes, don't regenerate routes
                false
            }
        };

        if should_update {
            self.update_displayed_items();
        }
    }

    // Infinite scrolling methods

    /// Checks if more routes are currently being loaded
    pub fn is_loading_more_routes(&self) -> bool {
        self.ui_state.is_loading_more_routes
    }

    /// Loads more routes for infinite scrolling when user scrolls near bottom
    pub fn load_more_routes_if_needed(&mut self) {
        // Only load for route display modes and if not already loading
        if self.ui_state.is_loading_more_routes {
            return;
        }

        let is_route_mode = matches!(
            self.ui_state.display_mode,
            DisplayMode::RandomRoutes
                | DisplayMode::NotFlownRoutes
                | DisplayMode::SpecificAircraftRoutes
        );

        if !is_route_mode {
            return;
        }

        self.ui_state.is_loading_more_routes = true;

        let departure_icao = self
            .ui_state
            .selected_departure_airport
            .as_ref()
            .map(|airport| airport.ICAO.as_str());

        match self.ui_state.display_mode {
            DisplayMode::RandomRoutes | DisplayMode::SpecificAircraftRoutes => {
                if let Some(aircraft) = &self.ui_state.selected_aircraft {
                    self.app_state
                        .append_routes_for_aircraft(aircraft, departure_icao);
                } else {
                    self.app_state.append_random_routes(departure_icao);
                }
            }
            DisplayMode::NotFlownRoutes => {
                self.app_state.append_not_flown_routes(departure_icao);
            }
            _ => unreachable!("Non-route DisplayMode reached in load_more_routes_if_needed; update is_route_mode check if new route modes are added."),
        }

        self.update_displayed_items();
        self.ui_state.is_loading_more_routes = false;
    }
}
