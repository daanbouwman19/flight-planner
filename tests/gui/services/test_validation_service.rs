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

    struct TestCase {
        description: &'static str,
        input_code: &'static str,
        available_airports: Vec<&'static str>,
        expected: bool,
    }

    #[test]
    fn test_is_valid_icao_code_parameterized() {
        let cases = vec![
            TestCase {
                description: "Valid code in list (uppercase)",
                input_code: "EGLL",
                available_airports: vec!["EGLL", "EHAM"],
                expected: true,
            },
            TestCase {
                description: "Valid code in list (lowercase)",
                input_code: "egll",
                available_airports: vec!["EGLL"],
                expected: true,
            },
            TestCase {
                description: "Valid code in list (mixed case)",
                input_code: "eGll",
                available_airports: vec!["EGLL"],
                expected: true,
            },
            TestCase {
                description: "Code not in list",
                input_code: "KLAX",
                available_airports: vec!["EGLL"],
                expected: false,
            },
            TestCase {
                description: "Empty string",
                input_code: "",
                available_airports: vec!["EGLL"],
                expected: false,
            },
            TestCase {
                description: "Too short",
                input_code: "EGL",
                available_airports: vec!["EGLL"],
                expected: false,
            },
            TestCase {
                description: "Too long",
                input_code: "EGLLL",
                available_airports: vec!["EGLL"],
                expected: false,
            },
            TestCase {
                description: "Correct length but invalid char (still invalid if not in list)",
                input_code: "EGL1",
                available_airports: vec!["EGLL"],
                expected: false,
            },
            TestCase {
                description: "Correct length but invalid char - dash (still invalid if not in list)",
                input_code: "EGL-",
                available_airports: vec!["EGLL"],
                expected: false,
            },
            TestCase {
                description: "Surrounding whitespace (length check fails)",
                input_code: " EGLL ",
                available_airports: vec!["EGLL"],
                expected: false,
            },
            TestCase {
                description: "Unicode char making length > 4 bytes (e.g. Turkish dotted I is 2 bytes)",
                // 'İ' is 2 bytes. "EGLİ" is 3 + 2 = 5 bytes.
                // is_valid_icao_code checks bytes length.
                input_code: "EGLİ",
                available_airports: vec!["EGLİ"], // Even if airport exists
                expected: false,
            },
        ];

        for case in cases {
            // Fix: dereference the reference from the iterator
            let airports: Vec<Arc<Airport>> = case
                .available_airports
                .iter()
                .map(|&icao| create_mock_airport(icao))
                .collect();

            let service = ValidationService::new(&airports);
            let result = service.is_valid_icao_code(case.input_code);

            assert_eq!(
                result, case.expected,
                "Failed case: '{}' for input '{}'",
                case.description, case.input_code
            );
        }
    }
}
