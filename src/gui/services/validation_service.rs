/// A service for performing validation logic.
pub struct ValidationService;

impl ValidationService {
    /// Checks if a string is a valid ICAO code.
    pub fn is_valid_icao_code(code: &str) -> bool {
        code.len() == 4 && code.chars().all(|c| c.is_ascii_alphabetic())
    }
}
