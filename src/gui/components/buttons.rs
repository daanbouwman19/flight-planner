use std::sync::Arc;
use std::collections::HashMap;

use egui::Ui;
use rand::prelude::*;

use crate::{
    gui::ui::{Gui, ListItemAircraft, ListItemAirport, ListItemHistory, TableItem},
    traits::HistoryOperations,
};

impl Gui<'_> {
    /// Updates the UI buttons.
    ///
    /// # Arguments
    ///
    /// * `ui` - The UI context.
    pub fn update_buttons(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            self.render_departure_input(ui);
            ui.separator();
            self.render_main_buttons(ui);
            ui.separator();
            self.render_aircraft_selection(ui);
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
        if ui
            .button("Select random aircraft")
            .on_hover_text("Select a random aircraft from the database")
            .clicked()
        {
            if let Some(aircraft) = self.get_all_aircraft().choose(&mut rand::rng()) {
                let list_item_aircraft = ListItemAircraft::from_aircraft(aircraft);
                self.set_displayed_items(vec![Arc::new(TableItem::Aircraft(list_item_aircraft))]);
                self.reset_ui_state_and_refresh(true);
            }
        }

        if ui.button("Get random airport").clicked() {
            if let Some(airport) = self.get_available_airports().choose(&mut rand::rng())
            {
                let list_item_airport = ListItemAirport {
                    id: airport.ID.to_string(),
                    name: airport.Name.clone(),
                    icao: airport.ICAO.clone(),
                };

                self.set_displayed_items(vec![Arc::new(TableItem::Airport(list_item_airport))]);
                self.reset_ui_state_and_refresh(true);
            }
        }
    }

    /// Renders list display buttons with improved encapsulation.
    ///
    /// # Arguments
    ///
    /// * `ui` - The UI context.
    fn render_list_buttons(&mut self, ui: &mut Ui) {
        if ui.button("List all airports").clicked() {
            let displayed_items = self
                .get_available_airports()
                .iter()
                .map(|airport| {
                    Arc::new(TableItem::Airport(ListItemAirport {
                        id: airport.ID.to_string(),
                        name: airport.Name.clone(),
                        icao: airport.ICAO.clone(),
                    }))
                })
                .collect();
            self.set_displayed_items(displayed_items);
            self.reset_ui_state_and_refresh(true);
        }

        if ui.button("List history").clicked() {
            match self.get_database_pool().get_history() {
                Ok(history) => {
                    // Create a HashMap for O(1) aircraft lookups
                    let aircraft_map: HashMap<i32, &Arc<crate::models::Aircraft>> = 
                        self.get_all_aircraft()
                            .iter()
                            .map(|aircraft| (aircraft.id, aircraft))
                            .collect();

                    let displayed_items = history
                        .into_iter()
                        .map(|history| {
                            // Use HashMap for O(1) aircraft lookup
                            let aircraft_name = aircraft_map
                                .get(&history.aircraft)
                                .map_or_else(
                                    || format!("Unknown Aircraft (ID: {})", history.aircraft),
                                    |aircraft| format!("{} {}", aircraft.manufacturer, aircraft.variant),
                                );

                            Arc::new(TableItem::History(ListItemHistory {
                                id: history.id.to_string(),
                                departure_icao: history.departure_icao,
                                arrival_icao: history.arrival_icao,
                                aircraft_name,
                                date: history.date,
                            }))
                        })
                        .collect();
                    self.set_displayed_items(displayed_items);
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
                let icao_upper = self.get_departure_airport_icao().to_uppercase();
                self.get_available_airports()
                    .iter()
                    .any(|airport| airport.ICAO == icao_upper)
            })
        };

        let button_enabled = departure_airport_valid;
        let button_tooltip = if departure_airport_valid {
            ""
        } else {
            "Please enter a valid departure airport ICAO code or leave empty for random"
        };

        if ui.add_enabled(button_enabled, egui::Button::new("Random route"))
            .on_hover_text(button_tooltip)
            .clicked() {
            self.generate_random_routes();
        }

        if ui.add_enabled(button_enabled, egui::Button::new("Random route from not flown"))
            .on_hover_text(button_tooltip)
            .clicked() {
            self.generate_not_flown_routes();
        }
    }

    /// Helper method to get departure ICAO as Option<&str> for route generation.
    /// Returns None if the departure airport ICAO is empty (meaning random departure),
    /// otherwise returns Some with the ICAO string.
    fn get_departure_icao_for_routes(&self) -> Option<&str> {
        if self.get_departure_airport_icao().is_empty() {
            None
        } else {
            Some(self.get_departure_airport_icao())
        }
    }

    /// Generates random routes using encapsulated state.
    fn generate_random_routes(&mut self) {
        self.clear_displayed_items();
        self.set_routes_from_not_flown(false);
        self.set_routes_for_specific_aircraft(false);

        let departure_icao = self.get_departure_icao_for_routes();
        
        let routes = self
            .get_route_generator()
            .generate_random_routes(self.get_all_aircraft(), departure_icao);

        self.extend_displayed_items(
            routes
                .into_iter()
                .map(|route| Arc::new(TableItem::Route(route))),
        );
        self.reset_ui_state_and_refresh(false);
    }

    /// Generates routes for not flown aircraft using encapsulated state.
    fn generate_not_flown_routes(&mut self) {
        self.clear_displayed_items();
        self.set_routes_from_not_flown(true);
        self.set_routes_for_specific_aircraft(false);

        let departure_icao = self.get_departure_icao_for_routes();
        
        let routes = self
            .get_route_generator()
            .generate_random_not_flown_aircraft_routes(self.get_all_aircraft(), departure_icao);

        self.extend_displayed_items(
            routes
                .into_iter()
                .map(|route| Arc::new(TableItem::Route(route))),
        );
        self.reset_ui_state_and_refresh(false);
    }

    /// Renders the aircraft selection section with improved encapsulation.
    ///
    /// # Arguments
    ///
    /// * `ui` - The UI context.
    fn render_aircraft_selection(&mut self, ui: &mut Ui) {
        ui.label("Select specific aircraft:");

        ui.horizontal(|ui| {
            // Create a custom searchable dropdown for aircraft selection using getter
            let button_text = self.get_selected_aircraft().map_or_else(
                || "Search aircraft...".to_string(),
                |aircraft| format!("{} {}", aircraft.manufacturer, aircraft.variant),
            );

            let button_response = ui.button(button_text);

            if button_response.clicked() {
                self.toggle_aircraft_dropdown();
            }
        });

        // Show the dropdown below the buttons if open using getter
        if self.is_aircraft_dropdown_open() {
            self.render_aircraft_dropdown(ui);
        }
    }

    /// Toggles the aircraft dropdown state using encapsulated methods.
    const fn toggle_aircraft_dropdown(&mut self) {
        let is_open = self.is_aircraft_dropdown_open();
        self.set_aircraft_dropdown_open(!is_open);
    }

    /// Renders the aircraft dropdown with improved state management.
    ///
    /// # Arguments
    ///
    /// * `ui` - The UI context.
    fn render_aircraft_dropdown(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.set_min_width(300.0);
            ui.set_max_height(300.0);

            // Search field at the top using getter/setter
            ui.horizontal(|ui| {
                ui.label("🔍");
                let mut current_search = self.get_aircraft_search().to_string();
                let search_response = ui.text_edit_singleline(&mut current_search);

                // Update search text if changed
                if search_response.changed() {
                    self.set_aircraft_search(current_search);
                }

                // Auto-focus the search field when dropdown is first opened
                search_response.request_focus();
            });
            ui.separator();

            self.render_aircraft_list(ui);
        });
    }

    /// Renders the filtered aircraft list.
    ///
    /// # Arguments
    ///
    /// * `ui` - The UI context.
    fn render_aircraft_list(&mut self, ui: &mut Ui) {
        // Show filtered aircraft list
        let mut found_matches = false;
        let mut selected_aircraft_for_routes: Option<Arc<crate::models::Aircraft>> = None;
        let search_text_lower = self.get_aircraft_search().to_lowercase();
        let current_search_empty = self.get_aircraft_search().is_empty();

        egui::ScrollArea::vertical()
            .max_height(250.0)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                ui.set_width(ui.available_width());

                // Create a vector of aircraft with their display info to avoid borrowing issues
                let aircraft_list: Vec<_> = self.get_all_aircraft().iter()
                    .map(|aircraft| {
                        let display_text = format!("{} {}", aircraft.manufacturer, aircraft.variant);
                        let matches_search = current_search_empty || 
                            display_text.to_lowercase().contains(&search_text_lower);
                        let is_selected = self.get_selected_aircraft()
                            .is_some_and(|selected| Arc::ptr_eq(selected, aircraft));
                        (Arc::clone(aircraft), display_text, matches_search, is_selected)
                    })
                    .collect();

                for (aircraft, aircraft_text, matches_search, is_selected) in aircraft_list {
                    if matches_search {
                        found_matches = true;

                        if ui.selectable_label(is_selected, aircraft_text).clicked() {
                            selected_aircraft_for_routes = Some(Arc::clone(&aircraft));
                        }
                    }
                }

                // Show "no results" message if search doesn't match anything
                if !current_search_empty && !found_matches {
                    ui.label("No aircraft found");
                }
            });

        // Handle aircraft selection after the UI borrowing is done
        if let Some(aircraft) = selected_aircraft_for_routes {
            self.handle_aircraft_selection(&aircraft);
        }
    }

    /// Handles aircraft selection and state updates.
    ///
    /// # Arguments
    ///
    /// * `aircraft` - The selected aircraft.
    fn handle_aircraft_selection(&mut self, aircraft: &Arc<crate::models::Aircraft>) {
        self.set_selected_aircraft(Some(Arc::clone(aircraft)));
        self.set_aircraft_search(String::new());
        self.set_aircraft_dropdown_open(false);
        self.generate_routes_for_selected_aircraft(aircraft);
    }

    /// Generates routes for the selected aircraft using encapsulated state.
    ///
    /// # Arguments
    ///
    /// * `aircraft` - The selected aircraft.
    fn generate_routes_for_selected_aircraft(&mut self, aircraft: &Arc<crate::models::Aircraft>) {
        self.clear_displayed_items();
        self.set_routes_from_not_flown(false);
        self.set_routes_for_specific_aircraft(true);

        let departure_icao = self.get_departure_icao_for_routes();
        
        let routes = self
            .get_route_generator()
            .generate_routes_for_aircraft(aircraft, departure_icao);

        self.extend_displayed_items(
            routes
                .into_iter()
                .map(|route| Arc::new(TableItem::Route(route))),
        );
        self.handle_search();
    }
}
