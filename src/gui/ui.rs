use crate::database::DatabasePool;
use crate::gui::components::{
    action_buttons::ActionButtons, route_popup::RoutePopup, search_controls::SearchControls,
    selection_controls::SelectionControls, table_display::TableDisplay,
};
use crate::gui::data::{ListItemRoute, TableItem};
use crate::gui::state::{
    AppState,
    popup_state::{DisplayMode, PopupState},
    search_state::SearchState,
};
use crate::models::{Aircraft, Airport};
use crate::modules::routes::RouteGenerator;
use eframe::egui::{self};
use log;
use rstar::{AABB, RTreeObject};
use std::sync::Arc;

/// The main GUI application using clean architecture.
pub struct Gui {
    /// The main application state with clean architecture.
    app_state: AppState,
    /// State for search functionality
    search_state: SearchState,
    /// State for popup dialogs
    popup_state: PopupState,
    /// State for the view
    view_state: ViewState,
}

/// UI-specific state that complements the clean AppState
pub struct ViewState {
    /// Currently selected departure airport
    selected_departure_airport: Option<Arc<Airport>>,
    /// Currently selected aircraft
    selected_aircraft: Option<Arc<Aircraft>>,
    /// Whether dropdowns are open
    aircraft_dropdown_open: bool,
    departure_dropdown_open: bool,
    /// Dropdown display counts for pagination
    aircraft_display_count: usize,
    departure_display_count: usize,
    /// All items for the current view
    all_items: Vec<Arc<TableItem>>,
    /// Infinite scrolling state
    is_loading_more_routes: bool,
    /// Separate search state for aircraft dropdown
    aircraft_search: String,
    /// Separate search state for departure airport dropdown
    departure_airport_search: String,
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
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
        let search_state = SearchState::default();
        let popup_state = PopupState::default();
        let view_state = ViewState::default();

