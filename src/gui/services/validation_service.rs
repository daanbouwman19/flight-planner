use crate::models::Airport;
use std::sync::Arc;

/// A service for performing validation checks, such as verifying ICAO codes.
///
/// This service is designed to be instantiated with the necessary data (like a
/// list of all airports) to perform its validation tasks.
pub struct ValidationService<'a> {
    /// A slice of all available airports used for validation lookups.
    airports: &'a [Arc<Airport>],
}

impl<'a> ValidationService<'a> {
    /// Creates a new `ValidationService`.
    ///
    /// # Arguments
    ///
    /// * `airports` - A slice of `Arc<Airport>` that the service will use for validation.
    pub fn new(airports: &'a [Arc<Airport>]) -> Self {
        Self { airports }
    }

    /// Checks if a given string is a valid and existing ICAO code.
    ///
    /// Validation checks include verifying the code's length and ensuring it
    /// exists in the list of known airports. The check is case-insensitive.
    ///
    /// # Arguments
    ///
    /// * `code` - The ICAO code string to validate.
    ///
    /// # Returns
    ///
    /// `true` if the code is a valid ICAO code, `false` otherwise.
    pub fn is_valid_icao_code(&self, code: &str) -> bool {
        if code.len() != 4 {
            return false;
        }
        self.airports.iter().any(|a| a.ICAO == code.to_uppercase())
    }
}
