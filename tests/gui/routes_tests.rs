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
        date_flown: Some("2024-12-10".to_string()),
        takeoff_distance: Some(2000),
        ..crate::common::create_test_aircraft(1, "Boeing", "737-800", "B738")
    });
    let aircraft2 = Arc::new(Aircraft {
        aircraft_range: 2500,
        cruise_speed: 430,
        takeoff_distance: Some(1800),
        ..crate::common::create_test_aircraft(2, "Airbus", "A320", "A320")
    });

    let all_aircraft = vec![aircraft1, aircraft2];

    let airport1 = Arc::new(Airport {
        Latitude: 52.3086,
        Longtitude: 4.7639,
        Elevation: -11,
        TransitionAltitude: Some(10000),
        SpeedLimit: Some(230),
        SpeedLimitAltitude: Some(6000),
        ..crate::common::create_test_airport(1, "Amsterdam Airport Schiphol", "EHAM")
    });
    let airport2 = Arc::new(Airport {
        Latitude: 51.9561,
        Longtitude: 4.4397,
        Elevation: -13,
        TransitionAltitude: Some(5000),
        SpeedLimit: Some(180),
        SpeedLimitAltitude: Some(4000),
        ..crate::common::create_test_airport(2, "Rotterdam The Hague Airport", "EHRD")
    });
    let all_airports = vec![airport1, airport2];

    let runway1 = Runway {
        TrueHeading: 92.0,
        Latitude: 52.3086,
        Longtitude: 4.7639,
        Elevation: -11,
        ..crate::common::create_test_runway(1, 1, "09")
    };
    let runway2 = Runway {
        TrueHeading: 62.0,
        Latitude: 51.9561,
        Longtitude: 4.4397,
        Elevation: -13,
        ..crate::common::create_test_runway(2, 2, "06")
    };

    let mut runway_map = HashMap::new();
    runway_map.insert(1, Arc::new(vec![runway1]));
    runway_map.insert(2, Arc::new(vec![runway2]));

    let spatial_airports = RTree::bulk_load(
        all_airports
            .iter()
            .map(|airport| {
                let longest_runway = runway_map
                    .get(&airport.ID)
                    .and_then(|runways| runways.iter().map(|r| r.Length).max())
                    .unwrap_or(0);
                SpatialAirport {
                    airport: flight_planner::models::airport::CachedAirport::new(Arc::clone(
                        airport,
                    )),
                    longest_runway_length: longest_runway,
                }
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
        assert!(route.route_length > 0.0);
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
        assert!(route.route_length > 0.0);
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
        assert!(route.route_length > 0.0);
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
        assert!(route.route_length > 0.0);
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
        category: "C".to_string(),
        takeoff_distance: Some(2134), // Requires ~7000ft
        ..crate::common::create_test_aircraft(3, "Test", "Test", "TEST")
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
fn test_sorted_airports_structure() {
    let (_all_aircraft, all_airports, all_runways, spatial_airports) = create_test_data();
    let route_generator = RouteGenerator::new(all_airports.clone(), all_runways, spatial_airports);

    // Verify that all_airports is sorted by runway length
    assert!(!route_generator.all_airports.is_empty());

    for i in 0..route_generator.all_airports.len() - 1 {
        let id1 = route_generator.all_airports[i].inner.ID;
        let id2 = route_generator.all_airports[i + 1].inner.ID;

        let len1 = route_generator.longest_runway_cache.get(&id1).unwrap();
        let len2 = route_generator.longest_runway_cache.get(&id2).unwrap();

        assert!(
            len1 <= len2,
            "Airports should be sorted by runway length ascending"
        );
    }

    // Verify parallel sorted_runway_lengths vector
    assert_eq!(
        route_generator.all_airports.len(),
        route_generator.sorted_runway_lengths.len(),
        "sorted_runway_lengths should match all_airports length"
    );

    for i in 0..route_generator.sorted_runway_lengths.len() {
        let id = route_generator.all_airports[i].inner.ID;
        let cache_len = route_generator.longest_runway_cache.get(&id).unwrap();
        let vec_len = route_generator.sorted_runway_lengths[i];

        assert_eq!(
            *cache_len, vec_len,
            "sorted_runway_lengths[{}] should match cached length",
            i
        );
    }
}

#[test]
fn test_get_airport_with_suitable_runway_optimized_logic() {
    let (_all_aircraft, all_airports, all_runways, spatial_airports) = create_test_data();
    let route_generator = RouteGenerator::new(all_airports, all_runways, spatial_airports);

    // Test with aircraft requiring different runway lengths
    let small_aircraft = Arc::new(Aircraft {
        aircraft_range: 500,
        cruise_speed: 120,
        takeoff_distance: Some(500), // ~1640ft
        ..crate::common::create_test_aircraft(10, "Cessna", "172", "C172")
    });

    let large_aircraft = Arc::new(Aircraft {
        aircraft_range: 8000,
        category: "H".to_string(),
        cruise_speed: 560,
        takeoff_distance: Some(3000), // ~9843ft
        ..crate::common::create_test_aircraft(11, "Boeing", "777", "B777")
    });

    // Test that the optimized function can find suitable airports
    let mut rng = rand::rng();
    let small_airport =
        route_generator.get_airport_with_suitable_runway_optimized(&small_aircraft, &mut rng);
    let large_airport =
        route_generator.get_airport_with_suitable_runway_optimized(&large_aircraft, &mut rng);

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
            .get(&airport.inner.ID)
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
            .get(&airport.inner.ID)
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
        aircraft_range: 10000,
        category: "H".to_string(),
        cruise_speed: 600,
        takeoff_distance: Some(20000), // ~65617ft - much longer than any runway
        ..crate::common::create_test_aircraft(12, "Theoretical", "Extreme", "EXTR")
    });

    let mut rng = rand::rng();
    let result =
        route_generator.get_airport_with_suitable_runway_optimized(&extreme_aircraft, &mut rng);

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
        aircraft_range: 2000,
        cruise_speed: 300,
        takeoff_distance: None, // No takeoff distance
        ..crate::common::create_test_aircraft(13, "Generic", "Unknown", "UNKN")
    });

    let mut rng = rand::rng();
    let result =
        route_generator.get_airport_with_suitable_runway_optimized(&no_distance_aircraft, &mut rng);

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
        takeoff_distance: Some(1500), // Moderate takeoff distance
        ..crate::common::create_test_aircraft(14, "Test", "Consistency", "TEST")
    });

    // Run the optimized selection multiple times to ensure it consistently finds valid airports
    let mut found_airports = std::collections::HashSet::new();

    for _ in 0..10 {
        let mut rng = rand::rng();
        if let Some(airport) =
            route_generator.get_airport_with_suitable_runway_optimized(&test_aircraft, &mut rng)
        {
            found_airports.insert(airport.inner.ID);

            // Verify each found airport meets the requirements
            let runway_length = route_generator
                .longest_runway_cache
                .get(&airport.inner.ID)
                .unwrap();
            let required_ft = (1500.0 * METERS_TO_FEET).round() as i32;
            assert!(
                *runway_length >= required_ft,
                "Airport {} should have runway >= {}ft, but has {}ft",
                airport.inner.ICAO,
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
            "Should not generate more routes than requested for amount {amount}"
        );

        // For our test data, we should be able to generate routes consistently
        assert!(
            !routes.is_empty(),
            "Should generate at least some routes for amount {amount}"
        );

        // Verify route quality
        for route in &routes {
            // Departure and destination should be different
            assert_ne!(
                route.departure.ID, route.destination.ID,
                "Departure and destination should be different"
            );

            // Route length should be valid
            assert!(route.route_length > 0.0, "Route length should be positive");

            // Runway lengths should be valid
            let dep_runway = route.departure_runway_length;
            let dest_runway = route.destination_runway_length;
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

    assert!(
        !routes.is_empty(),
        "Should generate at least one route with fixed departure"
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
fn test_binary_search_edge_cases() {
    let (_all_aircraft, all_airports, all_runways, spatial_airports) = create_test_data();
    let route_generator = RouteGenerator::new(all_airports, all_runways, spatial_airports);

    // Test edge case: aircraft with takeoff distance exactly matching an airport
    // In our test data, both airports have 10000ft runways

    let boundary_aircraft = Arc::new(Aircraft {
        takeoff_distance: Some(3048), // Exactly 10000ft when converted
        ..crate::common::create_test_aircraft(15, "Edge", "Case", "EDGE")
    });

    let mut rng = rand::rng();
    let result =
        route_generator.get_airport_with_suitable_runway_optimized(&boundary_aircraft, &mut rng);

    if let Some(airport) = result {
        let runway_length = route_generator
            .longest_runway_cache
            .get(&airport.inner.ID)
            .unwrap();
        let required_ft = (3048.0 * METERS_TO_FEET).round() as i32;
        assert!(
            *runway_length >= required_ft,
            "Airport should meet exact runway requirement"
        );
    }

    // Test binary search logic manually
    let required_ft = (3048.0 * METERS_TO_FEET).round() as i32; // ~10000ft
    let start_idx = route_generator
        .sorted_runway_lengths
        .partition_point(|&len| len < required_ft);

    // Should return index 0 because both airports have 10000ft, which is !< 10000ft (so it is false)
    // partition_point finds first index where predicate is false.
    // 10000 < 10000 is False.
    // So if first element is 10000, index 0 is returned. Correct.
    assert_eq!(start_idx, 0, "Binary search should include exact matches");
}
