use egui::Ui;

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
    }
}
