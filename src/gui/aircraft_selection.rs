use crate::gui::ui::{Gui, TableItem};
use crate::models::Aircraft;
use eframe::egui;
use std::sync::Arc;

impl Gui<'_> {
    /// Renders the aircraft selection section.
    ///
    /// # Arguments
    ///
    /// * `ui` - The UI context.
    pub(super) fn render_aircraft_selection(&mut self, ui: &mut egui::Ui) {
        ui.label("Select specific aircraft:");

        ui.horizontal(|ui| {
            // Create a custom searchable dropdown for aircraft selection
            let button_text = self.state.selected_aircraft.as_ref().map_or_else(
                || "Search aircraft...".to_string(),
                |aircraft| format!("{} {}", aircraft.manufacturer, aircraft.variant),
            );

            let button_response = ui.button(&button_text);

            if button_response.clicked() {
                self.state.aircraft_dropdown_open = !self.state.aircraft_dropdown_open;
            }
        });

        // Show the dropdown below the buttons if open
        if self.state.aircraft_dropdown_open {
            self.render_aircraft_dropdown(ui);
        }
    }

    /// Renders the aircraft dropdown.
    ///
    /// # Arguments
    ///
    /// * `ui` - The UI context.
    fn render_aircraft_dropdown(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.set_min_width(300.0);
            ui.set_max_height(300.0);

            // Search field at the top
            ui.horizontal(|ui| {
                ui.label("🔍");
                ui.text_edit_singleline(&mut self.state.aircraft_search);
            });
            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(250.0)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());

                    // Show filtered aircraft list
                    let mut found_matches = false;
                    let mut selected_aircraft_for_routes: Option<Arc<Aircraft>> = None;
                    let search_text_lower = self.state.aircraft_search.to_lowercase();

                    for aircraft in &self.state.all_aircraft {
                        let aircraft_text =
                            format!("{} {}", aircraft.manufacturer, aircraft.variant);
                        let aircraft_text_lower = aircraft_text.to_lowercase();

                        // Filter aircraft based on search text if provided
                        if self.state.aircraft_search.is_empty()
                            || aircraft_text_lower.contains(&search_text_lower)
                        {
                            found_matches = true;

                            if ui
                                .selectable_label(
                                    self.state
                                        .selected_aircraft
                                        .as_ref()
                                        .is_some_and(|selected| Arc::ptr_eq(selected, aircraft)),
                                    &aircraft_text,
                                )
                                .clicked()
                            {
                                self.state.selected_aircraft = Some(Arc::clone(aircraft));
                                self.state.aircraft_search.clear();
                                self.state.aircraft_dropdown_open = false;
                                selected_aircraft_for_routes = Some(Arc::clone(aircraft));
                            }
                        }
                    }

                    // Generate routes immediately if an aircraft was selected
                    if let Some(aircraft) = selected_aircraft_for_routes {
                        self.state.displayed_items.clear();
                        self.state.popup_state.routes_from_not_flown = false;
                        self.state.popup_state.routes_for_specific_aircraft = true;

                        let routes = self
                            .state
                            .route_generator
                            .as_mut()
                            .unwrap()
                            .generate_routes_for_aircraft(&aircraft);

                        self.state.displayed_items.extend(
                            routes
                                .into_iter()
                                .map(|route| Arc::new(TableItem::Route(route))),
                        );
                        self.handle_search();
                    }

                    // Show "no results" message if search doesn't match anything
                    if !self.state.aircraft_search.is_empty() && !found_matches {
                        ui.label("No aircraft found");
                    }
                });
        });
    }
}
