use egui::{Color32, Ui};
use std::sync::Arc;

use crate::gui::ui::Gui;
use crate::models::Airport;

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
            let is_valid = validate_departure_airport_icao(&icao, self.get_available_airports());
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

/// Pure component function for rendering departure input.
/// This function demonstrates the future direction for component separation.
/// Currently not used to avoid breaking changes, but shows the pattern.
///
/// # Arguments
///
/// * `ui` - The UI context
/// * `departure_icao` - Mutable reference to the departure ICAO string
/// * `validation_cache` - Mutable reference to the validation cache
/// * `last_validated_icao` - Mutable reference to the last validated ICAO for change detection
/// * `airports` - Slice of available airports
///
/// # Returns
///
/// Returns true if the validation state changed.
#[allow(dead_code)]
pub fn render_departure_input_component(
    ui: &mut Ui,
    departure_icao: &mut String,
    validation_cache: &mut Option<bool>,
    last_validated_icao: &mut String,
    airports: &[Arc<Airport>]
) -> bool {
    ui.label("Departure airport (ICAO):");
    
    let response = ui.text_edit_singleline(departure_icao);
    
    // Only validate if the input has changed or we don't have a cached result
    let validation_changed = if response.changed() || needs_validation_refresh(departure_icao, last_validated_icao) {
        update_departure_validation(
            departure_icao,
            validation_cache,
            last_validated_icao,
            airports
        )
    } else {
        false
    };
    
    // Show validation feedback
    render_validation_feedback(ui, departure_icao, *validation_cache);
    
    validation_changed
}

/// Pure function to render validation feedback.
///
/// # Arguments
///
/// * `ui` - The UI context
/// * `icao` - The current ICAO code
/// * `is_valid` - Optional validation result
fn render_validation_feedback(ui: &mut Ui, icao: &str, is_valid: Option<bool>) {
    if !icao.is_empty() {
        if let Some(valid) = is_valid {
            if valid {
                ui.colored_label(Color32::GREEN, "✓ Valid airport");
            } else {
                ui.colored_label(Color32::RED, "✗ Airport not found");
            }
        }
    }
}

/// Pure function to check if validation needs to be refreshed.
///
/// # Arguments
///
/// * `current_icao` - The current ICAO code
/// * `last_validated_icao` - The last validated ICAO code
///
/// # Returns
///
/// True if validation needs to be refreshed.
fn needs_validation_refresh(current_icao: &str, last_validated_icao: &str) -> bool {
    current_icao != last_validated_icao
}

/// Pure function to update departure validation with explicit state management.
///
/// # Arguments
///
/// * `icao` - The ICAO code to validate
/// * `validation_cache` - Mutable reference to the validation cache
/// * `last_validated_icao` - Mutable reference to the last validated ICAO
/// * `airports` - Slice of available airports
///
/// # Returns
///
/// True if the validation state changed.
fn update_departure_validation(
    icao: &str,
    validation_cache: &mut Option<bool>,
    last_validated_icao: &mut String,
    airports: &[Arc<Airport>]
) -> bool {
    let old_validation = *validation_cache;
    
    if icao.is_empty() {
        *validation_cache = None;
        last_validated_icao.clear();
    } else {
        let is_valid = validate_departure_airport_icao(icao, airports);
        *validation_cache = Some(is_valid);
        *last_validated_icao = icao.to_string();
    }
    
    old_validation != *validation_cache
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
