#[cfg(test)]
mod tests {
    use flight_planner::gui::services::weather_service::WeatherService;
    use std::env;

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

                    if (station == "UMII" || station == "UKLO") && e != "Station not found" {
                        panic!("Expected 'Station not found' for {}, got '{}'", station, e);
                    }

                    if e.contains("Failed to parse METAR JSON") {
                        // This is what we want to avoid/fix
                        if e.contains("EOF while parsing") {
                            // This was the specific error for MU14, now handled?
                            // If we see this, it means our fix didn't work or there's another case.
                            // But wait, I added a check for empty body.
                            // So if it fails with JSON error, it's a regression or new issue.
                            panic!("JSON Parsing failed for {}: {}", station, e);
                        }
                    }
                }
            }
            println!("--------------------------------");
        }
    }
}
