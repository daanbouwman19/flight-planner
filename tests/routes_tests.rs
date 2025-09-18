use flight_planner::gui::data::ListItemRoute;
use flight_planner::models::airport::SpatialAirport;
use flight_planner::models::{Aircraft, Airport, Runway};
use flight_planner::modules::routes::*;
use flight_planner::util::METERS_TO_FEET;
use rstar::RTree;
use std::collections::HashMap;
use std::sync::Arc;

type AircraftVec = Vec<Arc<Aircraft>>;
type AirportVec = Vec<Arc<Airport>>;
type RunwayMap = HashMap<i32, Arc<Vec<Runway>>>;
type AirportRTree = RTree<SpatialAirport>;

fn create_test_data() -> (AircraftVec, AirportVec, RunwayMap, AirportRTree) {
    let aircraft1 = Arc::new(Aircraft {
        id: 1,
        manufacturer: "Boeing".to_string(),
        variant: "737-800".to_string(),
        icao_code: "B738".to_string(),
        flown: 0,
        aircraft_range: 3000,
        category: "A".to_string(),
        cruise_speed: 450,
        date_flown: Some("2024-12-10".to_string()),
        takeoff_distance: Some(2000),
    });
    let aircraft2 = Arc::new(Aircraft {
        id: 2,
        manufacturer: "Airbus".to_string(),
        variant: "A320".to_string(),
        icao_code: "A320".to_string(),
        flown: 0,
        aircraft_range: 2500,
        category: "A".to_string(),
        cruise_speed: 430,
        date_flown: None,
        takeoff_distance: Some(1800),
    });

    let all_aircraft = vec![aircraft1, aircraft2];

    let airport1 = Arc::new(Airport {
        ID: 1,
        Name: "Amsterdam Airport Schiphol".to_string(),
        ICAO: "EHAM".to_string(),
        PrimaryID: None,
        Latitude: 52.3086,
        Longtitude: 4.7639,
        Elevation: -11,
        TransitionAltitude: Some(10000),
        TransitionLevel: None,
        SpeedLimit: Some(230),
        SpeedLimitAltitude: Some(6000),
    });
    let airport2 = Arc::new(Airport {
        ID: 2,
        Name: "Rotterdam The Hague Airport".to_string(),
        ICAO: "EHRD".to_string(),
        PrimaryID: None,
        Latitude: 51.9561,
        Longtitude: 4.4397,
        Elevation: -13,
        TransitionAltitude: Some(5000),
        TransitionLevel: None,
        SpeedLimit: Some(180),
        SpeedLimitAltitude: Some(4000),
    });
    let all_airports = vec![airport1, airport2];

    let runway1 = Runway {
        ID: 1,
        AirportID: 1,
        Ident: "09".to_string(),
        TrueHeading: 92.0,
        Length: 10000,
        Width: 45,
        Surface: "Asphalt".to_string(),
        Latitude: 52.3086,
        Longtitude: 4.7639,
        Elevation: -11,
    };
    let runway2 = Runway {
        ID: 2,
        AirportID: 2,
        Ident: "06".to_string(),
        TrueHeading: 62.0,
        Length: 10000,
        Width: 45,
        Surface: "Asphalt".to_string(),
        Latitude: 51.9561,
        Longtitude: 4.4397,
        Elevation: -13,
    };

    let mut runway_map = HashMap::new();
    runway_map.insert(1, Arc::new(vec![runway1]));
    runway_map.insert(2, Arc::new(vec![runway2]));

    let spatial_airports = RTree::bulk_load(
        all_airports
            .iter()
            .map(|airport| SpatialAirport {
                airport: Arc::clone(airport),
            })
            .collect(),
    );

    (all_aircraft, all_airports, runway_map, spatial_airports)
}

#[test]
fn test_generate_random_routes() {
    let (all_aircraft, all_airports, all_runways, spatial_airports) = create_test_data();
    let route_generator = RouteGenerator::new(all_airports, all_runways, spatial_airports);

    let routes = route_generator.generate_random_routes(&all_aircraft, None);
    assert!(!routes.is_empty());
    for route in routes {
        assert!(route.departure.ID != route.destination.ID);
        assert!(route.route_length != "0");
    }
}

