use common::create_test_airport;
use flight_planner::util::*;

mod common;

#[test]
fn test_calculate_haversine_distance_nm_parameterized() {
    let test_cases = vec![
        ("Same airport", 52.0, 4.0, 52.0, 4.0, 0),
        ("Different airports", 52.0, 4.0, 48.0, 2.0, 252),
        ("Negative coordinates", -34.0, -58.0, 40.0, 2.0, 5548),
        ("Antipodal", 0.0, 0.0, 0.0, 180.0, 10807),
    ];

    for (description, lat1, lon1, lat2, lon2, expected) in test_cases {
        let mut airport1 = create_test_airport(1, "Test1", "TST1");
        airport1.Latitude = lat1;
        airport1.Longtitude = lon1;

        let mut airport2 = create_test_airport(2, "Test2", "TST2");
        airport2.Latitude = lat2;
        airport2.Longtitude = lon2;

        let distance = calculate_haversine_distance_nm(&airport1, &airport2);
        assert_eq!(distance, expected, "Failed case: {}", description);
    }
}
