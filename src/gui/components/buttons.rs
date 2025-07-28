use std::sync::Arc;

use egui::Ui;
use rand::prelude::*;

use crate::{
    gui::ui::{Gui, ListItemAircraft, ListItemAirport, ListItemHistory, TableItem},
    models::History,
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
            self.render_main_buttons(ui);
            ui.separator();
            self.render_departure_input(ui);
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

    /// Renders random selection buttons.
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
            if let Some(aircraft) = self.all_aircraft.choose(&mut rand::rng()) {
                let list_item_aircraft = ListItemAircraft::from_aircraft(aircraft);
                self.displayed_items = vec![Arc::new(TableItem::Aircraft(list_item_aircraft))];
                self.search_state.query.clear();
                self.selected_aircraft = None;
                self.aircraft_search.clear();
                self.aircraft_dropdown_open = false;
                self.handle_search();
            }
        }

        if ui.button("Get random airport").clicked() {
            if let Some(airport) = self.route_generator.all_airports.choose(&mut rand::rng())
            {
                let list_item_airport = ListItemAirport {
                    id: airport.ID.to_string(),
                    name: airport.Name.clone(),
                    icao: airport.ICAO.clone(),
                };

                self.displayed_items = vec![Arc::new(TableItem::Airport(list_item_airport))];
                self.search_state.query.clear();
                self.selected_aircraft = None;
                self.aircraft_search.clear();
                self.aircraft_dropdown_open = false;
                self.handle_search();
            }
        }
    }

    /// Renders list display buttons.
    ///
    /// # Arguments
    ///
    /// * `ui` - The UI context.
    fn render_list_buttons(&mut self, ui: &mut Ui) {
        if ui.button("List all airports").clicked() {
            self.displayed_items = self
                .route_generator
                .all_airports
                .iter()
                .map(|airport| {
                    Arc::new(TableItem::Airport(ListItemAirport {
                        id: airport.ID.to_string(),
                        name: airport.Name.clone(),
                        icao: airport.ICAO.clone(),
                    }))
                })
                .collect();
            self.search_state.query.clear();
            self.selected_aircraft = None;
            self.aircraft_search.clear();
            self.aircraft_dropdown_open = false;
            self.handle_search();
        }

        if ui.button("List history").clicked() {
            let history: Vec<History> = self
                .database_pool
                .get_history()
                .expect("Failed to load history");

            self.displayed_items = history
                .into_iter()
                .map(|history| {
                    // Find the aircraft by ID to get its name
                    let aircraft_name = self
                        .all_aircraft
                        .iter()
                        .find(|aircraft| aircraft.id == history.aircraft)
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

            self.search_state.query.clear();
            self.selected_aircraft = None;
            self.aircraft_search.clear();
            self.aircraft_dropdown_open = false;
            self.handle_search();
        }
    }

    /// Renders route generation buttons.
    ///
    /// # Arguments
    ///
    /// * `ui` - The UI context.
    fn render_route_buttons(&mut self, ui: &mut Ui) {
        // Check if departure airport is valid when provided
        let departure_airport_valid = if self.departure_airport_icao.is_empty() {
            true // Empty is valid (means random departure)
        } else {
            let icao_upper = self.departure_airport_icao.to_uppercase();
            self.route_generator.all_airports
                .iter()
                .any(|airport| airport.ICAO == icao_upper)
        };

        let button_enabled = departure_airport_valid;
        let button_tooltip = if !departure_airport_valid {
            "Please enter a valid departure airport ICAO code or leave empty for random"
        } else {
            ""
        };

        if ui.add_enabled(button_enabled, egui::Button::new("Random route"))
            .on_hover_text(button_tooltip)
            .clicked() {
            self.displayed_items.clear();
            self.popup_state.routes_from_not_flown = false;
            self.popup_state.routes_for_specific_aircraft = false; // Normal random routes for all aircraft

            let departure_icao = if self.departure_airport_icao.is_empty() {
                None
            } else {
                Some(self.departure_airport_icao.as_str())
            };
            let routes = self
                .route_generator
                .generate_random_routes(&self.all_aircraft, departure_icao);

            self.displayed_items.extend(
                routes
                    .into_iter()
                    .map(|route| Arc::new(TableItem::Route(route))),
            );
            self.selected_aircraft = None;
            self.aircraft_search.clear();
            self.aircraft_dropdown_open = false;
            self.handle_search();
        }

        if ui.add_enabled(button_enabled, egui::Button::new("Random route from not flown"))
            .on_hover_text(button_tooltip)
            .clicked() {
            self.displayed_items.clear();
            self.popup_state.routes_from_not_flown = true;
            self.popup_state.routes_for_specific_aircraft = false; // Normal random routes for all aircraft

            let departure_icao = if self.departure_airport_icao.is_empty() {
                None
            } else {
                Some(self.departure_airport_icao.as_str())
            };
            let routes = self
                .route_generator
                .generate_random_not_flown_aircraft_routes(&self.all_aircraft, departure_icao);

            self.displayed_items.extend(
                routes
                    .into_iter()
                    .map(|route| Arc::new(TableItem::Route(route))),
            );
            self.selected_aircraft = None;
            self.aircraft_search.clear();
            self.aircraft_dropdown_open = false;
            self.handle_search();
        }
    }

    /// Renders the aircraft selection section.
    ///
    /// # Arguments
    ///
    /// * `ui` - The UI context.
    fn render_aircraft_selection(&mut self, ui: &mut Ui) {
        ui.label("Select specific aircraft:");

        ui.horizontal(|ui| {
            // Create a custom searchable dropdown for aircraft selection
            let button_text = self.selected_aircraft.as_ref().map_or_else(
                || "Search aircraft...".to_string(),
                |aircraft| format!("{} {}", aircraft.manufacturer, aircraft.variant),
            );

            let button_response = ui.button(button_text);

            if button_response.clicked() {
                self.aircraft_dropdown_open = !self.aircraft_dropdown_open;
            }
        });

        // Show the dropdown below the buttons if open
        if self.aircraft_dropdown_open {
            self.render_aircraft_dropdown(ui);
        }
    }

    /// Renders the aircraft dropdown.
    ///
    /// # Arguments
    ///
    /// * `ui` - The UI context.
    fn render_aircraft_dropdown(&mut self, ui: &mut Ui) {
        ui.group(|ui| {
            ui.set_min_width(300.0);
            ui.set_max_height(300.0);

            // Search field at the top
            ui.horizontal(|ui| {
                ui.label("🔍");
                let search_response = ui.text_edit_singleline(&mut self.aircraft_search);

                // Auto-focus the search field when dropdown is first opened
                search_response.request_focus();
            });
            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(250.0)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());

                    // Show filtered aircraft list
                    let mut found_matches = false;
                    let mut selected_aircraft_for_routes: Option<Arc<crate::models::Aircraft>> =
                        None;
                    let search_text_lower = self.aircraft_search.to_lowercase();

                    for aircraft in &self.all_aircraft {
                        let aircraft_text =
                            format!("{} {}", aircraft.manufacturer, aircraft.variant);
                        let aircraft_text_lower = aircraft_text.to_lowercase();

                        // Filter aircraft based on search text if provided
                        if self.aircraft_search.is_empty()
                            || aircraft_text_lower.contains(&search_text_lower)
                        {
                            found_matches = true;

                            if ui
                                .selectable_label(
                                    self.selected_aircraft
                                        .as_ref()
                                        .is_some_and(|selected| Arc::ptr_eq(selected, aircraft)),
                                    aircraft_text,
                                )
                                .clicked()
                            {
                                self.selected_aircraft = Some(Arc::clone(aircraft));
                                self.aircraft_search.clear();
                                self.aircraft_dropdown_open = false;
                                selected_aircraft_for_routes = Some(Arc::clone(aircraft));
                            }
                        }
                    }

                    // Generate routes immediately if an aircraft was selected
                    if let Some(aircraft) = selected_aircraft_for_routes {
                        self.displayed_items.clear();
                        self.popup_state.routes_from_not_flown = false;
                        self.popup_state.routes_for_specific_aircraft = true;

                        let departure_icao = if self.departure_airport_icao.is_empty() {
                            None
                        } else {
                            Some(self.departure_airport_icao.as_str())
                        };
                        let routes = self
                            .route_generator
                            .generate_routes_for_aircraft(&aircraft, departure_icao);

                        self.displayed_items.extend(
                            routes
                                .into_iter()
                                .map(|route| Arc::new(TableItem::Route(route))),
                        );
                        self.handle_search();
                    }

                    // Show "no results" message if search doesn't match anything
                    if !self.aircraft_search.is_empty() && !found_matches {
                        ui.label("No aircraft found");
                    }
                });
        });
    }
}
