use std::sync::Arc;

use flight_planner::gui::services::aircraft_service;
use flight_planner::models::Aircraft;

fn create_test_aircraft() -> Vec<Arc<Aircraft>> {
    vec![
        Arc::new(Aircraft {
            id: 1,
            manufacturer: "Cessna".to_string(),
            variant: "172".to_string(),
            icao_code: "C172".to_string(),
            flown: 1,
            aircraft_range: 640,
            category: "Light".to_string(),
            cruise_speed: 120,
            date_flown: Some("2024-01-15".to_string()),
            takeoff_distance: Some(1000),
        }),
        Arc::new(Aircraft {
            id: 2,
            manufacturer: "Boeing".to_string(),
            variant: "737-800".to_string(),
            icao_code: "B738".to_string(),
            flown: 0,
            aircraft_range: 3115,
            category: "Jet".to_string(),
            cruise_speed: 450,
            date_flown: None,
            takeoff_distance: Some(7500),
        }),
        Arc::new(Aircraft {
            id: 3,
            manufacturer: "Airbus".to_string(),
            variant: "A320".to_string(),
            icao_code: "A320".to_string(),
            flown: 0,
            aircraft_range: 3500,
            category: "Jet".to_string(),
            cruise_speed: 460,
            date_flown: None,
            takeoff_distance: Some(6500),
        }),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_to_list_items() {
        let aircraft = create_test_aircraft();
        let list_items = aircraft_service::transform_to_list_items(&aircraft);

        assert_eq!(list_items.len(), 3);

        // Check first aircraft
        assert_eq!(list_items[0].manufacturer, "Cessna");
        assert_eq!(list_items[0].variant, "172");
        assert_eq!(list_items[0].icao_code, "C172");
        assert_eq!(list_items[0].flown, 1);
        assert_eq!(list_items[0].range, "640 NM");
        assert_eq!(list_items[0].date_flown, "2024-01-15");

        // Check second aircraft
        assert_eq!(list_items[1].manufacturer, "Boeing");
        assert_eq!(list_items[1].variant, "737-800");
        assert_eq!(list_items[1].icao_code, "B738");
        assert_eq!(list_items[1].flown, 0);
        assert_eq!(list_items[1].date_flown, "Never");

        // Check third aircraft
        assert_eq!(list_items[2].manufacturer, "Airbus");
        assert_eq!(list_items[2].variant, "A320");
        assert_eq!(list_items[2].flown, 0);
    }

    #[test]
    fn test_transform_to_list_items_empty() {
        let aircraft = vec![];
        let list_items = aircraft_service::transform_to_list_items(&aircraft);
        assert_eq!(list_items.len(), 0);
    }

    #[test]
    fn test_filter_items_empty_query() {
        let aircraft = create_test_aircraft();
        let list_items = aircraft_service::transform_to_list_items(&aircraft);
        let filtered = aircraft_service::filter_items(&list_items, "");

        assert_eq!(filtered.len(), 3);
        assert_eq!(filtered[0].manufacturer, "Cessna");
        assert_eq!(filtered[1].manufacturer, "Boeing");
        assert_eq!(filtered[2].manufacturer, "Airbus");
    }

    #[test]
    fn test_filter_items_by_manufacturer() {
        let aircraft = create_test_aircraft();
        let list_items = aircraft_service::transform_to_list_items(&aircraft);
        let filtered = aircraft_service::filter_items(&list_items, "Cessna");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].manufacturer, "Cessna");
        assert_eq!(filtered[0].variant, "172");
    }

    #[test]
    fn test_filter_items_by_variant() {
        let aircraft = create_test_aircraft();
        let list_items = aircraft_service::transform_to_list_items(&aircraft);
        let filtered = aircraft_service::filter_items(&list_items, "737");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].manufacturer, "Boeing");
        assert_eq!(filtered[0].variant, "737-800");
    }

    #[test]
    fn test_filter_items_by_icao_code() {
        let aircraft = create_test_aircraft();
        let list_items = aircraft_service::transform_to_list_items(&aircraft);
        let filtered = aircraft_service::filter_items(&list_items, "A320");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].manufacturer, "Airbus");
        assert_eq!(filtered[0].icao_code, "A320");
    }

    #[test]
    fn test_filter_items_case_insensitive() {
        let aircraft = create_test_aircraft();
        let list_items = aircraft_service::transform_to_list_items(&aircraft);
        let filtered = aircraft_service::filter_items(&list_items, "cessna");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].manufacturer, "Cessna");
    }

    #[test]
    fn test_filter_items_partial_match() {
        let aircraft = create_test_aircraft();
        let list_items = aircraft_service::transform_to_list_items(&aircraft);
        let filtered = aircraft_service::filter_items(&list_items, "boe");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].manufacturer, "Boeing");
    }

    #[test]
    fn test_filter_items_no_matches() {
        let aircraft = create_test_aircraft();
        let list_items = aircraft_service::transform_to_list_items(&aircraft);
        let filtered = aircraft_service::filter_items(&list_items, "nonexistent");

        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_filter_items_multiple_matches() {
        let aircraft = vec![
            Arc::new(Aircraft {
                id: 1,
                manufacturer: "Boeing".to_string(),
                variant: "737-800".to_string(),
                icao_code: "B738".to_string(),
                flown: 0,
                aircraft_range: 3115,
                category: "Jet".to_string(),
                cruise_speed: 450,
                date_flown: None,
                takeoff_distance: Some(7500),
            }),
            Arc::new(Aircraft {
                id: 2,
                manufacturer: "Boeing".to_string(),
                variant: "747-400".to_string(),
                icao_code: "B744".to_string(),
                flown: 1,
                aircraft_range: 7260,
                category: "Jet".to_string(),
                cruise_speed: 490,
                date_flown: Some("2024-02-01".to_string()),
                takeoff_distance: Some(10000),
            }),
        ];

        let list_items = aircraft_service::transform_to_list_items(&aircraft);
        let filtered = aircraft_service::filter_items(&list_items, "Boeing");

        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].variant, "737-800");
        assert_eq!(filtered[1].variant, "747-400");
    }

    #[test]
    fn test_get_display_name_existing_aircraft() {
        let aircraft = create_test_aircraft();
        let display_name = aircraft_service::get_display_name(&aircraft, 1);
        assert_eq!(display_name, "Cessna 172");
    }

    #[test]
    fn test_get_display_name_nonexistent_aircraft() {
        let aircraft = create_test_aircraft();
        let display_name = aircraft_service::get_display_name(&aircraft, 999);
        assert_eq!(display_name, "Unknown Aircraft (ID: 999)");
    }

    #[test]
    fn test_get_display_name_empty_aircraft_list() {
        let aircraft = vec![];
        let display_name = aircraft_service::get_display_name(&aircraft, 1);
        assert_eq!(display_name, "Unknown Aircraft (ID: 1)");
    }

    #[test]
    fn test_get_display_name_multiple_aircraft() {
        let aircraft = create_test_aircraft();

        let display_name_1 = aircraft_service::get_display_name(&aircraft, 1);
        let display_name_2 = aircraft_service::get_display_name(&aircraft, 2);
        let display_name_3 = aircraft_service::get_display_name(&aircraft, 3);

        assert_eq!(display_name_1, "Cessna 172");
        assert_eq!(display_name_2, "Boeing 737-800");
        assert_eq!(display_name_3, "Airbus A320");
    }
}
