use flight_planner::modules::runway::*;

mod common;
use common::{create_test_runway, setup_test_pool_db};

#[test]
fn test_get_runways() {
    let pool = setup_test_pool_db();
    let runways = pool.get_runways().unwrap();
    assert_eq!(runways.len(), 6);
}

#[test]
fn test_format_runway() {
    let mut runway = create_test_runway(1, 1, "09");
    runway.TrueHeading = 92.0;
    runway.Length = 20000;
    runway.Width = 45;
    runway.Surface = "Asphalt".to_string();
    runway.Latitude = 52.3086;
    runway.Longtitude = 4.7639;
    runway.Elevation = -11;

    let formatted = format_runway(&runway);
    assert_eq!(
        formatted,
        "Runway: 09, heading: 92.00, length: 20000 ft, width: 45 ft, surface: Asphalt, elevation: -11ft"
    );
}
