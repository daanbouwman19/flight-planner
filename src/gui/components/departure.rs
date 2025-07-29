// OLD COMPONENT IMPLEMENTATION - DISABLED FOR CLEAN ARCHITECTURE
/*
impl Gui {
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
        UnifiedSelection::render(self, ui, SelectionType::DepartureAirport);
        false // Return false since validation is handled internally now
    }

    /// Regenerates routes after departure airport changes using encapsulated state.
    pub fn regenerate_routes_for_departure_change(&mut self) {
        // Use the new generic method that properly handles empty results
        self.update_displayed_routes(true);
    }

    /// Updates the departure validation state using encapsulated methods.
    /// Returns true if the validation state changed.
    pub fn update_departure_validation_state(&mut self) -> bool {
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
}
*/
