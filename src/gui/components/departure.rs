use egui::{Color32, Ui};

use crate::gui::ui::Gui;
use crate::gui::services::ValidationService;

impl Gui<'_> {
    /// Renders the departure airport input field with improved encapsulation.
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
        ui.label("Departure airport (ICAO):");
        
        let response = ui.text_edit_singleline(self.get_departure_airport_icao_mut());
        
        // Only validate if the input has changed or we don't have a cached result
        let validation_changed = if response.changed() || self.needs_departure_validation_refresh() {
            self.update_departure_validation_state()
        } else {
            false
        };
        
        // Show validation feedback using encapsulated getter
        self.render_departure_validation_feedback(ui);
        
        validation_changed
    }
    
    /// Updates the departure validation state using encapsulated methods.
    /// Returns true if the validation state changed.
    fn update_departure_validation_state(&mut self) -> bool {
        let old_validation = self.get_departure_airport_validation();
        
        if self.get_departure_airport_icao().is_empty() {
            self.clear_departure_validation_cache();
        } else {
            let icao = self.get_departure_airport_icao().to_string(); // Clone to avoid borrowing conflict
            let is_valid = ValidationService::validate_departure_airport_icao(&icao, self.get_available_airports());
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





