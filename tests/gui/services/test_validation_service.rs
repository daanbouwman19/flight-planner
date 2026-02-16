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
            Elevation: 0,
            TransitionAltitude: None,
            TransitionLevel: None,
            SpeedLimit: None,
            SpeedLimitAltitude: None,
            PrimaryID: None,
        })
    }

    #[test]
    fn test_is_valid_icao_code_parameterized() {
        // (description, input_code, available_airports, expected)
        let cases = vec![
            (
                "Valid code in list (uppercase)",
                "EGLL",
                vec!["EGLL", "EHAM"],
                true,
            ),
            ("Valid code in list (lowercase)", "egll", vec!["EGLL"], true),
            (
                "Valid code in list (mixed case)",
                "eGll",
                vec!["EGLL"],
                true,
            ),
            ("Code not in list", "KLAX", vec!["EGLL"], false),
            ("Empty string", "", vec!["EGLL"], false),
            ("Too short", "EGL", vec!["EGLL"], false),
            ("Too long", "EGLLL", vec!["EGLL"], false),
            (
                "Correct length but invalid char (still invalid if not in list)",
                "EGL1",
                vec!["EGLL"],
                false,
            ),
            (
                "Correct length but invalid char - dash (still invalid if not in list)",
                "EGL-",
                vec!["EGLL"],
                false,
            ),
            (
                "Surrounding whitespace (length check fails)",
                " EGLL ",
                vec!["EGLL"],
                false,
            ),
            (
                "Unicode char making length > 4 bytes (e.g. Turkish dotted I is 2 bytes)",
                // 'İ' is 2 bytes. "EGLİ" is 3 + 2 = 5 bytes.
                // is_valid_icao_code checks bytes length.
                "EGLİ",
                vec!["EGLİ"], // Even if airport exists
                false,
            ),
        ];

        for (description, input_code, available_airports, expected) in cases {
            // Updated to use into_iter() and map(create_mock_airport)
            let airports: Vec<Arc<Airport>> = available_airports
                .into_iter()
                .map(create_mock_airport)
                .collect();

            let service = ValidationService::new(&airports);
            let result = service.is_valid_icao_code(input_code);

            assert_eq!(
                result, expected,
                "Failed case: '{}' for input '{}'",
                description, input_code
            );
        }
    }
}
