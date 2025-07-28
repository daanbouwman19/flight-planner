use crate::gui::ui::{Gui, ListItemAircraft, ListItemAirport, ListItemHistory, TableItem};
use crate::models::History;
use crate::traits::HistoryOperations;
use eframe::egui;
use rand::prelude::*;
use std::sync::Arc;

impl Gui<'_> {
    /// Updates the UI buttons.
    ///
    /// # Arguments
    ///
    /// * `ui` - The UI context.
    pub(super) fn update_buttons(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
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
    fn render_main_buttons(&mut self, ui: &mut egui::Ui) {
        self.render_random_buttons(ui);
        self.render_list_buttons(ui);
        self.render_route_buttons(ui);
    }

    /// Renders random selection buttons.
    ///
    /// # Arguments
    ///
    /// * `ui` - The UI context.
    fn render_random_buttons(&mut self, ui: &mut egui::Ui) {
        if ui
            .button("Select random aircraft")
            .on_hover_text("Select a random aircraft from the database")
            .clicked()
        {
            if let Some(aircraft) = self.state.all_aircraft.choose(&mut rand::rngs::ThreadRng::default()) {
                let list_item_aircraft = ListItemAircraft::from_aircraft(aircraft);
                self.state.displayed_items =
                    vec![Arc::new(TableItem::Aircraft(list_item_aircraft))];
                self.state.search_state.query.clear();
                self.state.selected_aircraft = None;
                self.state.aircraft_search.clear();
                self.state.aircraft_dropdown_open = false;
                self.handle_search();
            }
        }

        if ui.button("Get random airport").clicked() {
            if let Some(airport) = self
                .state
                .route_generator
                .as_mut()
                .unwrap()
                .all_airports
                .choose(&mut rand::rngs::ThreadRng::default())
            {
                let list_item_airport = ListItemAirport {
                    id: airport.ID.to_string(),
                    name: airport.Name.clone(),
                    icao: airport.ICAO.clone(),
                };

                self.state.displayed_items =
                    vec![Arc::new(TableItem::Airport(list_item_airport))];
                self.state.search_state.query.clear();
                self.state.selected_aircraft = None;
                self.state.aircraft_search.clear();
                self.state.aircraft_dropdown_open = false;
                self.handle_search();
            }
        }
    }

    /// Renders list display buttons.
    ///
    /// # Arguments
    ///
    /// * `ui` - The UI context.
    fn render_list_buttons(&mut self, ui: &mut egui::Ui) {
        if ui.button("List all airports").clicked() {
            self.state.displayed_items = self
                .state
                .route_generator
                .as_mut()
                .unwrap()
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
            self.state.search_state.query.clear();
            self.state.selected_aircraft = None;
            self.state.aircraft_search.clear();
            self.state.aircraft_dropdown_open = false;
            self.handle_search();
        }

        if ui.button("List history").clicked() {
            let history: Vec<History> = self
                .database_pool
                .get_history()
                .expect("Failed to load history");

            self.state.displayed_items = history
                .into_iter()
                .map(|history| {
                    // Find the aircraft by ID to get its name
                    let aircraft_name = self
                        .state
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

            self.state.search_state.query.clear();
            self.state.selected_aircraft = None;
            self.state.aircraft_search.clear();
            self.state.aircraft_dropdown_open = false;
            self.handle_search();
        }
    }

    /// Renders route generation buttons.
    ///
    /// # Arguments
    ///
    /// * `ui` - The UI context.
    fn render_route_buttons(&mut self, ui: &mut egui::Ui) {
        if ui.button("Random route").clicked() {
            self.state.displayed_items.clear();
            self.state.popup_state.routes_from_not_flown = false;
            self.state.popup_state.routes_for_specific_aircraft = false; // Normal random routes for all aircraft

            let routes = self
                .state
                .route_generator
                .as_mut()
                .unwrap()
                .generate_random_routes(&self.state.all_aircraft);

            self.state.displayed_items.extend(
                routes
                    .into_iter()
                    .map(|route| Arc::new(TableItem::Route(route))),
            );
            self.state.selected_aircraft = None;
            self.state.aircraft_search.clear();
            self.state.aircraft_dropdown_open = false;
            self.handle_search();
        }

        if ui.button("Random route from not flown").clicked() {
            self.state.displayed_items.clear();
            self.state.popup_state.routes_from_not_flown = true;
            self.state.popup_state.routes_for_specific_aircraft = false; // Normal random routes for all aircraft

            let routes = self
                .state
                .route_generator
                .as_mut()
                .unwrap()
                .generate_random_not_flown_aircraft_routes(&self.state.all_aircraft);

            self.state.displayed_items.extend(
                routes
                    .into_iter()
                    .map(|route| Arc::new(TableItem::Route(route))),
            );
            self.state.selected_aircraft = None;
            self.state.aircraft_search.clear();
            self.state.aircraft_dropdown_open = false;
            self.handle_search();
        }
    }
}
