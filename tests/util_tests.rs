use flight_planner::models::Airport;
use flight_planner::util::*;

fn create_airport(lat: f64, lon: f64) -> Airport {
    Airport {
        Latitude: lat,
        Longtitude: lon,
        ..Default::default()
    }
}

#[test]
fn test_calculate_haversine_distance_nm_same_airport() {
    let airport = create_airport(52.0, 4.0);
    let distance = calculate_haversine_distance_nm(&airport, &airport);
    assert_eq!(distance, 0);
}

#[test]
fn test_calculate_haversine_distance_nm_different_airports() {
    let airport1 = create_airport(52.0, 4.0);
    let airport2 = create_airport(48.0, 2.0);

    let distance = calculate_haversine_distance_nm(&airport1, &airport2);
    assert_eq!(distance, 252);
}

#[test]
fn test_calculate_haversine_distance_nm_negative_coordinates() {
    let airport1 = create_airport(-34.0, -58.0);
    let airport2 = create_airport(40.0, 2.0);

    let distance = calculate_haversine_distance_nm(&airport1, &airport2);
    assert_eq!(distance, 5548);
}

#[test]
fn test_calculate_haversine_distance_nm_antipodal() {
    // Test maximum distance (approx half earth circumference)
    // 3440 * PI ≈ 10807.07
    let airport1 = create_airport(0.0, 0.0);
    let airport2 = create_airport(0.0, 180.0);

    let distance = calculate_haversine_distance_nm(&airport1, &airport2);
    assert_eq!(distance, 10807);
}
