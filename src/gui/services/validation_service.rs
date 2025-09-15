use crate::models::Airport;
use std::sync::Arc;

/// A service for performing validation logic.
pub struct ValidationService<'a> {
    airports: &'a [Arc<Airport>],
}

impl<'a> ValidationService<'a> {
    /// Creates a new `ValidationService`.
    pub fn new(airports: &'a [Arc<Airport>]) -> Self {
        Self { airports }
    }

    /// Checks if a string is a valid ICAO code.
    pub fn is_valid_icao_code(&self, code: &str) -> bool {
        if code.len() != 4 {
            return false;
        }
        self.airports.iter().any(|a| a.ICAO == code)
    }
}
