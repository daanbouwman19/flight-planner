#[cfg(test)]
mod tests {
    use flight_planner::gui::services::weather_service::WeatherService;
    use flight_planner::models::weather::WeatherError;
    use httpmock::prelude::*;

    #[test]
    fn test_problematic_airports() {
        // Start a mock server
        let server = MockServer::start();

        // Mock KJFK (Success)
        let _kjfk_mock = server.mock(|when, then| {
            when.method(GET).path("/api/metar/KJFK");
            then.status(200)
                .header("content-type", "application/json")
                .body(
                    r#"{
                    "meta": {"timestamp": "2023-10-27T10:51:00Z"},
                    "raw": "KJFK 271051Z 36006KT 10SM FEW250 12/04 A3026 RMK AO2 SLP245 T01220044",
                    "flight_rules": "VFR",
                    "san": "KJFK",
                    "time": {"repr": "271051Z", "dt": "2023-10-27T10:51:00Z"}
                }"#,
                );
        });

        // Mock EHAM (Success)
        let _eham_mock = server.mock(|when, then| {
            when.method(GET).path("/api/metar/EHAM");
            then.status(200)
                .header("content-type", "application/json")
                .body(
                    r#"{
                    "meta": {"timestamp": "2023-10-27T10:55:00Z"},
                    "raw": "EHAM 271055Z 24012KT 9999 FEW025 12/08 Q1002 NOSIG",
                    "flight_rules": "VFR",
                    "san": "EHAM",
                    "time": {"repr": "271055Z", "dt": "2023-10-27T10:55:00Z"}
                }"#,
                );
        });

        // Mock YNUL (No Content / No Data)
        let _ynul_mock = server.mock(|when, then| {
            when.method(GET).path("/api/metar/YNUL");
            then.status(204);
        });

        // Mock UMII (Station Not Found - 400)
        let _umii_mock = server.mock(|when, then| {
            when.method(GET).path("/api/metar/UMII");
            then.status(400);
        });

        // Mock UKLO (Station Not Found - 400)
        let _uklo_mock = server.mock(|when, then| {
            when.method(GET).path("/api/metar/UKLO");
            then.status(400);
        });

        // Mock HLFL (No Data - 204)
        let _hlfl_mock = server.mock(|when, then| {
            when.method(GET).path("/api/metar/HLFL");
            then.status(204);
        });

        // Mock MU14 (Empty Body - Parse Error/NoData)
        // If the API returns 200 but empty body, our service might handle it as NoData or Parse error depending on implementation.
        // The original test comment said "Empty response (handled)".
        // Let's mock it as an empty JSON body or just empty string.
        let _mu14_mock = server.mock(|when, then| {
            when.method(GET).path("/api/metar/MU14");
            then.status(200).body("");
        });

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
            "KJFK", // Should work
            "EHAM", // Should work
            "YNUL", // No METAR data
            "UMII", // Station not found (400)
            "UKLO", // Station not found (400)
            "HLFL", // No METAR data
            "MU14", // Empty response (handled)
        ];

        for station in airports {
            println!("Testing station: {}", station);
            let result = service.fetch_metar(station);

            match result {
                Ok(metar) => {
                    println!("  Success: Found METAR for {}", station);
                    // Basic validation
                    if let Some(raw) = metar.raw {
                        println!("  Raw: {}", raw);
                    }
                }
                Err(e) => {
                    println!("  Error for {}: {}", station, e);
                    // We expect certain errors for certain airports

                    if (station == "UMII" || station == "UKLO")
                        && !matches!(e, WeatherError::StationNotFound)
                    {
                        panic!("Expected StationNotFound for {}, got {:?}", station, e);
                    }

                    if (station == "YNUL" || station == "HLFL" || station == "MU14")
                        && !matches!(e, WeatherError::NoData)
                    {
                        panic!("Expected NoData for {}, got {:?}", station, e);
                    }

                    if let WeatherError::Parse(msg) = &e
                        && msg.contains("EOF while parsing")
                    {
                        // This might happen for MU14 if it returns empty body and we try to parse it
                        // But wait, the service checks `body.trim().is_empty()` and returns NoData.
                        // So MU14 should return NoData if body is empty.
                        // If it returns invalid JSON, it would be Parse error.
                        // Let's stick to the logic: if it fails with Parse, we check the message.
                        // But for MU14 we mocked empty body, so it should be NoData.
                        // If we wanted to test Parse error, we should return invalid JSON.
                        // The original test had a check for this, so maybe it was hitting a case where it wasn't empty but invalid?
                        // For now, let's assume NoData for MU14 as per the check above.
                        // If this specific panic triggers, we know why.
                        panic!("JSON Parsing failed for {}: {}", station, msg);
                    }
                }
            }
            println!("--------------------------------");
        }
    }
}