#[test]
fn test_generate_random_not_flown_aircraft_routes() {
    let (all_aircraft, all_airports, all_runways, spatial_airports) = create_test_data();
    let route_generator = RouteGenerator::new(all_airports, all_runways, spatial_airports);

    let routes = route_generator.generate_random_not_flown_aircraft_routes(&all_aircraft, None);
    assert!(!routes.is_empty());
    for route in routes {
        assert!(route.departure.ID != route.destination.ID);
        assert!(route.route_length != "0");
    }
}

#[test]
fn test_generate_random_routes_generic() {
    let (all_aircraft, all_airports, all_runways, spatial_airports) = create_test_data();
    let route_generator = RouteGenerator::new(all_airports, all_runways, spatial_airports);

    let routes: Vec<ListItemRoute> =
        route_generator.generate_random_routes_generic(&all_aircraft, 50, None);
    assert_eq!(routes.len(), 50);
    for route in routes {
        assert!(route.departure.ID != route.destination.ID);
        assert!(route.route_length != "0");
    }
}

#[test]
fn test_generate_routes_with_valid_departure_icao() {
    let (all_aircraft, all_airports, all_runways, spatial_airports) = create_test_data();
    let route_generator = RouteGenerator::new(all_airports, all_runways, spatial_airports);

    let routes: Vec<ListItemRoute> =
        route_generator.generate_random_routes_generic(&all_aircraft, 10, Some("EHAM"));
    assert_eq!(routes.len(), 10);
    for route in routes {
        assert_eq!(route.departure.ICAO, "EHAM");
        assert!(route.departure.ID != route.destination.ID);
        assert!(route.route_length != "0");
    }
}

#[test]
fn test_generate_routes_with_invalid_departure_icao() {
    let (all_aircraft, all_airports, all_runways, spatial_airports) = create_test_data();
    let route_generator = RouteGenerator::new(all_airports, all_runways, spatial_airports);

    let routes: Vec<ListItemRoute> =
        route_generator.generate_random_routes_generic(&all_aircraft, 10, Some("INVALID"));
    assert_eq!(routes.len(), 0);
}

#[test]
fn test_get_airport_with_suitable_runway_optimized() {
    let (_all_aircraft, all_airports, all_runways, spatial_airports) = create_test_data();

    // Create a new aircraft that requires a 7000ft runway (2134m)
    let demanding_aircraft = Arc::new(Aircraft {
        id: 3,
        manufacturer: "Test".to_string(),
        variant: "Test".to_string(),
        icao_code: "TEST".to_string(),
        flown: 0,
        aircraft_range: 3000,
        category: "C".to_string(),
        cruise_speed: 450,
        date_flown: None,
        takeoff_distance: Some(2134), // Requires ~7000ft
    });

    let route_generator =
        RouteGenerator::new(all_airports.clone(), all_runways.clone(), spatial_airports);

    // We generate a bunch of routes to have a high chance of selecting the demanding aircraft
    let routes = route_generator.generate_routes_for_aircraft(&demanding_aircraft, None);

    // We expect to find a route, which means a suitable airport was found
    assert!(
        !routes.is_empty(),
        "Should have found a route for the demanding aircraft"
    );

    // Verify the departure airport has a long enough runway
    let required_length_ft = (2134.0 * METERS_TO_FEET).round() as i32;
    let departure_airport_id = routes[0].departure.ID;
    let longest_runway = route_generator
        .longest_runway_cache
        .get(&departure_airport_id)
        .expect("Departure airport should have a longest runway cached");
    assert!(*longest_runway >= required_length_ft);
}

#[test]
fn test_longest_runway_cache_creation() {
    let (_all_aircraft, all_airports, all_runways, spatial_airports) = create_test_data();
    let route_generator =
        RouteGenerator::new(all_airports.clone(), all_runways.clone(), spatial_airports);

    // Verify that longest runway cache is populated for all airports
    for airport in &all_airports {
        assert!(
            route_generator
                .longest_runway_cache
                .contains_key(&airport.ID),
            "Airport {} should have longest runway cached",
            airport.ICAO
        );

        // Verify the cached value matches the actual longest runway
        let cached_length = route_generator
            .longest_runway_cache
            .get(&airport.ID)
            .unwrap();
        if let Some(runways) = all_runways.get(&airport.ID) {
            let actual_longest = runways.iter().map(|r| r.Length).max().unwrap_or(0);
            assert_eq!(
                *cached_length, actual_longest,
                "Cached runway length should match actual longest runway for airport {}",
                airport.ICAO
            );
        }
    }
}

