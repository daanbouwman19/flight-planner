use flight_planner::modules::routes::*;
use flight_planner::gui::ui::ListItemRoute;
use flight_planner::models::{Aircraft, Airport, Runway};
use rstar::RTree;
use std::collections::HashMap;
use std::sync::Arc;

type AircraftVec = Vec<Arc<Aircraft>>;
type AirportVec = Vec<Arc<Airport>>;
type RunwayMap = HashMap<i32, Arc<Vec<Runway>>>;
type AirportRTree = RTree<flight_planner::gui::ui::SpatialAirport>;

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
            .map(|airport| flight_planner::gui::ui::SpatialAirport {
                airport: Arc::clone(airport),
            })
            .collect(),
    );

    (all_aircraft, all_airports, runway_map, spatial_airports)
}

#[test]
fn test_generate_random_routes() {
    let (all_aircraft, all_airports, all_runways, spatial_airports) = create_test_data();
    let route_generator = RouteGenerator {
        all_airports,
        all_runways,
        spatial_airports,
    };

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
    let route_generator = RouteGenerator {
        all_airports,
        all_runways,
        spatial_airports,
    };

    let routes =
        route_generator.generate_random_not_flown_aircraft_routes(&all_aircraft, None);
    assert!(!routes.is_empty());
    for route in routes {
        assert!(route.departure.ID != route.destination.ID);
        assert!(route.route_length != "0");
    }
}

#[test]
fn test_generate_random_routes_generic() {
    let (all_aircraft, all_airports, all_runways, spatial_airports) = create_test_data();
    let route_generator = RouteGenerator {
        all_airports,
        all_runways,
        spatial_airports,
    };

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
    let route_generator = RouteGenerator {
        all_airports,
        all_runways,
        spatial_airports,
    };

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
    let route_generator = RouteGenerator {
        all_airports,
        all_runways,
        spatial_airports,
    };

    let routes: Vec<ListItemRoute> =
        route_generator.generate_random_routes_generic(&all_aircraft, 10, Some("INVALID"));
    assert_eq!(routes.len(), 0);
}
