use std::sync::Arc;

use flight_planner::gui::data::ListItemRoute;
use flight_planner::gui::services::route_service;
use flight_planner::models::{Aircraft, Airport};

fn create_test_routes() -> Vec<ListItemRoute> {
    let aircraft1 = Arc::new(Aircraft {
        id: 1,
        manufacturer: "Cessna".to_string(),
        variant: "172".to_string(),
        icao_code: "C172".to_string(),
        flown: 0,
        aircraft_range: 640,
        category: "Light".to_string(),
        cruise_speed: 120,
        date_flown: None,
        takeoff_distance: Some(1000),
    });

    let aircraft2 = Arc::new(Aircraft {
        id: 2,
        manufacturer: "Boeing".to_string(),
        variant: "737-800".to_string(),
        icao_code: "B738".to_string(),
        flown: 1,
        aircraft_range: 3115,
        category: "Jet".to_string(),
        cruise_speed: 450,
        date_flown: Some("2024-01-15".to_string()),
        takeoff_distance: Some(7500),
    });

    let airport1 = Arc::new(Airport {
        ID: 1,
        ICAO: "KJFK".to_string(),
        Name: "John F. Kennedy International Airport".to_string(),
        PrimaryID: None,
        Latitude: 40.6413,
        Longtitude: -73.7781,
        Elevation: 13,
        TransitionAltitude: None,
        TransitionLevel: None,
        SpeedLimit: None,
        SpeedLimitAltitude: None,
    });

    let airport2 = Arc::new(Airport {
        ID: 2,
        ICAO: "KLAX".to_string(),
        Name: "Los Angeles International Airport".to_string(),
        PrimaryID: None,
        Latitude: 33.9425,
        Longtitude: -118.4081,
        Elevation: 125,
        TransitionAltitude: None,
        TransitionLevel: None,
        SpeedLimit: None,
        SpeedLimitAltitude: None,
    });

    let airport3 = Arc::new(Airport {
        ID: 3,
        ICAO: "EGLL".to_string(),
        Name: "London Heathrow Airport".to_string(),
        PrimaryID: None,
        Latitude: 51.4700,
        Longtitude: -0.4543,
        Elevation: 83,
        TransitionAltitude: None,
        TransitionLevel: None,
        SpeedLimit: None,
        SpeedLimitAltitude: None,
    });

    vec![
        ListItemRoute {
            departure: Arc::clone(&airport1),
            destination: Arc::clone(&airport2),
            aircraft: Arc::clone(&aircraft1),
            departure_runway_length: "14511ft".to_string(),
            destination_runway_length: "12091ft".to_string(),
            route_length: "2475.5 NM".to_string(),
        },
        ListItemRoute {
            departure: Arc::clone(&airport2),
            destination: Arc::clone(&airport3),
            aircraft: Arc::clone(&aircraft2),
            departure_runway_length: "12091ft".to_string(),
            destination_runway_length: "12802ft".to_string(),
            route_length: "5440.2 NM".to_string(),
        },
        ListItemRoute {
            departure: Arc::clone(&airport3),
            destination: Arc::clone(&airport1),
            aircraft: Arc::clone(&aircraft1),
            departure_runway_length: "12802ft".to_string(),
            destination_runway_length: "14511ft".to_string(),
            route_length: "3459.1 NM".to_string(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_items_empty_query() {
        let routes = create_test_routes();
        let filtered = route_service::filter_items(&routes, "");

        assert_eq!(filtered.len(), 3);
    }

    #[test]
    fn test_filter_items_by_departure_icao() {
        let routes = create_test_routes();
        let filtered = route_service::filter_items(&routes, "KJFK");

        // KJFK appears as departure in route 1 and destination in route 3
        assert_eq!(filtered.len(), 2);
        // Verify that both routes contain KJFK in either departure or destination
        assert!(filtered
            .iter()
            .all(|route| route.departure.ICAO == "KJFK" || route.destination.ICAO == "KJFK"));
    }

    #[test]
    fn test_filter_items_by_destination_icao() {
        let routes = create_test_routes();
        let filtered = route_service::filter_items(&routes, "KLAX");

        // KLAX appears as destination in route 1 and departure in route 2
        assert_eq!(filtered.len(), 2);
        // Verify that both routes contain KLAX in either departure or destination
        assert!(filtered
            .iter()
            .all(|route| route.departure.ICAO == "KLAX" || route.destination.ICAO == "KLAX"));
    }

    #[test]
    fn test_filter_items_by_aircraft() {
        let routes = create_test_routes();
        let filtered = route_service::filter_items(&routes, "Boeing");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].aircraft.manufacturer, "Boeing");
    }

    #[test]
    fn test_filter_items_by_distance() {
        let routes = create_test_routes();
        let filtered = route_service::filter_items(&routes, "2475");

        assert_eq!(filtered.len(), 1);
        assert!(filtered[0].route_length.contains("2475"));
    }

    #[test]
    fn test_filter_items_case_insensitive() {
        let routes = create_test_routes();
        let filtered = route_service::filter_items(&routes, "boeing");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].aircraft.manufacturer, "Boeing");
    }

    #[test]
    fn test_filter_items_no_matches() {
        let routes = create_test_routes();
        let filtered = route_service::filter_items(&routes, "nonexistent");

        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_sort_items_by_departure_ascending() {
        let routes = create_test_routes();
        let mut routes_vec = routes;
        route_service::sort_items(&mut routes_vec, "departure", true);

        assert_eq!(routes_vec[0].departure.ICAO, "EGLL");
        assert_eq!(routes_vec[1].departure.ICAO, "KJFK");
        assert_eq!(routes_vec[2].departure.ICAO, "KLAX");
    }

    #[test]
    fn test_sort_items_by_departure_descending() {
        let routes = create_test_routes();
        let mut routes_vec = routes;
        route_service::sort_items(&mut routes_vec, "departure", false);

        assert_eq!(routes_vec[0].departure.ICAO, "KLAX");
        assert_eq!(routes_vec[1].departure.ICAO, "KJFK");
        assert_eq!(routes_vec[2].departure.ICAO, "EGLL");
    }

    #[test]
    fn test_sort_items_by_arrival_ascending() {
        let routes = create_test_routes();
        let mut routes_vec = routes;
        route_service::sort_items(&mut routes_vec, "arrival", true);

        assert_eq!(routes_vec[0].destination.ICAO, "EGLL");
        assert_eq!(routes_vec[1].destination.ICAO, "KJFK");
        assert_eq!(routes_vec[2].destination.ICAO, "KLAX");
    }

    #[test]
    fn test_sort_items_by_aircraft_ascending() {
        let routes = create_test_routes();
        let mut routes_vec = routes;
        route_service::sort_items(&mut routes_vec, "aircraft", true);

        // Boeing 737-800 comes before Cessna 172 alphabetically
        assert_eq!(routes_vec[0].aircraft.manufacturer, "Boeing");
        assert_eq!(routes_vec[1].aircraft.manufacturer, "Cessna");
        assert_eq!(routes_vec[2].aircraft.manufacturer, "Cessna");
    }

    #[test]
    fn test_sort_items_by_distance_ascending() {
        let routes = create_test_routes();
        let mut routes_vec = routes;
        route_service::sort_items(&mut routes_vec, "distance", true);

        // Should be sorted: 2475.5, 3459.1, 5440.2
        assert_eq!(routes_vec[0].route_length, "2475.5 NM");
        assert_eq!(routes_vec[1].route_length, "3459.1 NM");
        assert_eq!(routes_vec[2].route_length, "5440.2 NM");
    }

    #[test]
    fn test_sort_items_by_distance_descending() {
        let routes = create_test_routes();
        let mut routes_vec = routes;
        route_service::sort_items(&mut routes_vec, "distance", false);

        // Should be sorted: 5440.2, 3459.1, 2475.5
        assert_eq!(routes_vec[0].route_length, "5440.2 NM");
        assert_eq!(routes_vec[1].route_length, "3459.1 NM");
        assert_eq!(routes_vec[2].route_length, "2475.5 NM");
    }

    #[test]
    fn test_sort_items_unknown_column() {
        let routes = create_test_routes();
        let original_routes = routes.clone();
        let mut routes_vec = routes;

        route_service::sort_items(&mut routes_vec, "unknown", true);

        // Should remain unchanged
        assert_eq!(
            routes_vec[0].departure.ICAO,
            original_routes[0].departure.ICAO
        );
        assert_eq!(
            routes_vec[1].departure.ICAO,
            original_routes[1].departure.ICAO
        );
        assert_eq!(
            routes_vec[2].departure.ICAO,
            original_routes[2].departure.ICAO
        );
    }

    #[test]
    fn test_sort_items_empty_routes() {
        let mut routes: Vec<ListItemRoute> = vec![];
        route_service::sort_items(&mut routes, "departure", true);

        assert_eq!(routes.len(), 0);
    }
}
