#[cfg(test)]
mod tests {
    use flight_planner::gui::services::weather_service::WeatherService;
    use flight_planner::models::weather::WeatherError;
    use httpmock::prelude::*;

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
    fn test_problematic_airports() {
        // Start a mock server
        let server = MockServer::start();

        // Mock KJFK (Success)
        let _kjfk_mock = setup_mock(
            &server,
            "KJFK",
            200,
            Some(
                r#"{
            "meta": {"timestamp": "2023-10-27T10:51:00Z"},
            "raw": "KJFK 271051Z 36006KT 10SM FEW250 12/04 A3026 RMK AO2 SLP245 T01220044",
            "flight_rules": "VFR",
            "san": "KJFK",
            "time": {"repr": "271051Z", "dt": "2023-10-27T10:51:00Z"}
        }"#,
            ),
        );

        // Mock EHAM (Success)
        let _eham_mock = setup_mock(
            &server,
            "EHAM",
            200,
            Some(
                r#"{
            "meta": {"timestamp": "2023-10-27T10:55:00Z"},
            "raw": "EHAM 271055Z 24012KT 9999 FEW025 12/08 Q1002 NOSIG",
            "flight_rules": "VFR",
            "san": "EHAM",
            "time": {"repr": "271055Z", "dt": "2023-10-27T10:55:00Z"}
        }"#,
            ),
        );

        // Mock No Content / No Data cases (204)
        let _no_data_mocks: Vec<_> = ["YNUL", "HLFL"]
            .into_iter()
            .map(|station| setup_mock(&server, station, 204, None))
            .collect();

        // Mock Station Not Found cases (400)
        let _not_found_mocks: Vec<_> = ["UMII", "UKLO"]
            .into_iter()
            .map(|station| setup_mock(&server, station, 400, None))
            .collect();

        // Mock MU14 (Empty Body - NoData)
        let _mu14_mock = setup_mock(&server, "MU14", 200, Some(""));

        // Mock Malformed JSON (Parse Error)
        let _malformed_mock = setup_mock(&server, "MALFORMED", 200, Some("{ invalid json "));

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

        let airports = vec![
            "KJFK",      // Should work
            "EHAM",      // Should work
            "YNUL",      // No METAR data
            "UMII",      // Station not found (400)
            "UKLO",      // Station not found (400)
            "HLFL",      // No METAR data
            "MU14",      // Empty response (NoData)
            "MALFORMED", // Parse Error
        ];

        for station in airports {
            println!("Testing station: {}", station);
            let result = service.fetch_metar(station);

            match station {
                "KJFK" | "EHAM" => {
                    assert!(
                        result.is_ok(),
                        "Expected success for {}, got {:?}",
                        station,
                        result.err()
                    );
                    if let Ok(metar) = result {
                        println!("  Success: Found METAR for {}", station);
                        if let Some(raw) = metar.raw {
                            println!("  Raw: {}", raw);
                        }
                    }
                }
                "YNUL" | "HLFL" | "MU14" => {
                    assert!(
                        matches!(result, Err(WeatherError::NoData)),
                        "Expected NoData for {}, got {:?}",
                        station,
                        result
                    );
                }
                "UMII" | "UKLO" => {
                    assert!(
                        matches!(result, Err(WeatherError::StationNotFound)),
                        "Expected StationNotFound for {}, got {:?}",
                        station,
                        result
                    );
                }
                "MALFORMED" => {
                    if let Err(WeatherError::Parse(msg)) = &result {
                        println!("  Got expected Parse error: {}", msg);
                    } else {
                        panic!("Expected Parse error for {}, got {:?}", station, result);
                    }
                }
                _ => panic!("Unexpected station: {}", station),
            }
            println!("--------------------------------");
        }
    }
}