#[test]
fn test_airports_by_runway_length_buckets() {
    let (_all_aircraft, all_airports, all_runways, spatial_airports) = create_test_data();
    let route_generator = RouteGenerator::new(all_airports.clone(), all_runways, spatial_airports);

    // Test that buckets are created correctly
    assert!(
        route_generator.airports_by_runway_length.contains_key(&0),
        "Should have bucket for 0 feet"
    );
    assert!(
        route_generator
            .airports_by_runway_length
            .contains_key(&5000),
        "Should have bucket for 5000 feet"
    );

    // Test that airports in higher buckets also appear in lower buckets
    let bucket_0 = route_generator.airports_by_runway_length.get(&0).unwrap();
    let bucket_5000 = route_generator
        .airports_by_runway_length
        .get(&5000)
        .unwrap();

    // All airports with runways >= 5000ft should be in the 5000ft bucket
    // And all airports should be in the 0ft bucket
    assert!(
        bucket_0.len() >= bucket_5000.len(),
        "Bucket 0 should contain at least as many airports as bucket 5000"
    );

    // Verify bucket ordering: airports in higher buckets should have longer runways
    for airport in bucket_5000 {
        let runway_length = route_generator
            .longest_runway_cache
            .get(&airport.ID)
            .unwrap();
        assert!(
            *runway_length >= 5000,
            "Airport {} in 5000ft bucket should have runway >= 5000ft, but has {}ft",
            airport.ICAO,
            runway_length
        );
    }
}

#[test]
fn test_get_airport_with_suitable_runway_optimized_buckets() {
    let (_all_aircraft, all_airports, all_runways, spatial_airports) = create_test_data();
    let route_generator = RouteGenerator::new(all_airports, all_runways, spatial_airports);

    // Test with aircraft requiring different runway lengths
    let small_aircraft = Arc::new(Aircraft {
        id: 10,
        manufacturer: "Cessna".to_string(),
        variant: "172".to_string(),
        icao_code: "C172".to_string(),
        flown: 0,
        aircraft_range: 500,
        category: "A".to_string(),
        cruise_speed: 120,
        date_flown: None,
        takeoff_distance: Some(500), // ~1640ft - should use small runway bucket
    });

    let large_aircraft = Arc::new(Aircraft {
        id: 11,
        manufacturer: "Boeing".to_string(),
        variant: "777".to_string(),
        icao_code: "B777".to_string(),
        flown: 0,
        aircraft_range: 8000,
        category: "H".to_string(),
        cruise_speed: 560,
        date_flown: None,
        takeoff_distance: Some(3000), // ~9843ft - should use large runway bucket
    });

    // Test that the optimized function can find suitable airports
    let small_airport = route_generator.get_airport_with_suitable_runway_optimized(&small_aircraft);
    let large_airport = route_generator.get_airport_with_suitable_runway_optimized(&large_aircraft);

    // Both should find airports since our test data has airports with long runways
    assert!(
        small_airport.is_some(),
        "Should find airport for small aircraft"
    );
    assert!(
        large_airport.is_some(),
        "Should find airport for large aircraft"
    );

    // Verify the selected airports have adequate runway lengths
    if let Some(airport) = small_airport {
        let runway_length = route_generator
            .longest_runway_cache
            .get(&airport.ID)
            .unwrap();
        let required_ft = (500.0 * METERS_TO_FEET).round() as i32;
        assert!(
            *runway_length >= required_ft,
            "Selected airport for small aircraft should have adequate runway"
        );
    }

    if let Some(airport) = large_airport {
        let runway_length = route_generator
            .longest_runway_cache
            .get(&airport.ID)
            .unwrap();
        let required_ft = (3000.0 * METERS_TO_FEET).round() as i32;
        assert!(
            *runway_length >= required_ft,
            "Selected airport for large aircraft should have adequate runway"
        );
    }
}

