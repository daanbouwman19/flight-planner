use std::sync::Arc;

use crate::models::{Aircraft, Airport};

/// State for UI-specific interactions and selections.
#[derive(Default)]
pub struct UiState {
    /// Currently selected aircraft for route generation.
    selected_aircraft: Option<Arc<Aircraft>>,
    /// Search text for aircraft selection.
    aircraft_search: String,
    /// Whether the aircraft dropdown is open.
    aircraft_dropdown_open: bool,
    /// Number of aircraft currently displayed in dropdown (for chunked loading).
    aircraft_dropdown_display_count: usize,
    /// The selected departure airport.
    departure_airport: Option<Arc<Airport>>,
    /// Cached validation result for departure airport
    departure_airport_valid: Option<bool>,
    /// Last validated departure airport ICAO to detect changes
    last_validated_departure_icao: String,
    /// Search text for departure airport selection.
    departure_airport_search: String,
    /// Whether the departure airport dropdown is open.
    departure_airport_dropdown_open: bool,
    /// Number of airports currently displayed in dropdown (for chunked loading).
    departure_airport_dropdown_display_count: usize,
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

    /// Gets the current departure airport.
    pub const fn get_departure_airport(&self) -> Option<&Arc<Airport>> {
        self.departure_airport.as_ref()
    }

    /// Gets the current departure airport ICAO code.
    pub fn get_departure_airport_icao(&self) -> &str {
        self.departure_airport
            .as_ref()
            .map_or("", |airport| &airport.ICAO)
    }

    /// Sets the selected departure airport.
    pub fn set_departure_airport(&mut self, airport: Option<Arc<Airport>>) {
        self.departure_airport = airport;
        // Clear validation cache when departure changes
        self.clear_departure_validation_cache();
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
        let departure_airport = if preserve_departure {
            self.departure_airport.clone()
        } else {
            None
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
            self.departure_airport = departure_airport;
            self.departure_airport_valid = departure_valid;
            self.last_validated_departure_icao = last_validated;
        }
    }

    /// Gets a mutable reference to the aircraft dropdown display count.
    pub const fn get_aircraft_dropdown_display_count_mut(&mut self) -> &mut usize {
        &mut self.aircraft_dropdown_display_count
    }

    /// Gets a mutable reference to the departure airport dropdown display count.
    pub const fn get_departure_airport_dropdown_display_count_mut(&mut self) -> &mut usize {
        &mut self.departure_airport_dropdown_display_count
    }
}
