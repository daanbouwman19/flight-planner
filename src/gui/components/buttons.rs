use std::sync::Arc;

use egui::Ui;

use crate::{
    gui::components::unified_selection::{SelectionType, UnifiedSelection},
    gui::services::{RouteService, ValidationService},
    gui::state::popup_state::DisplayMode,
    gui::ui::Gui,
    modules::routes::GENERATE_AMOUNT,
};

impl Gui<'_> {
    /// Updates the UI buttons.
    ///
    /// # Arguments
    ///
    /// * `ui` - The UI context.
    pub fn update_buttons(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            // Group related route generation parameters together
            ui.group(|ui| {
                ui.vertical(|ui| {
                    ui.label("Route Generation Parameters");
                    ui.separator();
                    self.render_departure_input(ui);
                    ui.add_space(5.0);
                    self.render_aircraft_selection(ui);
                });
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(5.0);

            // Action buttons
            ui.label("Actions");
            ui.separator();
            self.render_main_buttons(ui);
        });
    }

    /// Renders the main action buttons.
    ///
    /// # Arguments
    ///
    /// * `ui` - The UI context.
    fn render_main_buttons(&mut self, ui: &mut Ui) {
        self.render_random_buttons(ui);
        self.render_list_buttons(ui);
        self.render_route_buttons(ui);
    }

    /// Renders random selection buttons with improved encapsulation.
    ///
    /// # Arguments
    ///
    /// * `ui` - The UI context.
    fn render_random_buttons(&mut self, ui: &mut Ui) {
        if ui.button("Get random airports").clicked() {
            let airports = self
                .get_route_service()
                .generate_random_airports(GENERATE_AMOUNT);

            self.set_displayed_items_with_mode(airports, DisplayMode::RandomAirports);
            self.reset_ui_state_and_refresh(true);
        }
    }

    /// Renders list display buttons with improved encapsulation.
    ///
    /// # Arguments
    ///
    /// * `ui` - The UI context.
    fn render_list_buttons(&mut self, ui: &mut Ui) {
        if ui.button("List all airports").clicked() {
            let airports = self.get_available_airports().to_vec(); // Clone to avoid borrowing conflict
            let displayed_items = self.get_route_service().load_airport_items(&airports);
            self.set_displayed_items_with_mode(displayed_items, DisplayMode::Other);
            self.reset_ui_state_and_refresh(true);
        }

        if ui.button("List all aircraft").clicked() {
            let all_aircraft = self.get_all_aircraft().to_vec(); // Clone to avoid borrowing conflict
            let displayed_items = RouteService::load_aircraft_items(&all_aircraft);
            self.set_displayed_items_with_mode(displayed_items, DisplayMode::Other);
            self.reset_ui_state_and_refresh(true);
        }

        if ui.button("List history").clicked() {
            let all_aircraft = self.get_all_aircraft().to_vec(); // Clone to avoid borrowing conflict
            match RouteService::load_history_items(self.get_database_pool(), &all_aircraft) {
                Ok(displayed_items) => {
                    self.set_displayed_items_with_mode(displayed_items, DisplayMode::Other);
                    self.reset_ui_state_and_refresh(true);
                }
                Err(e) => {
                    log::error!("Failed to load history from database: {e}");
                    // Show empty list as fallback
                    self.clear_displayed_items();
                    self.reset_ui_state_and_refresh(true);
                }
            }
        }
    }

    /// Renders route generation buttons with improved encapsulation.
    ///
    /// # Arguments
    ///
    /// * `ui` - The UI context.
    fn render_route_buttons(&mut self, ui: &mut Ui) {
        // Check if departure airport is valid when provided using encapsulated state
        let departure_airport_valid = if self.get_departure_airport_icao().is_empty() {
            true // Empty is valid (means random departure)
        } else {
            // Use cached validation if available, otherwise validate
            self.get_departure_airport_validation().unwrap_or_else(|| {
                let icao = self.get_departure_airport_icao();
                let airports = self.get_available_airports();
                ValidationService::validate_departure_airport_icao(icao, airports)
            })
        };

        let button_enabled = departure_airport_valid;
        let button_tooltip = if departure_airport_valid {
            ""
        } else {
            "Please enter a valid departure airport ICAO code or leave empty for random"
        };

        if ui
            .add_enabled(button_enabled, egui::Button::new("Random route"))
            .on_hover_text(button_tooltip)
            .clicked()
        {
            self.generate_random_routes();
        }

        if ui
            .add_enabled(
                button_enabled,
                egui::Button::new("Random route from not flown"),
            )
            .on_hover_text(button_tooltip)
            .clicked()
        {
            self.generate_not_flown_routes();
        }
    }

    /// Helper method to reset display state and set specific display mode.
    /// Generates random routes using encapsulated state.
    fn generate_random_routes(&mut self) {
        self.clear_and_set_display_mode(DisplayMode::RandomRoutes);

        // Use the centralized route generation logic
        let routes = self.generate_current_routes();

        self.extend_displayed_items(routes);
        self.reset_ui_state_and_refresh(false);
    }

    /// Generates routes for not flown aircraft using encapsulated state.
    fn generate_not_flown_routes(&mut self) {
        self.clear_and_set_display_mode(DisplayMode::NotFlownRoutes);

        // Use the centralized route generation logic
        let routes = self.generate_current_routes();

        self.extend_displayed_items(routes);
        self.reset_ui_state_and_refresh(false);
    }

    /// Renders the aircraft selection section with improved encapsulation.
    ///
    /// # Arguments
    ///
    /// * `ui` - The UI context.
    fn render_aircraft_selection(&mut self, ui: &mut Ui) {
        UnifiedSelection::render(self, ui, SelectionType::Aircraft);
    }

    /// Generates routes for the selected aircraft using encapsulated state.
    ///
    /// # Arguments
    ///
    /// * `_aircraft` - The selected aircraft (unused - aircraft is retrieved from state).
    pub fn generate_routes_for_selected_aircraft(
        &mut self,
        _aircraft: &Arc<crate::models::Aircraft>,
    ) {
        self.clear_and_set_display_mode(DisplayMode::SpecificAircraftRoutes);

        // Use the centralized route generation logic
        let routes = self.generate_current_routes();

        self.extend_displayed_items(routes);
        self.handle_search();
    }
}
