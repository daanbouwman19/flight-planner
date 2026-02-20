#[cfg(test)]
mod tests {
    use flight_planner::gui::services::weather_service::WeatherService;
    use flight_planner::models::weather::WeatherError;
    use httpmock::prelude::*;

    #[derive(Debug, Clone, Copy)]
    enum TestExpectation {
        Success,
        NoData,
        StationNotFound,
        ParseError,
    }

    struct TestCase<'a> {
        station: &'a str,
        status: u16,
        body: Option<&'a str>,
        expectation: TestExpectation,
    }

    #[test]
    fn test_weather_service_handling_of_various_response_types() {
        // Arrange: Setup Server & DB
        let server = MockServer::start();

        use diesel_migrations::MigrationHarness;
        use flight_planner::database::DatabasePool;

        let pool = DatabasePool::new(Some(":memory:"), Some(":memory:")).unwrap();
        {
            let mut conn = pool.airport_pool.get().unwrap();
            conn.run_pending_migrations(flight_planner::MIGRATIONS)
                .unwrap();
        }

        let service =
            WeatherService::new("test_api_key".to_string(), pool).with_base_url(server.base_url());

        let test_cases = vec![
            TestCase {
                station: "KJFK",
                status: 200,
                body: Some(
                    r#"{
                    "meta": {"timestamp": "2023-10-27T10:51:00Z"},
                    "raw": "KJFK 271051Z 36006KT 10SM FEW250 12/04 A3026 RMK AO2 SLP245 T01220044",
                    "flight_rules": "VFR",
                    "san": "KJFK",
                    "time": {"repr": "271051Z", "dt": "2023-10-27T10:51:00Z"}
                }"#,
                ),
                expectation: TestExpectation::Success,
            },
            TestCase {
                station: "EHAM",
                status: 200,
                body: Some(
                    r#"{
                    "meta": {"timestamp": "2023-10-27T10:55:00Z"},
                    "raw": "EHAM 271055Z 24012KT 9999 FEW025 12/08 Q1002 NOSIG",
                    "flight_rules": "VFR",
                    "san": "EHAM",
                    "time": {"repr": "271055Z", "dt": "2023-10-27T10:55:00Z"}
                }"#,
                ),
                expectation: TestExpectation::Success,
            },
            TestCase {
                station: "YNUL",
                status: 204,
                body: None,
                expectation: TestExpectation::NoData,
            },
            TestCase {
                station: "HLFL",
                status: 204,
                body: None,
                expectation: TestExpectation::NoData,
            },
            TestCase {
                station: "UMII",
                status: 400,
                body: None,
                expectation: TestExpectation::StationNotFound,
            },
            TestCase {
                station: "UKLO",
                status: 400,
                body: None,
                expectation: TestExpectation::StationNotFound,
            },
            TestCase {
                station: "MU14",
                status: 200,
                body: Some(""),
                expectation: TestExpectation::NoData,
            },
            TestCase {
                station: "MALFORMED",
                status: 200,
                body: Some("{ invalid json "),
                expectation: TestExpectation::ParseError,
            },
        ];

        for case in test_cases {
            println!("Testing station: {}", case.station);

            // Arrange: Setup Mock
            let _mock = server.mock(|when, then| {
                when.method(GET)
                    .path(format!("/api/metar/{}", case.station));
                let response = then.status(case.status);
                if let Some(b) = case.body {
                    response.header("content-type", "application/json").body(b);
                }
            });

            // Act
            let result = service.fetch_metar(case.station);

            // Assert
            match case.expectation {
                TestExpectation::Success => {
                    assert!(
                        result.is_ok(),
                        "Expected success for {}, got {:?}",
                        case.station,
                        result.err()
                    );
                    let metar = result.unwrap();
                    if let Some(raw) = metar.raw {
                        println!("  Raw: {}", raw);
                    }
                }
                TestExpectation::NoData => {
                    assert!(
                        matches!(result, Err(WeatherError::NoData)),
                        "Expected NoData for {}, got {:?}",
                        case.station,
                        result
                    );
                }
                TestExpectation::StationNotFound => {
                    assert!(
                        matches!(result, Err(WeatherError::StationNotFound)),
                        "Expected StationNotFound for {}, got {:?}",
                        case.station,
                        result
                    );
                }
                TestExpectation::ParseError => {
                    assert!(
                        matches!(result, Err(WeatherError::Parse(_))),
                        "Expected Parse error for {}, got {:?}",
                        case.station,
                        result
                    );
                }
            }
        }
    }
}
