#[cfg(test)]
mod tests {
    use flight_planner::gui::services::weather_service::WeatherService;
    use flight_planner::models::weather::WeatherError;
    use httpmock::prelude::*;

    struct TestCase {
        station: &'static str,
        status: u16,
        body: Option<&'static str>,
        expected_error: Option<WeatherError>,
    }

    // Helper to setup mocks
    fn setup_mock<'a>(
        server: &'a MockServer,
        station: &str,
        status: u16,
        body: Option<&str>,
    ) -> httpmock::Mock<'a> {
        server.mock(|when, then| {
            when.method(GET).path(format!("/api/metar/{}", station));
            let response = then.status(status);
            if let Some(b) = body {
                response.header("content-type", "application/json").body(b);
            }
        })
    }

    #[test]
    fn test_weather_service_integration() {
        // Start a mock server
        let server = MockServer::start();

        let test_cases = vec![
            TestCase {
                station: "KJFK",
                status: 200,
                body: Some(r#"{
                    "meta": {"timestamp": "2023-10-27T10:51:00Z"},
                    "raw": "KJFK 271051Z 36006KT 10SM FEW250 12/04 A3026 RMK AO2 SLP245 T01220044",
                    "flight_rules": "VFR",
                    "san": "KJFK",
                    "time": {"repr": "271051Z", "dt": "2023-10-27T10:51:00Z"}
                }"#),
                expected_error: None,
            },
            TestCase {
                station: "EHAM",
                status: 200,
                body: Some(r#"{
                    "meta": {"timestamp": "2023-10-27T10:55:00Z"},
                    "raw": "EHAM 271055Z 24012KT 9999 FEW025 12/08 Q1002 NOSIG",
                    "flight_rules": "VFR",
                    "san": "EHAM",
                    "time": {"repr": "271055Z", "dt": "2023-10-27T10:55:00Z"}
                }"#),
                expected_error: None,
            },
            TestCase {
                station: "YNUL",
                status: 204,
                body: None,
                expected_error: Some(WeatherError::NoData),
            },
            TestCase {
                station: "HLFL",
                status: 204,
                body: None,
                expected_error: Some(WeatherError::NoData),
            },
            TestCase {
                station: "UMII",
                status: 400,
                body: None,
                expected_error: Some(WeatherError::StationNotFound),
            },
            TestCase {
                station: "UKLO",
                status: 400,
                body: None,
                expected_error: Some(WeatherError::StationNotFound),
            },
            TestCase {
                station: "MU14",
                status: 200,
                body: Some(""),
                expected_error: Some(WeatherError::NoData),
            },
            TestCase {
                station: "MALFORMED",
                status: 200,
                body: Some("{ invalid json "),
                // For Parse error, we only check the variant, so the message content here doesn't matter
                // as long as it's the correct variant type.
                expected_error: Some(WeatherError::Parse(String::new())),
            },
        ];

        // Setup Database (dependency)
        use diesel_migrations::MigrationHarness;
        use flight_planner::database::DatabasePool;

        let pool = DatabasePool::new(Some(":memory:"), Some(":memory:")).unwrap();
        {
            let mut conn = pool.airport_pool.get().unwrap();
            conn.run_pending_migrations(flight_planner::MIGRATIONS)
                .unwrap();
        }

        // Initialize service with Mock Server URL
        let service =
            WeatherService::new("test_api_key".to_string(), pool).with_base_url(server.base_url());

        for case in test_cases {
            println!("Testing station: {}", case.station);

            // Setup mock for this case
            let _mock = setup_mock(&server, case.station, case.status, case.body);

            let result = service.fetch_metar(case.station);

            match (result, &case.expected_error) {
                (Ok(metar), None) => {
                    // Success case
                    assert_eq!(metar.san, Some(case.station.to_string()));
                    println!("  Success: Found METAR for {}", case.station);
                }
                (Err(e), Some(expected)) => {
                    // Error case
                    match expected {
                        WeatherError::Parse(_) => {
                            // Loose check for Parse error: only verify the variant matches.
                            // We ignore the actual error message string to avoid brittleness.
                            if let WeatherError::Parse(_) = e {
                                // Variant matches, test passes
                            } else {
                                panic!("Expected Parse error for {}, got {:?}", case.station, e);
                            }
                        }
                        _ => {
                            // Strict check for other error types (NoData, StationNotFound)
                            assert_eq!(&e, expected, "Error mismatch for {}", case.station);
                        }
                    }
                }
                (Ok(_), Some(expected)) => {
                    panic!(
                        "Expected error {:?} for {}, got Success",
                        expected, case.station
                    );
                }
                (Err(e), None) => {
                    panic!("Expected success for {}, got error {:?}", case.station, e);
                }
            }
            println!("--------------------------------");
        }
    }
}
