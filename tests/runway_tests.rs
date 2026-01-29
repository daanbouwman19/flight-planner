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
fn test_format_runway_parameterized() {
    let test_cases = vec![
        (
            "Standard runway",
            "09",
            92.0,
            20000,
            45,
            "Asphalt",
            -11,
            "Runway: 09, heading: 92.00, length: 20000 ft, width: 45 ft, surface: Asphalt, elevation: -11ft",
        ),
        (
            "Concrete runway positive elevation",
            "18R",
            180.5,
            10000,
            30,
            "Concrete",
            100,
            "Runway: 18R, heading: 180.50, length: 10000 ft, width: 30 ft, surface: Concrete, elevation: 100ft",
        ),
        (
            "Grass runway zero elevation",
            "36",
            0.0,
            5000,
            20,
            "Grass",
            0,
            "Runway: 36, heading: 0.00, length: 5000 ft, width: 20 ft, surface: Grass, elevation: 0ft",
        ),
    ];

    for (description, ident, heading, length, width, surface, elevation, expected) in test_cases {
        let mut runway = create_test_runway(1, 1, ident);
        runway.TrueHeading = heading;
        runway.Length = length;
        runway.Width = width;
        runway.Surface = surface.to_string();
        runway.Elevation = elevation;

        let formatted = format_runway(&runway);
        assert_eq!(formatted, expected, "Failed case: {}", description);
    }
}