        Ok(Gui {
            app_state,
            search_state,
            popup_state,
            view_state,
        })
    }

    // Clean architecture getter methods - these delegate to AppState

    /// Gets the current departure airport.
    pub fn get_departure_airport(&self) -> Option<&Arc<Airport>> {
        self.view_state.selected_departure_airport.as_ref()
    }

    /// Gets the current departure airport ICAO code.
    pub fn get_departure_airport_icao(&self) -> &str {
        self.view_state
            .selected_departure_airport
            .as_ref()
            .map(|airport| airport.ICAO.as_str())
            .unwrap_or("")
    }

    /// Gets the departure airport validation result if available.
    pub fn get_departure_airport_validation(&self) -> Option<bool> {
        // In clean architecture, validation happens when needed
        self.view_state
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
        self.view_state.selected_aircraft.as_ref()
    }

    /// Gets the current aircraft search query.
    pub fn get_aircraft_search(&self) -> &str {
        &self.view_state.aircraft_search
    }

    /// Gets whether the aircraft dropdown is open.
    pub fn is_aircraft_dropdown_open(&self) -> bool {
        self.view_state.aircraft_dropdown_open
    }

    /// Gets the current departure airport search query.
    pub fn get_departure_airport_search(&self) -> &str {
        &self.view_state.departure_airport_search
    }

    /// Gets whether the departure airport dropdown is open.
    pub fn is_departure_airport_dropdown_open(&self) -> bool {
        self.view_state.departure_dropdown_open
    }

    /// Sets whether the departure airport dropdown is open.
    pub fn set_departure_airport_dropdown_open(&mut self, open: bool) {
        self.view_state.departure_dropdown_open = open;
    }

    /// Gets a mutable reference to the aircraft dropdown display count.
    pub fn get_aircraft_dropdown_display_count_mut(&mut self) -> &mut usize {
        &mut self.view_state.aircraft_display_count
    }

    /// Gets a mutable reference to the departure airport dropdown display count.
    pub fn get_departure_airport_dropdown_display_count_mut(&mut self) -> &mut usize {
        &mut self.view_state.departure_display_count
    }

    /// Sets the aircraft search text.
    pub fn set_aircraft_search(&mut self, search: String) {
        self.view_state.aircraft_search = search;
    }

    /// Sets the departure airport search text.
    pub fn set_departure_airport_search(&mut self, search: String) {
        self.view_state.departure_airport_search = search;
    }

    /// Regenerates routes for departure change.
    pub fn regenerate_routes_for_departure_change(&mut self) {
        if let Some(ref airport) = self.view_state.selected_departure_airport {
            let icao = airport.ICAO.clone();
            self.regenerate_random_routes_for_departure(&icao);
        } else {
            self.regenerate_random_routes_for_departure("");
        }
    }

    /// Sets the aircraft dropdown open state.
    pub fn set_aircraft_dropdown_open(&mut self, open: bool) {
        self.view_state.aircraft_dropdown_open = open;
    }

    /// Checks if the popup/alert should be shown.
    pub fn show_popup(&self) -> bool {
        self.popup_state.show_alert()
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
        self.popup_state.set_show_alert(show);
    }

    /// Sets the selected route for popup.
    pub fn set_selected_route_for_popup(&mut self, route: Option<ListItemRoute>) {
        self.popup_state.set_selected_route(route);
    }

    /// Gets the selected route for popup.
    pub fn get_selected_route_for_popup(&self) -> Option<&ListItemRoute> {
        self.popup_state.get_selected_route()
    }

    /// Gets the current search query.
    pub fn get_search_query(&self) -> &str {
        self.search_state.get_query()
    }

    /// Gets the filtered items.
    pub fn get_filtered_items(&self) -> &[Arc<TableItem>] {
        self.search_state.get_filtered_items()
    }

    // UI state mutation methods

    /// Checks if the current display mode is a route mode.
    fn is_route_mode(&self) -> bool {
        matches!(
            self.popup_state.get_display_mode(),
            DisplayMode::RandomRoutes
                | DisplayMode::NotFlownRoutes
                | DisplayMode::SpecificAircraftRoutes
        )
    }

    /// Determines the appropriate route mode based on current selections.
    fn get_appropriate_route_mode(&self) -> DisplayMode {
        if self.view_state.selected_aircraft.is_some() {
            DisplayMode::SpecificAircraftRoutes
        } else {
            DisplayMode::RandomRoutes
        }
    }

    /// Switches to route mode if currently in a non-route mode and a selection is being made.
    fn maybe_switch_to_route_mode(&mut self, selection_being_made: bool) -> bool {
        if selection_being_made && !self.is_route_mode() {
            let new_mode = self.get_appropriate_route_mode();
            self.popup_state.set_display_mode(new_mode);
            true
        } else {
            false
        }
    }

    /// Sets the selected departure airport.
    pub fn set_departure_airport(&mut self, airport: Option<Arc<Airport>>) {
        self.maybe_switch_to_route_mode(airport.is_some());
        self.view_state.selected_departure_airport = airport;
    }

    /// Sets the selected aircraft.
    pub fn set_selected_aircraft(&mut self, aircraft: Option<Arc<Aircraft>>) {
        let aircraft_being_selected = aircraft.is_some();
        self.maybe_switch_to_route_mode(aircraft_being_selected);

        self.view_state.selected_aircraft = aircraft;

        // Handle mode transitions within route modes
        if self.is_route_mode() {
            if self.view_state.selected_aircraft.is_some() {
                // Switch to specific aircraft routes if not already there
                if !matches!(
                    self.popup_state.get_display_mode(),
                    DisplayMode::SpecificAircraftRoutes
                ) {
                    self.popup_state
                        .set_display_mode(DisplayMode::SpecificAircraftRoutes);
                }
            } else {
                // Switch to random routes if clearing aircraft from specific mode
                if matches!(
                    self.popup_state.get_display_mode(),
                    DisplayMode::SpecificAircraftRoutes
                ) {
                    self.popup_state.set_display_mode(DisplayMode::RandomRoutes);
                }
            }
        }
    }

    /// Opens the aircraft dropdown.
    pub fn open_aircraft_dropdown(&mut self) {
        self.view_state.aircraft_dropdown_open = true;
    }

    /// Closes the aircraft dropdown.
    pub fn close_aircraft_dropdown(&mut self) {
        self.view_state.aircraft_dropdown_open = false;
    }

    /// Opens the departure airport dropdown.
    pub fn open_departure_dropdown(&mut self) {
        self.view_state.departure_dropdown_open = true;
    }

    /// Closes the departure airport dropdown.
    pub fn close_departure_dropdown(&mut self) {
        self.view_state.departure_dropdown_open = false;
    }

    /// Updates the search query.
    pub fn update_search_query(&mut self, query: String) {
        self.search_state.update_query(query);
    }

    /// Checks if search should be executed.
    pub fn should_execute_search(&self) -> bool {
        self.search_state.should_execute_search()
    }

    /// Marks search as executed.
    pub fn mark_search_executed(&mut self) {
        self.search_state.set_search_pending(false);
    }

    /// Updates filtered items based on current display mode and search.
    pub fn update_filtered_items(&mut self) {
        use crate::gui::services::SearchService;

        let query = self.search_state.get_query();
        let filtered_items = SearchService::filter_items(&self.view_state.all_items, query);
        self.search_state.set_filtered_items(filtered_items);
    }

    /// Gets the displayed items for the current view.
    pub fn get_displayed_items(&self) -> &[Arc<TableItem>] {
        if self.search_state.get_query().is_empty() {
            &self.view_state.all_items
        } else {
            self.search_state.get_filtered_items()
        }
    }

    /// Sets all items for the current display mode.
    pub fn set_all_items(&mut self, items: Vec<Arc<TableItem>>) {
        self.view_state.all_items = items;
    }

    /// Sets items with a specific display mode.
    pub fn set_items_with_display_mode(
        &mut self,
        items: Vec<Arc<TableItem>>,
        display_mode: DisplayMode,
    ) {
        self.popup_state.set_display_mode(display_mode);
        self.view_state.all_items = items;
        self.update_filtered_items();
    }

    /// Clears all items and sets display mode.
    pub fn clear_and_set_display_mode(&mut self, display_mode: DisplayMode) {
        self.popup_state.set_display_mode(display_mode);
        self.view_state.all_items.clear();
        self.search_state.clear_query();
    }

    /// Clears all items.
    pub fn clear_all_items(&mut self) {
        self.view_state.all_items.clear();
        // Clear filtered items but preserve the search query
        self.search_state.set_filtered_items(Vec::new());
    }

    /// Extends all items.
    pub fn extend_all_items(&mut self, items: Vec<Arc<TableItem>>) {
        self.view_state.all_items.extend(items);
        if !self.search_state.get_query().is_empty() {
            self.update_filtered_items();
        }
    }

    /// Gets all items.
    pub fn get_all_items(&self) -> &[Arc<TableItem>] {
        &self.view_state.all_items
    }

    // Popup-related methods

    /// Shows the popup alert.
    pub fn show_alert(&self) -> bool {
        self.popup_state.show_alert()
    }

    /// Gets the selected route for popup.
    pub fn get_selected_route(&self) -> Option<&ListItemRoute> {
        self.popup_state.get_selected_route()
    }

    /// Sets whether to show alert.
    pub fn set_show_alert(&mut self, show: bool) {
        self.popup_state.set_show_alert(show);
    }

    /// Sets the selected route for popup.
    pub fn set_selected_route(&mut self, route: Option<ListItemRoute>) {
        self.popup_state.set_selected_route(route);
    }

    // Business logic methods that delegate to clean AppState

    /// Regenerates random routes using clean architecture.
    pub fn regenerate_random_routes(&mut self) {
        let departure_icao = self
            .view_state
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
        let items: Vec<Arc<TableItem>> = match self.popup_state.get_display_mode() {
            DisplayMode::RandomAirports | DisplayMode::Airports => self
                .app_state
                .airport_items()
                .iter()
                .map(|item| Arc::new(TableItem::Airport(item.clone())))
                .collect(),
            DisplayMode::RandomRoutes
            | DisplayMode::NotFlownRoutes
            | DisplayMode::SpecificAircraftRoutes => self
                .app_state
                .route_items()
                .iter()
                .map(|item| Arc::new(TableItem::Route(item.clone())))
                .collect(),
            DisplayMode::History => self
                .app_state
                .history_items()
                .iter()
                .map(|item| Arc::new(TableItem::History(item.clone())))
                .collect(),
            DisplayMode::Statistics => {
                // Statistics don't use traditional table items
                Vec::new()
            }
            DisplayMode::Other => {
                // For "Other" mode, we could be showing aircraft, history, or other items
                // This will be set explicitly by the calling code
                Vec::new()
            }
        };

        self.view_state.all_items = items;
        if !self.search_state.get_query().is_empty() {
            self.update_filtered_items();
        }
    }

    /// Resets the UI state.
    pub fn reset_state(&mut self, clear_search_query: bool, update_items: bool) {
        if clear_search_query {
            self.search_state.clear_query();
        }

        self.view_state = ViewState::default();

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
        if matches!(self.popup_state.get_display_mode(), DisplayMode::Other) {
            let aircraft_items: Vec<_> = self
                .get_all_aircraft()
                .iter()
                .map(crate::gui::data::ListItemAircraft::new)
                .collect();
            let table_items: Vec<Arc<TableItem>> = aircraft_items
                .into_iter()
                .map(|item| Arc::new(TableItem::Aircraft(item)))
                .collect();
            self.set_all_items(table_items);

            // Also update filtered items if there's a search query
            if !self.search_state.get_query().is_empty() {
                self.update_filtered_items();
            }
        }

        Ok(())
    }

    // Additional methods expected by the UI components

    /// Gets whether routes are from not flown aircraft.
    pub fn routes_from_not_flown(&self) -> bool {
        self.popup_state.routes_from_not_flown()
    }

    /// Gets whether airports are random.
    pub fn airports_random(&self) -> bool {
        self.popup_state.airports_random()
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
        self.popup_state.get_display_mode()
    }

    /// Sets the display mode.
    pub fn set_display_mode(&mut self, mode: DisplayMode) {
        self.popup_state.set_display_mode(mode);
    }

    /// Gets flight statistics using caching for improved performance.
    pub fn get_flight_statistics(
        &mut self,
    ) -> Result<crate::modules::data_operations::FlightStatistics, Box<dyn std::error::Error>> {
        self.app_state.get_flight_statistics()
    }

    /// Updates the displayed items based on current mode.
    pub fn update_displayed_items(&mut self) {
        self.view_state.all_items = match self.popup_state.get_display_mode() {
            DisplayMode::RandomRoutes
            | DisplayMode::NotFlownRoutes
            | DisplayMode::SpecificAircraftRoutes => self
                .app_state
                .route_items()
                .iter()
                .map(|r| Arc::new(TableItem::Route(r.clone())))
                .collect(),
            DisplayMode::History => self
                .app_state
                .history_items()
                .iter()
                .map(|h| Arc::new(TableItem::History(h.clone())))
                .collect(),
            DisplayMode::Airports | DisplayMode::RandomAirports => self
                .app_state
                .airport_items()
                .iter()
                .map(|a| Arc::new(TableItem::Airport(a.clone())))
                .collect(),
            DisplayMode::Statistics => {
                Vec::new() // Statistics don't use table items
            }
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
            .view_state
            .selected_departure_airport
            .as_ref()
            .map(|airport| airport.ICAO.as_str());

        let should_update = match self.popup_state.get_display_mode() {
            DisplayMode::RandomRoutes | DisplayMode::SpecificAircraftRoutes => {
                if let Some(aircraft) = &self.view_state.selected_aircraft {
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
        self.view_state.is_loading_more_routes
    }

    /// Loads more routes for infinite scrolling when user scrolls near bottom
    pub fn load_more_routes_if_needed(&mut self) {
        // Only load for route display modes and if not already loading
        if self.view_state.is_loading_more_routes {
            return;
        }

        let is_route_mode = matches!(
            self.popup_state.get_display_mode(),
            DisplayMode::RandomRoutes
                | DisplayMode::NotFlownRoutes
                | DisplayMode::SpecificAircraftRoutes
        );

        if !is_route_mode {
            return;
        }

        self.view_state.is_loading_more_routes = true;

        let departure_icao = self
            .view_state
            .selected_departure_airport
            .as_ref()
            .map(|airport| airport.ICAO.as_str());

        match self.popup_state.get_display_mode() {
            DisplayMode::RandomRoutes | DisplayMode::SpecificAircraftRoutes => {
                if let Some(aircraft) = &self.view_state.selected_aircraft {
                    self.app_state
                        .append_routes_for_aircraft(aircraft, departure_icao);
                } else {
                    self.app_state.append_random_routes(departure_icao);
                }
            }
            DisplayMode::NotFlownRoutes => {
                self.app_state.append_not_flown_routes(departure_icao);
            }
            _ => unreachable!(
                "Non-route DisplayMode reached in load_more_routes_if_needed; update is_route_mode check if new route modes are added."
            ),
        }

        self.update_displayed_items();
        self.view_state.is_loading_more_routes = false;
    }
}
