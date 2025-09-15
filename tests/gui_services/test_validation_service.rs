#[cfg(test)]
mod tests {
    use flight_planner::gui::services::validation_service::ValidationService;

    #[test]
    fn test_is_valid_icao_code_valid() {
        assert!(ValidationService::is_valid_icao_code("EGLL"));
        assert!(ValidationService::is_valid_icao_code("KLAX"));
        assert!(ValidationService::is_valid_icao_code("RJAA"));
    }

    #[test]
    fn test_is_valid_icao_code_invalid_length() {
        assert!(!ValidationService::is_valid_icao_code("EGL"));
        assert!(!ValidationService::is_valid_icao_code("EGLLL"));
    }

    #[test]
    fn test_is_valid_icao_code_invalid_characters() {
        assert!(!ValidationService::is_valid_icao_code("EGL1"));
        assert!(!ValidationService::is_valid_icao_code("EGL-"));
    }

    #[test]
    fn test_is_valid_icao_code_empty() {
        assert!(!ValidationService::is_valid_icao_code(""));
    }

    #[test]
    fn test_is_valid_icao_code_lowercase() {
        assert!(ValidationService::is_valid_icao_code("egll"));
    }
}