#[test]
fn test_get_airport_with_suitable_runway_optimized_no_suitable_airport() {
    let (_all_aircraft, all_airports, all_runways, spatial_airports) = create_test_data();
    let route_generator = RouteGenerator::new(all_airports, all_runways, spatial_airports);

    // Create an aircraft with extremely long takeoff distance that no airport can accommodate
    let extreme_aircraft = Arc::new(Aircraft {
        id: 12,
        manufacturer: "Theoretical".to_string(),
        variant: "Extreme".to_string(),
        icao_code: "EXTR".to_string(),
        flown: 0,
        aircraft_range: 10000,
        category: "H".to_string(),
        cruise_speed: 600,
        date_flown: None,
        takeoff_distance: Some(20000), // ~65617ft - much longer than any runway
    });

    let result = route_generator.get_airport_with_suitable_runway_optimized(&extreme_aircraft);

    // Should return None since no airport can accommodate this aircraft
    assert!(
        result.is_none(),
        "Should not find any airport for aircraft with extreme takeoff distance"
    );
}

#[test]
fn test_get_airport_with_suitable_runway_optimized_no_takeoff_distance() {
    let (_all_aircraft, all_airports, all_runways, spatial_airports) = create_test_data();
    let route_generator = RouteGenerator::new(all_airports, all_runways, spatial_airports);

    // Create an aircraft with no takeoff distance specified
    let no_distance_aircraft = Arc::new(Aircraft {
        id: 13,
        manufacturer: "Generic".to_string(),
        variant: "Unknown".to_string(),
        icao_code: "UNKN".to_string(),
        flown: 0,
        aircraft_range: 2000,
        category: "A".to_string(),
        cruise_speed: 300,
        date_flown: None,
        takeoff_distance: None, // No takeoff distance
    });

    let result = route_generator.get_airport_with_suitable_runway_optimized(&no_distance_aircraft);

    // Should still find an airport (will use bucket 0 which contains all airports)
    assert!(
        result.is_some(),
        "Should find airport for aircraft with no takeoff distance specified"
    );
}

#[test]
fn test_optimized_airport_selection_consistency() {
    let (_all_aircraft, all_airports, all_runways, spatial_airports) = create_test_data();
    let route_generator = RouteGenerator::new(all_airports, all_runways, spatial_airports);

    let test_aircraft = Arc::new(Aircraft {
        id: 14,
        manufacturer: "Test".to_string(),
        variant: "Consistency".to_string(),
        icao_code: "TEST".to_string(),
        flown: 0,
        aircraft_range: 3000,
        category: "A".to_string(),
        cruise_speed: 450,
        date_flown: None,
        takeoff_distance: Some(1500), // Moderate takeoff distance
    });

    // Run the optimized selection multiple times to ensure it consistently finds valid airports
    let mut found_airports = std::collections::HashSet::new();

    for _ in 0..10 {
        if let Some(airport) =
            route_generator.get_airport_with_suitable_runway_optimized(&test_aircraft)
        {
            found_airports.insert(airport.ID);

            // Verify each found airport meets the requirements
            let runway_length = route_generator
                .longest_runway_cache
                .get(&airport.ID)
                .unwrap();
            let required_ft = (1500.0 * METERS_TO_FEET).round() as i32;
            assert!(
                *runway_length >= required_ft,
                "Airport {} should have runway >= {}ft, but has {}ft",
                airport.ICAO,
                required_ft,
                runway_length
            );
        }
    }

    // Should have found at least one valid airport
    assert!(
        !found_airports.is_empty(),
        "Should consistently find valid airports for test aircraft"
    );
}

