#[cfg(test)]
mod tests {
    use flight_planner::gui::services::weather_service::WeatherService;
    use std::env;

    use flight_planner::models::weather::WeatherError;

    #[test]
    fn test_problematic_airports() {
        // Load .env file
        dotenv::dotenv().ok();

        let api_key = match env::var("AVWX_API_KEY") {
            Ok(key) => key,
            Err(_) => {
                println!("Skipping test_problematic_airports: AVWX_API_KEY not set");
                return;
            }
        };

        let service = WeatherService::new(api_key);

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

                    if (station == "UMII" || station == "UKLO") && !matches!(e, WeatherError::StationNotFound) {
                        panic!("Expected StationNotFound for {}, got {:?}", station, e);
                    }
                    
                    if (station == "YNUL" || station == "HLFL" || station == "MU14") && !matches!(e, WeatherError::NoData) {
                         panic!("Expected NoData for {}, got {:?}", station, e);
                    }

                    if let WeatherError::Parse(msg) = &e
                        && msg.contains("EOF while parsing") {
                             panic!("JSON Parsing failed for {}: {}", station, msg);
                        }
                }
            }
            println!("--------------------------------");
        }
    }
}
