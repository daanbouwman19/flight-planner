#[cfg(test)]
mod tests {
    use flight_planner::gui::services::ValidationService;
    use flight_planner::models::Airport;
    use std::sync::Arc;

    /// Helper function to create a test airport.
    fn create_test_airport(
        id: i32,
        name: &str,
        icao: &str,
        latitude: f64,
        longitude: f64,
    ) -> Airport {
        Airport {
            ID: id,
            Name: name.to_string(),
            ICAO: icao.to_string(),
            PrimaryID: None,
            Latitude: latitude,
            Longtitude: longitude, // Note: keeping the typo as it exists in the model
            Elevation: 100,
            TransitionAltitude: None,
            TransitionLevel: None,
            SpeedLimit: None,
            SpeedLimitAltitude: None,
        }
    }

    /// Helper function to create test airports vector.
    fn create_test_airports() -> Vec<Arc<Airport>> {
        vec![
            Arc::new(create_test_airport(
                1,
                "London Heathrow",
                "EGLL",
                51.4700,
                -0.4543,
            )),
            Arc::new(create_test_airport(
                2,
                "Charles de Gaulle",
                "LFPG",
                49.0097,
                2.5479,
            )),
            Arc::new(create_test_airport(
                3,
                "JFK International",
                "KJFK",
                40.6413,
                -73.7781,
            )),
            Arc::new(create_test_airport(
                4,
                "Los Angeles International",
                "KLAX",
                33.9425,
                -118.4081,
            )),
        ]
    }

    #[test]
    fn test_validate_departure_airport_icao_empty_string_is_valid() {
        // Arrange
        let airports = create_test_airports();

        // Act
        let result = ValidationService::validate_departure_airport_icao("", &airports);

        // Assert
        assert!(result, "Empty ICAO should be valid (random departure)");
    }

    #[test]
    fn test_validate_departure_airport_icao_valid_icao_exact_match() {
        // Arrange
        let airports = create_test_airports();

        // Act
        let result = ValidationService::validate_departure_airport_icao("EGLL", &airports);

        // Assert
        assert!(result, "Valid ICAO should return true");
    }

    #[test]
    fn test_validate_departure_airport_icao_valid_icao_case_insensitive() {
        // Arrange
        let airports = create_test_airports();

        // Act
        let result_lower = ValidationService::validate_departure_airport_icao("egll", &airports);
        let result_mixed = ValidationService::validate_departure_airport_icao("EgLl", &airports);

        // Assert
        assert!(result_lower, "Lowercase ICAO should be valid");
        assert!(result_mixed, "Mixed case ICAO should be valid");
    }

    #[test]
    fn test_validate_departure_airport_icao_invalid_icao() {
        // Arrange
        let airports = create_test_airports();

        // Act
        let result = ValidationService::validate_departure_airport_icao("INVALID", &airports);

        // Assert
        assert!(!result, "Invalid ICAO should return false");
    }

    #[test]
    fn test_validate_departure_airport_icao_partial_match_invalid() {
        // Arrange
        let airports = create_test_airports();

        // Act
        let result = ValidationService::validate_departure_airport_icao("EGL", &airports);

        // Assert
        assert!(!result, "Partial ICAO match should return false");
    }

    #[test]
    fn test_validate_departure_airport_icao_extra_characters_invalid() {
        // Arrange
        let airports = create_test_airports();

        // Act
        let result = ValidationService::validate_departure_airport_icao("EGLLX", &airports);

        // Assert
        assert!(!result, "ICAO with extra characters should return false");
    }

    #[test]
    fn test_validate_departure_airport_icao_whitespace_invalid() {
        // Arrange
        let airports = create_test_airports();

        // Act
        let result_leading = ValidationService::validate_departure_airport_icao(" EGLL", &airports);
        let result_trailing =
            ValidationService::validate_departure_airport_icao("EGLL ", &airports);
        let result_both = ValidationService::validate_departure_airport_icao(" EGLL ", &airports);

        // Assert
        assert!(
            !result_leading,
            "ICAO with leading whitespace should return false"
        );
        assert!(
            !result_trailing,
            "ICAO with trailing whitespace should return false"
        );
        assert!(
            !result_both,
            "ICAO with surrounding whitespace should return false"
        );
    }

    #[test]
    fn test_validate_departure_airport_icao_empty_airports_list() {
        // Arrange
        let airports: Vec<Arc<Airport>> = vec![];

        // Act
        let result_empty = ValidationService::validate_departure_airport_icao("", &airports);
        let result_icao = ValidationService::validate_departure_airport_icao("EGLL", &airports);

        // Assert
        assert!(
            result_empty,
            "Empty string should be valid even with no airports"
        );
        assert!(!result_icao, "Any ICAO should be invalid with no airports");
    }

    #[test]
    fn test_validate_departure_airport_icao_multiple_valid_codes() {
        // Arrange
        let airports = create_test_airports();

        // Act & Assert
        assert!(ValidationService::validate_departure_airport_icao(
            "EGLL", &airports
        ));
        assert!(ValidationService::validate_departure_airport_icao(
            "LFPG", &airports
        ));
        assert!(ValidationService::validate_departure_airport_icao(
            "KJFK", &airports
        ));
        assert!(ValidationService::validate_departure_airport_icao(
            "KLAX", &airports
        ));
    }

    #[test]
    fn test_validate_departure_airport_icao_single_airport() {
        // Arrange
        let airports = vec![Arc::new(create_test_airport(
            1,
            "London Heathrow",
            "EGLL",
            51.4700,
            -0.4543,
        ))];

        // Act & Assert
        assert!(ValidationService::validate_departure_airport_icao(
            "EGLL", &airports
        ));
        assert!(!ValidationService::validate_departure_airport_icao(
            "LFPG", &airports
        ));
        assert!(ValidationService::validate_departure_airport_icao(
            "", &airports
        ));
    }

    #[test]
    fn test_validate_departure_airport_icao_duplicate_airports() {
        // Arrange - simulate duplicate airports (shouldn't happen in real data, but testing robustness)
        let airports = vec![
            Arc::new(create_test_airport(
                1,
                "London Heathrow",
                "EGLL",
                51.4700,
                -0.4543,
            )),
            Arc::new(create_test_airport(
                2,
                "London Heathrow Duplicate",
                "EGLL",
                51.4700,
                -0.4543,
            )),
        ];

        // Act
        let result = ValidationService::validate_departure_airport_icao("EGLL", &airports);

        // Assert
        assert!(result, "Should return true even with duplicate ICAO codes");
    }

    #[test]
    fn test_validate_departure_airport_icao_numbers_and_special_chars() {
        // Arrange
        let airports = create_test_airports();

        // Act & Assert
        assert!(!ValidationService::validate_departure_airport_icao(
            "123", &airports
        ));
        assert!(!ValidationService::validate_departure_airport_icao(
            "EGL!", &airports
        ));
        assert!(!ValidationService::validate_departure_airport_icao(
            "EG-LL", &airports
        ));
        assert!(!ValidationService::validate_departure_airport_icao(
            "EG.LL", &airports
        ));
    }
}
