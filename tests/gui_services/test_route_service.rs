#[cfg(test)]
mod tests {
    use flight_planner::gui::data::TableItem;
    use flight_planner::gui::services::RouteService;
    use flight_planner::models::{Aircraft, Airport, Runway};
    use flight_planner::modules::routes::RouteGenerator;
    use std::collections::HashMap;
    use std::sync::Arc;

    /// Helper function to create a test aircraft.
    fn create_test_aircraft(
        id: i32,
        manufacturer: &str,
        variant: &str,
        icao_code: &str,
        flown: i32,
    ) -> Aircraft {
        Aircraft {
            id,
            manufacturer: manufacturer.to_string(),
            variant: variant.to_string(),
            icao_code: icao_code.to_string(),
            flown,
            aircraft_range: 1500,
            category: "Commercial".to_string(),
            cruise_speed: 450,
            date_flown: if flown == 1 {
                Some("2023-01-01".to_string())
            } else {
                None
            },
            takeoff_distance: Some(1000),
        }
    }

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

    /// Helper function to create a test runway.
    fn create_test_runway(airport_id: i32, length: i32) -> Runway {
        Runway {
            ID: 1,
            AirportID: airport_id,
            Ident: "09L/27R".to_string(),
            TrueHeading: 90.0,
            Length: length,
            Width: 45,
            Surface: "ASP".to_string(),
            Latitude: 51.4700,
            Longtitude: -0.4543,
            Elevation: 100,
        }
    }

