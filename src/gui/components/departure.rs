use egui::{Color32, Ui};
use std::sync::Arc;

use crate::gui::ui::Gui;
use crate::models::Airport;

impl Gui<'_> {
    /// Renders the departure airport input field.
    /// This method uses encapsulated access patterns to demonstrate better
    /// separation of concerns and clearer data flow.
    ///
    /// # Arguments
    ///
    /// * `ui` - The UI context.
    pub fn render_departure_input(&mut self, ui: &mut Ui) {
        ui.label("Departure airport (ICAO):");
        
        let response = ui.text_edit_singleline(&mut self.departure_airport_icao);
        
        // Only validate if the input has changed or we don't have a cached result
        if response.changed() || self.needs_departure_validation_refresh() {
            self.validate_departure_airport_encapsulated();
        }
        
        // Show validation feedback using encapsulated getter
        if !self.get_departure_airport_icao().is_empty() {
            if let Some(is_valid) = self.get_departure_airport_validation() {
                if is_valid {
                    ui.colored_label(Color32::GREEN, "✓ Valid airport");
                } else {
                    ui.colored_label(Color32::RED, "✗ Airport not found");
                }
            }
        } else {
            // Clear cache when input is empty using encapsulated method
            self.clear_departure_validation_cache();
        }
    }
    
    /// Validates the departure airport ICAO using encapsulated methods.
    /// This demonstrates how business logic can be separated from state management.
    fn validate_departure_airport_encapsulated(&mut self) {
        let icao = self.departure_airport_icao.clone(); // Clone to avoid borrowing issues
        
        if icao.is_empty() {
            self.clear_departure_validation_cache();
            return;
        }
        
        let is_valid = validate_departure_airport_icao(&icao, self.get_available_airports());
        self.set_departure_validation(&icao, is_valid);
    }
}

/// Validates a departure airport ICAO code against a list of airports.
/// This is a pure function that doesn't rely on mutable state, making it
/// easy to test and reason about independently.
///
/// # Arguments
///
/// * `icao` - The ICAO code to validate
/// * `airports` - The list of available airports
///
/// # Returns
///
/// `true` if the airport exists, `false` otherwise
fn validate_departure_airport_icao(icao: &str, airports: &[Arc<Airport>]) -> bool {
    if icao.is_empty() {
        return false;
    }
    
    let icao_upper = icao.to_uppercase();
    airports
        .iter()
        .any(|airport| airport.ICAO == icao_upper)
}
