use crate::models::Airport;
use std::sync::Arc;

/// Service for validating user input and data.
pub struct ValidationService;

impl ValidationService {
    /// Validates a departure airport ICAO code against available airports.
    ///
    /// # Arguments
    ///
    /// * `icao` - The ICAO code to validate
    /// * `airports` - Slice of available airports
    ///
    /// # Returns
    ///
    /// Returns true if the ICAO code matches an available airport.
    pub fn validate_departure_airport_icao(icao: &str, airports: &[Arc<Airport>]) -> bool {
        if icao.is_empty() {
            return true; // Empty is valid (means random departure)
        }

        let icao_upper = icao.to_uppercase();
        airports.iter().any(|airport| airport.ICAO == icao_upper)
    }
}
