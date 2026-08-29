use common::create_test_airport;
use flight_planner_lib::util::*;

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

#[test]
fn test_validate_env_path() {
    use common::with_env_overrides;
    use std::path::PathBuf;

    // 1. Missing env var returns None
    with_env_overrides(vec![("TEST_SAFE_PATH_VAR", None)], || {
        assert_eq!(validate_env_path("TEST_SAFE_PATH_VAR"), None);
    });

    // 2. Empty string returns None
    with_env_overrides(vec![("TEST_SAFE_PATH_VAR", Some(""))], || {
        assert_eq!(validate_env_path("TEST_SAFE_PATH_VAR"), None);
    });

    // 3. Path traversal attempts return None
    with_env_overrides(vec![("TEST_SAFE_PATH_VAR", Some("../traversal"))], || {
        assert_eq!(validate_env_path("TEST_SAFE_PATH_VAR"), None);
    });

    with_env_overrides(vec![("TEST_SAFE_PATH_VAR", Some("foo/../../bar"))], || {
        assert_eq!(validate_env_path("TEST_SAFE_PATH_VAR"), None);
    });

    with_env_overrides(vec![("TEST_SAFE_PATH_VAR", Some(".."))], || {
        assert_eq!(validate_env_path("TEST_SAFE_PATH_VAR"), None);
    });

    // 4. Valid safe relative and absolute paths return Some(PathBuf)
    with_env_overrides(vec![("TEST_SAFE_PATH_VAR", Some("valid/path/dir"))], || {
        assert_eq!(
            validate_env_path("TEST_SAFE_PATH_VAR"),
            Some(PathBuf::from("valid/path/dir"))
        );
    });
}
