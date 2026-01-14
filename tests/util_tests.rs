use flight_planner::util::*;
use common::create_test_airport;

mod common;

#[test]
fn test_calculate_haversine_distance_nm_same_airport() {
    let airport = create_test_airport(1, "Test", "TEST");
    let distance = calculate_haversine_distance_nm(&airport, &airport);
    assert_eq!(distance, 0);
}

#[test]
fn test_calculate_haversine_distance_nm_different_airports() {
    let mut airport1 = create_test_airport(1, "Test1", "TST1");
    airport1.Latitude = 52.0;
    airport1.Longtitude = 4.0;

    let mut airport2 = create_test_airport(2, "Test2", "TST2");
    airport2.Latitude = 48.0;
    airport2.Longtitude = 2.0;

    let distance = calculate_haversine_distance_nm(&airport1, &airport2);
    assert_eq!(distance, 252);
}

#[test]
fn test_calculate_haversine_distance_nm_negative_coordinates() {
    let mut airport1 = create_test_airport(1, "Test1", "TST1");
    airport1.Latitude = -34.0;
    airport1.Longtitude = -58.0;

    let mut airport2 = create_test_airport(2, "Test2", "TST2");
    airport2.Latitude = 40.0;
    airport2.Longtitude = 2.0;

    let distance = calculate_haversine_distance_nm(&airport1, &airport2);
    assert_eq!(distance, 5548);
}

#[test]
fn test_calculate_haversine_distance_nm_antipodal() {
    // Test maximum distance (approx half earth circumference)
    // 3440 * PI ≈ 10807.07
    let mut airport1 = create_test_airport(1, "Test1", "TST1");
    airport1.Latitude = 0.0;
    airport1.Longtitude = 0.0;

    let mut airport2 = create_test_airport(2, "Test2", "TST2");
    airport2.Latitude = 0.0;
    airport2.Longtitude = 180.0;

    let distance = calculate_haversine_distance_nm(&airport1, &airport2);
    assert_eq!(distance, 10807);
}
