use flight_planner::models::Airport;
use flight_planner::util::*;

/// Helper to create a test airport with specific coordinates.
fn create_test_airport(lat: f64, lon: f64) -> Airport {
    Airport {
        Latitude: lat,
        Longtitude: lon,
        ..Default::default()
    }
}

#[test]
fn test_calculate_haversine_distance_nm_cases() {
    struct TestCase {
        name: &'static str,
        airport1: Airport,
        airport2: Airport,
        expected_distance: i32,
    }

    // Earth radius used in util.rs is 3440.0 nm.
    // Antipodal distance = pi * R = 3.14159... * 3440.0 = 10807.07... -> 10807
    let test_cases = vec![
        TestCase {
            name: "same airport",
            airport1: create_test_airport(52.0, 4.0),
            airport2: create_test_airport(52.0, 4.0),
            expected_distance: 0,
        },
        TestCase {
            name: "different airports",
            airport1: create_test_airport(52.0, 4.0),
            airport2: create_test_airport(48.0, 2.0),
            expected_distance: 252,
        },
        TestCase {
            name: "negative coordinates",
            airport1: create_test_airport(-34.0, -58.0),
            airport2: create_test_airport(40.0, 2.0),
            expected_distance: 5548,
        },
        TestCase {
            name: "antipodal points",
            airport1: create_test_airport(0.0, 0.0),
            airport2: create_test_airport(0.0, 180.0),
            expected_distance: 10807,
        },
    ];

    for case in test_cases {
        let distance = calculate_haversine_distance_nm(&case.airport1, &case.airport2);
        assert_eq!(
            distance, case.expected_distance,
            "Failed test case: {}, expected {}, got {}",
            case.name, case.expected_distance, distance
        );
    }
}
