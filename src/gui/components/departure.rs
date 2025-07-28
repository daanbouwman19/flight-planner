use egui::{Color32, Ui};

use crate::gui::ui::Gui;

impl Gui<'_> {
    /// Renders the departure airport input field.
    ///
    /// # Arguments
    ///
    /// * `ui` - The UI context.
    pub fn render_departure_input(&mut self, ui: &mut Ui) {
        ui.label("Departure airport (ICAO):");
        
        ui.text_edit_singleline(&mut self.departure_airport_icao);
        
        // Show validation feedback
        if !self.departure_airport_icao.is_empty() {
            let icao_upper = self.departure_airport_icao.to_uppercase();
            let airport_exists = self.route_generator.all_airports
                .iter()
                .any(|airport| airport.ICAO == icao_upper);
            
            if airport_exists {
                ui.colored_label(Color32::GREEN, "✓ Valid airport");
            } else {
                ui.colored_label(Color32::RED, "✗ Airport not found");
            }
        }
    }
}