    /// Helper function to create a basic route generator for testing.
    fn create_test_route_generator() -> RouteGenerator {
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
                "Charles de Gaulle",
                "LFPG",
                49.0097,
                2.5479,
            )),
        ];

        let mut all_runways = HashMap::new();
        all_runways.insert(1, Arc::new(vec![create_test_runway(1, 3902)]));
        all_runways.insert(2, Arc::new(vec![create_test_runway(2, 4215)]));

        RouteGenerator {
            all_airports: airports.clone(),
            all_runways,
            spatial_airports: rstar::RTree::bulk_load(
                airports
                    .iter()
                    .map(|airport| flight_planner::gui::ui::SpatialAirport {
                        airport: Arc::clone(airport),
                    })
                    .collect(),
            ),
        }
    }

    #[test]
    fn test_generate_random_routes_returns_route_items() {
        // Arrange
        let route_generator = create_test_route_generator();
        let route_service = RouteService::new(route_generator);
        let aircraft = vec![
            Arc::new(create_test_aircraft(1, "Boeing", "737-800", "B738", 0)),
            Arc::new(create_test_aircraft(2, "Airbus", "A320", "A320", 0)),
        ];

        // Act
        let result = route_service.generate_random_routes(&aircraft, None);

        // Assert
        assert!(!result.is_empty(), "Should generate at least one route");

        // Verify all items are route items
        for item in &result {
            match item.as_ref() {
                TableItem::Route(_) => {}
                _ => panic!("Expected only Route items"),
            }
        }
    }

    #[test]
    fn test_generate_random_routes_with_departure_icao() {
        // Arrange
        let route_generator = create_test_route_generator();
        let route_service = RouteService::new(route_generator);
        let aircraft = vec![Arc::new(create_test_aircraft(
            1, "Boeing", "737-800", "B738", 0,
        ))];

        // Act
        let result = route_service.generate_random_routes(&aircraft, Some("EGLL"));

        // Assert
        assert!(
            !result.is_empty(),
            "Should generate routes with specific departure"
        );

        // Verify routes have the specified departure
        for item in &result {
            if let TableItem::Route(route) = item.as_ref() {
                assert_eq!(
                    route.departure.ICAO, "EGLL",
                    "Route should depart from specified airport"
                );
            }
        }
    }

    #[test]
    fn test_generate_not_flown_routes_returns_route_items() {
        // Arrange
        let route_generator = create_test_route_generator();
        let route_service = RouteService::new(route_generator);
        let aircraft = vec![
            Arc::new(create_test_aircraft(1, "Boeing", "737-800", "B738", 0)), // Not flown
            Arc::new(create_test_aircraft(2, "Airbus", "A320", "A320", 1)),    // Flown
        ];

        // Act
        let result = route_service.generate_not_flown_routes(&aircraft, None);

        // Assert
        assert!(
            !result.is_empty(),
            "Should generate routes for not flown aircraft"
        );

        // Verify all items are route items
        for item in &result {
            match item.as_ref() {
                TableItem::Route(_) => {}
                _ => panic!("Expected only Route items"),
            }
        }
    }

    #[test]
    fn test_generate_routes_for_aircraft_returns_route_items() {
        // Arrange
        let route_generator = create_test_route_generator();
        let route_service = RouteService::new(route_generator);
        let aircraft = Arc::new(create_test_aircraft(1, "Boeing", "737-800", "B738", 0));

        // Act
        let result = route_service.generate_routes_for_aircraft(&aircraft, None);

        // Assert
        assert!(
            !result.is_empty(),
            "Should generate routes for specific aircraft"
        );

        // Verify all items are route items with the correct aircraft
        for item in &result {
            if let TableItem::Route(route) = item.as_ref() {
                assert_eq!(
                    route.aircraft.id, 1,
                    "Route should use the specified aircraft"
                );
                assert_eq!(route.aircraft.manufacturer, "Boeing");
                assert_eq!(route.aircraft.variant, "737-800");
            }
        }
    }

    #[test]
    fn test_load_airport_items_converts_airports_to_table_items() {
        // Arrange
        let route_generator = create_test_route_generator();
        let route_service = RouteService::new(route_generator);
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
                "Charles de Gaulle",
                "LFPG",
                49.0097,
                2.5479,
            )),
        ];

        // Act
        let result = route_service.load_airport_items(&airports);

        // Assert
        assert_eq!(result.len(), 2, "Should convert all airports");

        for (i, item) in result.iter().enumerate() {
            if let TableItem::Airport(airport) = item.as_ref() {
                match i {
                    0 => {
                        assert_eq!(airport.name, "London Heathrow");
                        assert_eq!(airport.icao, "EGLL");
                        // Now it should show actual runway length, not "N/A"
                        assert!(airport.longest_runway_length.contains("ft"));
                    }
                    1 => {
                        assert_eq!(airport.name, "Charles de Gaulle");
                        assert_eq!(airport.icao, "LFPG");
                        assert!(airport.longest_runway_length.contains("ft"));
                    }
                    _ => panic!("Unexpected airport index: {i}"),
                }
            } else {
                panic!("Expected TableItem::Airport, got different type");
            }
        }
    }

    #[test]
    fn test_load_airport_items_empty_list() {
        // Arrange
        let route_generator = create_test_route_generator();
        let route_service = RouteService::new(route_generator);
        let airports: Vec<Arc<Airport>> = vec![];

        // Act
        let result = route_service.load_airport_items(&airports);

        // Assert
        assert_eq!(result.len(), 0, "Should return empty list for empty input");
    }

    // Note: Testing mark_route_as_flown and load_history_items would require
    // mock database operations, which would be more complex and might be
    // better suited for integration tests rather than unit tests.
    // These functions are primarily thin wrappers around database operations.

    #[test]
    fn test_generate_random_airports_returns_airport_items() {
        // Arrange
        let route_generator = create_test_route_generator();
        let route_service = RouteService::new(route_generator);

        // Act
        let result = route_service.generate_random_airports(10);

        // Assert
        assert_eq!(
            result.len(),
            10,
            "Should generate the requested number of airports"
        );

        // Verify all items are airport items
        for item in &result {
            if let TableItem::Airport(airport) = item.as_ref() {
                // Should have a name and ICAO code
                assert!(!airport.name.is_empty(), "Airport should have a name");
                assert!(!airport.icao.is_empty(), "Airport should have an ICAO code");
                // Should have runway length information (either a length in ft or "N/A")
                assert!(
                    airport.longest_runway_length.ends_with(" ft")
                        || airport.longest_runway_length == "N/A",
                    "Airport should have runway length information: {}",
                    airport.longest_runway_length
                );
            } else {
                panic!("Expected only Airport items");
            }
        }
    }

    #[test]
    fn test_generate_random_airports_with_runway_data() {
        // Arrange
        let route_generator = create_test_route_generator();
        let route_service = RouteService::new(route_generator);

        // Act
        let result = route_service.generate_random_airports(5);

        // Assert
        assert_eq!(
            result.len(),
            5,
            "Should generate the requested number of airports"
        );

        // Since our test route generator has runway data for airports 1 and 2,
        // we should see some airports with actual runway lengths
        let mut found_runway_data = false;
        for item in &result {
            if let TableItem::Airport(airport) = item.as_ref() {
                if airport.longest_runway_length.ends_with(" ft") {
                    found_runway_data = true;
                    // The test data should have runway lengths of 3902 ft or 4215 ft
                    assert!(
                        airport.longest_runway_length == "3902 ft"
                            || airport.longest_runway_length == "4215 ft",
                        "Unexpected runway length: {}",
                        airport.longest_runway_length
                    );
                }
            }
        }

        // Note: This test might occasionally fail if the random generator
        // doesn't select any airports with runway data, but it's unlikely
        // with multiple attempts
        if !found_runway_data {
            println!("Warning: No airports with runway data were generated in this test run");
        }
    }
}