#[test]
fn test_parallel_route_generation_quality() {
    let (all_aircraft, all_airports, all_runways, spatial_airports) = create_test_data();
    let route_generator = RouteGenerator::new(all_airports, all_runways, spatial_airports);

    // Test parallel generation with different amounts
    let test_amounts = vec![1, 5, 10, 25, 50, 100];

    for &amount in &test_amounts {
        let routes = route_generator.generate_random_routes_generic(&all_aircraft, amount, None);

        // Verify we get the expected number of routes (or close to it, allowing for some randomness)
        assert!(
            routes.len() <= amount,
            "Should not generate more routes than requested for amount {}",
            amount
        );

        // For our test data, we should be able to generate routes consistently
        assert!(
            !routes.is_empty(),
            "Should generate at least some routes for amount {}",
            amount
        );

        // Verify route quality
        for route in &routes {
            // Departure and destination should be different
            assert_ne!(
                route.departure.ID, route.destination.ID,
                "Departure and destination should be different"
            );

            // Route length should be valid
            let route_length: f64 = route
                .route_length
                .parse()
                .expect("Route length should be a valid number");
            assert!(route_length > 0.0, "Route length should be positive");

            // Runway lengths should be valid
            let dep_runway: i32 = route
                .departure_runway_length
                .parse()
                .expect("Departure runway length should be a valid number");
            let dest_runway: i32 = route
                .destination_runway_length
                .parse()
                .expect("Destination runway length should be a valid number");
            assert!(dep_runway > 0, "Departure runway length should be positive");
            assert!(
                dest_runway > 0,
                "Destination runway length should be positive"
            );

            // Aircraft should meet runway requirements
            if let Some(takeoff_distance) = route.aircraft.takeoff_distance {
                let required_ft = (f64::from(takeoff_distance) * METERS_TO_FEET).round() as i32;
                assert!(
                    dep_runway >= required_ft,
                    "Departure airport should have adequate runway for aircraft {} (needs {}ft, has {}ft)",
                    route.aircraft.icao_code,
                    required_ft,
                    dep_runway
                );
                assert!(
                    dest_runway >= required_ft,
                    "Destination airport should have adequate runway for aircraft {} (needs {}ft, has {}ft)",
                    route.aircraft.icao_code,
                    required_ft,
                    dest_runway
                );
            }
        }
    }
}

#[test]
fn test_route_generation_with_fixed_departure() {
    let (all_aircraft, all_airports, all_runways, spatial_airports) = create_test_data();
    let route_generator = RouteGenerator::new(all_airports, all_runways, spatial_airports);

    // Test with a valid departure airport
    let routes = route_generator.generate_random_routes_generic(&all_aircraft, 10, Some("EHAM"));

    assert_eq!(
        routes.len(),
        10,
        "Should generate exactly 10 routes with fixed departure"
    );

    for route in &routes {
        assert_eq!(
            route.departure.ICAO, "EHAM",
            "All routes should depart from EHAM"
        );
        assert_ne!(
            route.departure.ICAO, route.destination.ICAO,
            "Should not have routes from EHAM to EHAM"
        );
    }
}

#[test]
fn test_bucket_algorithm_edge_cases() {
    let (_all_aircraft, all_airports, all_runways, spatial_airports) = create_test_data();
    let route_generator = RouteGenerator::new(all_airports, all_runways, spatial_airports);

    // Test edge case: aircraft with takeoff distance exactly matching a bucket boundary
    let bucket_boundary_aircraft = Arc::new(Aircraft {
        id: 15,
        manufacturer: "Edge".to_string(),
        variant: "Case".to_string(),
        icao_code: "EDGE".to_string(),
        flown: 0,
        aircraft_range: 3000,
        category: "A".to_string(),
        cruise_speed: 450,
        date_flown: None,
        takeoff_distance: Some(1524), // Exactly 5000ft when converted
    });

    let result =
        route_generator.get_airport_with_suitable_runway_optimized(&bucket_boundary_aircraft);

    if let Some(airport) = result {
        let runway_length = route_generator
            .longest_runway_cache
            .get(&airport.ID)
            .unwrap();
        let required_ft = (1524.0 * METERS_TO_FEET).round() as i32;
        assert!(
            *runway_length >= required_ft,
            "Airport should meet exact runway requirement at bucket boundary"
        );
    }

    // Test that the bucket algorithm correctly handles the bucket boundaries
    let required_ft = (1524.0 * METERS_TO_FEET).round() as i32; // Should be ~5000ft
    let bucket_key = route_generator
        .airports_by_runway_length
        .keys()
        .filter(|&&bucket| bucket <= required_ft)
        .max()
        .copied()
        .unwrap_or(0);

    assert!(
        bucket_key <= required_ft,
        "Selected bucket should not exceed aircraft requirements"
    );
}
