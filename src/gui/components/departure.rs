use egui::{Color32, Ui};
use rand::prelude::*;

use crate::gui::services::ValidationService;
use crate::gui::ui::Gui;

impl Gui<'_> {
    /// Renders the departure airport input field with dropdown search functionality.
    /// This method demonstrates better separation of concerns by using getter/setter methods
    /// and pure business logic functions.
    ///
    /// # Arguments
    ///
    /// * `ui` - The UI context.
    ///
    /// # Returns
    ///
    /// Returns true if the departure validation state changed.
    pub fn render_departure_input(&mut self, ui: &mut Ui) -> bool {
        ui.label("Departure airport:");

        let validation_changed = ui
            .horizontal(|ui| {
                // Button to show selected airport or prompt for selection
                let button_text = if self.get_departure_airport_icao().is_empty() {
                    "🔀 No specific departure".to_string()
                } else {
                    // Try to find the airport name to show in the button
                    let airport_name = self
                        .get_available_airports()
                        .iter()
                        .find(|airport| airport.ICAO == self.get_departure_airport_icao())
                        .map_or_else(
                            || self.get_departure_airport_icao().to_string(),
                            |airport| format!("{} ({})", airport.Name, airport.ICAO),
                        );
                    airport_name
                };

                let button_response = ui.button(button_text);

                if button_response.clicked() {
                    self.toggle_departure_airport_dropdown();
                }

                false
            })
            .inner;

        // Show the dropdown below the buttons if open
        let dropdown_validation_changed = if self.is_departure_airport_dropdown_open() {
            self.render_departure_airport_dropdown(ui)
        } else {
            false
        };

        // Show validation feedback
        self.render_departure_validation_feedback(ui);

        validation_changed || dropdown_validation_changed
    }

    /// Toggles the departure airport dropdown state.
    const fn toggle_departure_airport_dropdown(&mut self) {
        let is_open = self.is_departure_airport_dropdown_open();
        self.set_departure_airport_dropdown_open(!is_open);
    }

    /// Renders the departure airport dropdown with search functionality.
    ///
    /// # Arguments
    ///
    /// * `ui` - The UI context.
    ///
    /// # Returns
    ///
    /// Returns true if validation state changed.
    fn render_departure_airport_dropdown(&mut self, ui: &mut Ui) -> bool {
        let mut validation_changed = false;

        ui.group(|ui| {
            ui.set_min_width(400.0);
            ui.set_max_height(300.0);

            // Search field at the top
            ui.horizontal(|ui| {
                ui.label("🔍");
                let mut current_search = self.get_departure_airport_search().to_string();
                let search_response = ui.add(
                    egui::TextEdit::singleline(&mut current_search)
                        .hint_text("Search by name or ICAO (e.g. 'London' or 'EGLL')")
                        .desired_width(ui.available_width() - 30.0),
                );

                // Update search text if changed
                if search_response.changed() {
                    self.set_departure_airport_search(current_search);
                }

                // Auto-focus the search field when dropdown is first opened
                search_response.request_focus();
            });
            ui.separator();

            // Render the filtered airport list
            validation_changed = self.render_departure_airport_list(ui);
        });

        validation_changed
    }

    /// Renders the filtered departure airport list.
    ///
    /// # Arguments
    ///
    /// * `ui` - The UI context.
    ///
    /// # Returns
    ///
    /// Returns true if validation state changed.
    fn render_departure_airport_list(&mut self, ui: &mut Ui) -> bool {
        const MAX_RESULTS: usize = 50; // Limit results for performance

        let mut selected_airport_icao: Option<String> = None;
        let search_text = self.get_departure_airport_search();
        let search_text_lower = search_text.to_lowercase();
        let current_search_empty = search_text.is_empty();

        egui::ScrollArea::vertical()
            .max_height(250.0)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                ui.set_width(ui.available_width());

                // Always show option for random departure at the top
                let random_button_text = "🎲 Pick random departure airport";
                if ui.selectable_label(false, random_button_text).clicked() {
                    // Pick a random airport from available airports
                    if let Some(random_airport) =
                        self.get_available_airports().choose(&mut rand::rng())
                    {
                        selected_airport_icao = Some(random_airport.ICAO.clone());
                    }
                }

                // Option for no specific departure (system will pick randomly during route generation)
                let is_no_departure_selected = self.get_departure_airport_icao().is_empty();
                if ui
                    .selectable_label(
                        is_no_departure_selected,
                        "🔀 No specific departure (system picks randomly)",
                    )
                    .clicked()
                {
                    selected_airport_icao = Some(String::new());
                }

                ui.separator();

                // If search is empty, show a helpful message instead of all airports
                if current_search_empty {
                    ui.label("💡 Type to search for airports by name or ICAO code");
                    ui.label("   Examples: 'London', 'EGLL', 'JFK', 'Amsterdam'");
                } else if search_text.len() < 2 {
                    ui.label("💡 Type at least 2 characters to search");
                } else {
                    // Only search when we have enough characters
                    let mut found_matches = false;
                    let mut match_count = 0;

                    // Search through airports efficiently
                    for airport in self.get_available_airports() {
                        if match_count >= MAX_RESULTS {
                            break;
                        }

                        let name_matches = airport.Name.to_lowercase().contains(&search_text_lower);
                        let icao_matches = airport.ICAO.to_lowercase().contains(&search_text_lower);

                        if name_matches || icao_matches {
                            found_matches = true;
                            match_count += 1;

                            let display_text = format!("{} ({})", airport.Name, airport.ICAO);
                            let is_selected = self.get_departure_airport_icao() == airport.ICAO;

                            if ui.selectable_label(is_selected, display_text).clicked() {
                                selected_airport_icao = Some(airport.ICAO.clone());
                            }
                        }
                    }

                    // Show helpful messages based on search results
                    if !found_matches {
                        ui.label("🔍 No airports found");
                        ui.label("   Try different search terms");
                    } else if match_count >= MAX_RESULTS {
                        ui.separator();
                        ui.label(format!(
                            "📄 Showing first {MAX_RESULTS} results - refine search for more specific results"
                        ));
                    }
                }
            });

        // Handle airport selection after the UI borrowing is done
        selected_airport_icao.is_some_and(|icao| {
            self.handle_departure_airport_selection(&icao);
            true // Validation state changed
        })
    }

    /// Handles departure airport selection and state updates.
    ///
    /// # Arguments
    ///
    /// * `icao` - The selected airport ICAO code.
    fn handle_departure_airport_selection(&mut self, icao: &str) {
        *self.get_departure_airport_icao_mut() = icao.to_string();
        self.set_departure_airport_search(String::new());
        self.set_departure_airport_dropdown_open(false);

        // Update validation state
        self.update_departure_validation_state();
    }

    /// Updates the departure validation state using encapsulated methods.
    /// Returns true if the validation state changed.
    fn update_departure_validation_state(&mut self) -> bool {
        let old_validation = self.get_departure_airport_validation();

        if self.get_departure_airport_icao().is_empty() {
            self.clear_departure_validation_cache();
        } else {
            let icao = self.get_departure_airport_icao().to_string(); // Clone to avoid borrowing conflict
            let is_valid = ValidationService::validate_departure_airport_icao(
                &icao,
                self.get_available_airports(),
            );
            self.set_departure_validation(&icao, is_valid);
        }

        old_validation != self.get_departure_airport_validation()
    }

    /// Renders validation feedback using encapsulated state access.
    fn render_departure_validation_feedback(&self, ui: &mut Ui) {
        let icao = self.get_departure_airport_icao();
        if !icao.is_empty() {
            if let Some(is_valid) = self.get_departure_airport_validation() {
                if is_valid {
                    ui.colored_label(Color32::GREEN, "✓ Valid airport");
                } else {
                    ui.colored_label(Color32::RED, "✗ Airport not found");
                }
            }
        }
    }
}
