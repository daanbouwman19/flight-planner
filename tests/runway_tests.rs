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
    struct TestCase<'a> {
        description: &'a str,
        ident: &'a str,
        heading: f64,
        length: i32,
        width: i32,
        surface: &'a str,
        elevation: i32,
        expected: &'a str,
    }

    let test_cases = vec![
        TestCase {
            description: "Standard runway",
            ident: "09",
            heading: 92.0,
            length: 20000,
            width: 45,
            surface: "Asphalt",
            elevation: -11,
            expected: "Runway: 09, heading: 92.00, length: 20000 ft, width: 45 ft, surface: Asphalt, elevation: -11 ft",
        },
        TestCase {
            description: "Concrete runway positive elevation",
            ident: "18R",
            heading: 180.5,
            length: 10000,
            width: 30,
            surface: "Concrete",
            elevation: 100,
            expected: "Runway: 18R, heading: 180.50, length: 10000 ft, width: 30 ft, surface: Concrete, elevation: 100 ft",
        },
        TestCase {
            description: "Grass runway zero elevation",
            ident: "36",
            heading: 0.0,
            length: 5000,
            width: 20,
            surface: "Grass",
            elevation: 0,
            expected: "Runway: 36, heading: 0.00, length: 5000 ft, width: 20 ft, surface: Grass, elevation: 0 ft",
        },
    ];

    for case in test_cases {
        let mut runway = create_test_runway(1, 1, case.ident);
        runway.TrueHeading = case.heading;
        runway.Length = case.length;
        runway.Width = case.width;
        runway.Surface = case.surface.to_string();
        runway.Elevation = case.elevation;

        let formatted = format_runway(&runway);
        assert_eq!(
            formatted, case.expected,
            "Failed case: {}",
            case.description
        );
    }
}
