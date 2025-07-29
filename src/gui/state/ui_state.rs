use std::sync::Arc;

use crate::models::Aircraft;

/// State for UI-specific interactions and selections.
#[derive(Default)]
pub struct UiState {
    /// Currently selected aircraft for route generation.
    selected_aircraft: Option<Arc<Aircraft>>,
    /// Search text for aircraft selection.
    aircraft_search: String,
    /// Whether the aircraft dropdown is open.
    aircraft_dropdown_open: bool,
    /// The ICAO code of the departure airport.
    departure_airport_icao: String,
    /// Cached validation result for departure airport
    departure_airport_valid: Option<bool>,
    /// Last validated departure airport ICAO to detect changes
    last_validated_departure_icao: String,
    /// Search text for departure airport selection.
    departure_airport_search: String,
    /// Whether the departure airport dropdown is open.
    departure_airport_dropdown_open: bool,
}

impl UiState {
    /// Creates a new UI state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets the currently selected aircraft for route generation.
    pub const fn get_selected_aircraft(&self) -> Option<&Arc<Aircraft>> {
        self.selected_aircraft.as_ref()
    }

    /// Gets the current aircraft search text.
    pub fn get_aircraft_search(&self) -> &str {
        &self.aircraft_search
    }

    /// Gets whether the aircraft dropdown is open.
    pub const fn is_aircraft_dropdown_open(&self) -> bool {
        self.aircraft_dropdown_open
    }

    /// Gets the current departure airport ICAO code.
    pub fn get_departure_airport_icao(&self) -> &str {
        &self.departure_airport_icao
    }

    /// Gets a mutable reference to the departure airport ICAO code.
    pub const fn get_departure_airport_icao_mut(&mut self) -> &mut String {
        &mut self.departure_airport_icao
    }

    /// Gets the departure airport validation result if available.
    pub const fn get_departure_airport_validation(&self) -> Option<bool> {
        self.departure_airport_valid
    }

    /// Gets the current departure airport search text.
    pub fn get_departure_airport_search(&self) -> &str {
        &self.departure_airport_search
    }

    /// Gets whether the departure airport dropdown is open.
    pub const fn is_departure_airport_dropdown_open(&self) -> bool {
        self.departure_airport_dropdown_open
    }

    /// Sets the selected aircraft.
    pub fn set_selected_aircraft(&mut self, aircraft: Option<Arc<Aircraft>>) {
        self.selected_aircraft = aircraft;
    }

    /// Sets the aircraft search text.
    pub fn set_aircraft_search(&mut self, search: String) {
        self.aircraft_search = search;
    }

    /// Sets whether the aircraft dropdown is open.
    pub const fn set_aircraft_dropdown_open(&mut self, open: bool) {
        self.aircraft_dropdown_open = open;
    }

    /// Sets the departure airport search text.
    pub fn set_departure_airport_search(&mut self, search: String) {
        self.departure_airport_search = search;
    }

    /// Sets whether the departure airport dropdown is open.
    pub const fn set_departure_airport_dropdown_open(&mut self, open: bool) {
        self.departure_airport_dropdown_open = open;
    }

    /// Clears the departure airport validation cache.
    pub fn clear_departure_validation_cache(&mut self) {
        self.departure_airport_valid = None;
        self.last_validated_departure_icao.clear();
    }

    /// Sets the departure airport validation result and updates the cache.
    pub fn set_departure_validation(&mut self, icao: &str, is_valid: bool) {
        self.departure_airport_valid = Some(is_valid);
        self.last_validated_departure_icao = icao.to_string();
    }

    /// Resets the UI state to default values, optionally preserving the departure airport.
    pub fn reset(&mut self, preserve_departure: bool) {
        let departure_icao = if preserve_departure {
            self.departure_airport_icao.clone()
        } else {
            String::new()
        };

        let departure_valid = if preserve_departure {
            self.departure_airport_valid
        } else {
            None
        };

        let last_validated = if preserve_departure {
            self.last_validated_departure_icao.clone()
        } else {
            String::new()
        };

        *self = Self::default();

        if preserve_departure {
            self.departure_airport_icao = departure_icao;
            self.departure_airport_valid = departure_valid;
            self.last_validated_departure_icao = last_validated;
        }
    }
}
