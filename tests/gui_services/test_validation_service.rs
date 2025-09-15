#[cfg(test)]
mod tests {
    use flight_planner::gui::services::validation_service::ValidationService;
    use flight_planner::models::Airport;
    use std::sync::Arc;

    fn create_mock_airport(icao: &str) -> Arc<Airport> {
        Arc::new(Airport {
            ID: 0,
            ICAO: icao.to_string(),
            Name: "Test Airport".to_string(),
            Latitude: 0.0,
            Longtitude: 0.0,
            Altitude: 0,
            Country: "Test Country".to_string(),
            City: "Test City".to_string(),
            IATA: "".to_string(),
            PrimaryID: "".to_string(),
        })
    }

    #[test]
    fn test_is_valid_icao_code_valid() {
        let airports = vec![create_mock_airport("EGLL")];
        let service = ValidationService::new(&airports);
        assert!(service.is_valid_icao_code("EGLL"));
    }

    #[test]
    fn test_is_valid_icao_code_invalid() {
        let airports = vec![create_mock_airport("EGLL")];
        let service = ValidationService::new(&airports);
        assert!(!service.is_valid_icao_code("EGL"));
        assert!(!service.is_valid_icao_code("EGLLL"));
        assert!(!service.is_valid_icao_code("EGL1"));
        assert!(!service.is_valid_icao_code("EGL-"));
    }

    #[test]
    fn test_is_valid_icao_code_empty() {
        let airports = vec![];
        let service = ValidationService::new(&airports);
        assert!(!service.is_valid_icao_code(""));
    }

    #[test]
    fn test_is_valid_icao_code_lowercase() {
        let airports = vec![create_mock_airport("EGLL")];
        let service = ValidationService::new(&airports);
        assert!(!service.is_valid_icao_code("egll"));
    }

    #[test]
    fn test_is_valid_icao_code_not_in_list() {
        let airports = vec![create_mock_airport("EGLL")];
        let service = ValidationService::new(&airports);
        assert!(!service.is_valid_icao_code("KLAX"));
    }
}
