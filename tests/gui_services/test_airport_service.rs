use std::collections::HashMap;
use std::sync::Arc;

use flight_planner::gui::services::airport_service;
use flight_planner::models::{Airport, Runway};

fn create_test_airports() -> Vec<Arc<Airport>> {
    vec![
        Arc::new(Airport {
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
        }),
        Arc::new(Airport {
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
        }),
        Arc::new(Airport {
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
        }),
    ]
}

fn create_test_runways() -> HashMap<i32, Arc<Vec<Runway>>> {
    let mut runway_map = HashMap::new();

    // KJFK runways
    runway_map.insert(
        1,
        Arc::new(vec![
            Runway {
                ID: 1,
                AirportID: 1,
                Ident: "04L/22R".to_string(),
                TrueHeading: 40.0,
                Length: 14511,
                Width: 150,
                Surface: "Concrete".to_string(),
                Latitude: 40.6413,
                Longtitude: -73.7781,
                Elevation: 13,
            },
            Runway {
                ID: 2,
                AirportID: 1,
                Ident: "09L/27R".to_string(),
                TrueHeading: 90.0,
                Length: 10000,
                Width: 150,
                Surface: "Concrete".to_string(),
                Latitude: 40.6413,
                Longtitude: -73.7781,
                Elevation: 13,
            },
        ]),
    );

    // KLAX runways
    runway_map.insert(
        2,
        Arc::new(vec![Runway {
            ID: 3,
            AirportID: 2,
            Ident: "07L/25R".to_string(),
            TrueHeading: 70.0,
            Length: 12091,
            Width: 150,
            Surface: "Concrete".to_string(),
            Latitude: 33.9425,
            Longtitude: -118.4081,
            Elevation: 125,
        }]),
    );

    // EGLL has no runways in this test data

    runway_map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_to_list_items_with_runways() {
        let airports = create_test_airports();
        let runways = create_test_runways();
        let list_items = airport_service::transform_to_list_items_with_runways(&airports, &runways);

        assert_eq!(list_items.len(), 3);

        // Check KJFK with longest runway
        assert_eq!(list_items[0].name, "John F. Kennedy International Airport");
        assert_eq!(list_items[0].icao, "KJFK");
        assert_eq!(list_items[0].longest_runway_length, "14511ft");

        // Check KLAX with single runway
        assert_eq!(list_items[1].name, "Los Angeles International Airport");
        assert_eq!(list_items[1].icao, "KLAX");
        assert_eq!(list_items[1].longest_runway_length, "12091ft");

        // Check EGLL with no runways
        assert_eq!(list_items[2].name, "London Heathrow Airport");
        assert_eq!(list_items[2].icao, "EGLL");
        assert_eq!(list_items[2].longest_runway_length, "No runways");
    }

    #[test]
    fn test_transform_to_list_items_with_runways_empty() {
        let airports = vec![];
        let runways = HashMap::new();
        let list_items = airport_service::transform_to_list_items_with_runways(&airports, &runways);
        assert_eq!(list_items.len(), 0);
    }

    #[test]
    fn test_transform_to_list_items_with_runways_no_runway_data() {
        let airports = create_test_airports();
        let runways = HashMap::new();
        let list_items = airport_service::transform_to_list_items_with_runways(&airports, &runways);

        assert_eq!(list_items.len(), 3);

        // All airports should show "No runways" since runway map is empty
        for item in &list_items {
            assert_eq!(item.longest_runway_length, "No runways");
        }
    }

    #[test]
    fn test_transform_to_list_items() {
        let airports = create_test_airports();
        let list_items = airport_service::transform_to_list_items(&airports);

        assert_eq!(list_items.len(), 3);

        // Check first airport
        assert_eq!(list_items[0].name, "John F. Kennedy International Airport");
        assert_eq!(list_items[0].icao, "KJFK");
        assert_eq!(list_items[0].longest_runway_length, "0");

        // Check second airport
        assert_eq!(list_items[1].name, "Los Angeles International Airport");
        assert_eq!(list_items[1].icao, "KLAX");
        assert_eq!(list_items[1].longest_runway_length, "0");

        // Check third airport
        assert_eq!(list_items[2].name, "London Heathrow Airport");
        assert_eq!(list_items[2].icao, "EGLL");
        assert_eq!(list_items[2].longest_runway_length, "0");
    }

    #[test]
    fn test_transform_to_list_items_empty() {
        let airports = vec![];
        let list_items = airport_service::transform_to_list_items(&airports);
        assert_eq!(list_items.len(), 0);
    }

    #[test]
    fn test_filter_items_empty_query() {
        let airports = create_test_airports();
        let list_items = airport_service::transform_to_list_items(&airports);
        let filtered = airport_service::filter_items(&list_items, "");

        assert_eq!(filtered.len(), 3);
        assert_eq!(filtered[0].icao, "KJFK");
        assert_eq!(filtered[1].icao, "KLAX");
        assert_eq!(filtered[2].icao, "EGLL");
    }

    #[test]
    fn test_filter_items_by_icao() {
        let airports = create_test_airports();
        let list_items = airport_service::transform_to_list_items(&airports);
        let filtered = airport_service::filter_items(&list_items, "KJFK");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].icao, "KJFK");
        assert_eq!(filtered[0].name, "John F. Kennedy International Airport");
    }

    #[test]
    fn test_filter_items_by_name() {
        let airports = create_test_airports();
        let list_items = airport_service::transform_to_list_items(&airports);
        let filtered = airport_service::filter_items(&list_items, "Kennedy");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].icao, "KJFK");
        assert_eq!(filtered[0].name, "John F. Kennedy International Airport");
    }

    #[test]
    fn test_filter_items_case_insensitive() {
        let airports = create_test_airports();
        let list_items = airport_service::transform_to_list_items(&airports);
        let filtered = airport_service::filter_items(&list_items, "angeles");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].icao, "KLAX");
    }

    #[test]
    fn test_filter_items_partial_match() {
        let airports = create_test_airports();
        let list_items = airport_service::transform_to_list_items(&airports);
        let filtered = airport_service::filter_items(&list_items, "International");

        assert_eq!(filtered.len(), 2); // KJFK and KLAX both have "International" in their names
        assert!(filtered.iter().any(|item| item.icao == "KJFK"));
        assert!(filtered.iter().any(|item| item.icao == "KLAX"));
    }

    #[test]
    fn test_filter_items_no_matches() {
        let airports = create_test_airports();
        let list_items = airport_service::transform_to_list_items(&airports);
        let filtered = airport_service::filter_items(&list_items, "nonexistent");

        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_filter_items_multiple_criteria() {
        let airports = create_test_airports();
        let list_items = airport_service::transform_to_list_items(&airports);
        let filtered = airport_service::filter_items(&list_items, "L"); // Should match KLAX and EGLL

        assert_eq!(filtered.len(), 3); // KLAX (matches in ICAO and "Los Angeles"), EGLL (matches in ICAO and "London"), plus KJFK matches "International" which has an "L"
        // All three airports contain "L" in some form
    }

    #[test]
    fn test_get_display_name_existing_airport() {
        let airports = create_test_airports();
        let display_name = airport_service::get_display_name(&airports, "KJFK");
        assert_eq!(display_name, "John F. Kennedy International Airport (KJFK)");
    }

    #[test]
    fn test_get_display_name_nonexistent_airport() {
        let airports = create_test_airports();
        let display_name = airport_service::get_display_name(&airports, "KXYZ");
        assert_eq!(display_name, "Unknown Airport (KXYZ)");
    }

    #[test]
    fn test_get_display_name_empty_airport_list() {
        let airports = vec![];
        let display_name = airport_service::get_display_name(&airports, "KJFK");
        assert_eq!(display_name, "Unknown Airport (KJFK)");
    }

    #[test]
    fn test_get_display_name_multiple_airports() {
        let airports = create_test_airports();

        let display_name_1 = airport_service::get_display_name(&airports, "KJFK");
        let display_name_2 = airport_service::get_display_name(&airports, "KLAX");
        let display_name_3 = airport_service::get_display_name(&airports, "EGLL");

        assert_eq!(
            display_name_1,
            "John F. Kennedy International Airport (KJFK)"
        );
        assert_eq!(display_name_2, "Los Angeles International Airport (KLAX)");
        assert_eq!(display_name_3, "London Heathrow Airport (EGLL)");
    }

    #[test]
    fn test_runway_selection_longest() {
        let airports = vec![Arc::new(Airport {
            ID: 1,
            ICAO: "TEST".to_string(),
            Name: "Test Airport".to_string(),
            PrimaryID: None,
            Latitude: 0.0,
            Longtitude: 0.0,
            Elevation: 0,
            TransitionAltitude: None,
            TransitionLevel: None,
            SpeedLimit: None,
            SpeedLimitAltitude: None,
        })];

        let mut runways = HashMap::new();
        runways.insert(
            1,
            Arc::new(vec![
                Runway {
                    ID: 1,
                    AirportID: 1,
                    Ident: "09/27".to_string(),
                    TrueHeading: 90.0,
                    Length: 5000,
                    Width: 100,
                    Surface: "Concrete".to_string(),
                    Latitude: 0.0,
                    Longtitude: 0.0,
                    Elevation: 0,
                },
                Runway {
                    ID: 2,
                    AirportID: 1,
                    Ident: "18/36".to_string(),
                    TrueHeading: 180.0,
                    Length: 8000,
                    Width: 100,
                    Surface: "Concrete".to_string(),
                    Latitude: 0.0,
                    Longtitude: 0.0,
                    Elevation: 0,
                },
                Runway {
                    ID: 3,
                    AirportID: 1,
                    Ident: "27/09".to_string(),
                    TrueHeading: 270.0,
                    Length: 6000,
                    Width: 100,
                    Surface: "Concrete".to_string(),
                    Latitude: 0.0,
                    Longtitude: 0.0,
                    Elevation: 0,
                },
            ]),
        );

        let list_items = airport_service::transform_to_list_items_with_runways(&airports, &runways);

        assert_eq!(list_items.len(), 1);
        assert_eq!(list_items[0].longest_runway_length, "8000ft");
    }
}
