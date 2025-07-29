use flight_planner::gui::data::ListItemHistory;
use flight_planner::gui::services::history_service;

fn create_test_history() -> Vec<ListItemHistory> {
    vec![
        ListItemHistory {
            id: "1".to_string(),
            departure_icao: "KJFK".to_string(),
            arrival_icao: "KLAX".to_string(),
            aircraft_name: "Boeing 737-800".to_string(),
            date: "2024-01-15".to_string(),
        },
        ListItemHistory {
            id: "2".to_string(),
            departure_icao: "EGLL".to_string(),
            arrival_icao: "KJFK".to_string(),
            aircraft_name: "Airbus A320".to_string(),
            date: "2024-02-20".to_string(),
        },
        ListItemHistory {
            id: "3".to_string(),
            departure_icao: "KLAX".to_string(),
            arrival_icao: "EGLL".to_string(),
            aircraft_name: "Cessna 172".to_string(),
            date: "2024-03-10".to_string(),
        },
        ListItemHistory {
            id: "4".to_string(),
            departure_icao: "KJFK".to_string(),
            arrival_icao: "EGLL".to_string(),
            aircraft_name: "Boeing 747-400".to_string(),
            date: "2024-01-05".to_string(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_items_empty_query() {
        let history = create_test_history();
        let filtered = history_service::filter_items(&history, "");

        assert_eq!(filtered.len(), 4);
        assert_eq!(filtered[0].departure_icao, "KJFK");
        assert_eq!(filtered[1].departure_icao, "EGLL");
        assert_eq!(filtered[2].departure_icao, "KLAX");
        assert_eq!(filtered[3].departure_icao, "KJFK");
    }

    #[test]
    fn test_filter_items_by_departure_icao() {
        let history = create_test_history();
        let filtered = history_service::filter_items(&history, "KJFK");

        assert_eq!(filtered.len(), 3); // KJFK appears in departure for items 1,4 and arrival for item 2
        assert!(filtered
            .iter()
            .any(|item| item.id == "1" && item.departure_icao == "KJFK"));
        assert!(filtered
            .iter()
            .any(|item| item.id == "2" && item.arrival_icao == "KJFK"));
        assert!(filtered
            .iter()
            .any(|item| item.id == "4" && item.departure_icao == "KJFK"));
    }

    #[test]
    fn test_filter_items_by_arrival_icao() {
        let history = create_test_history();
        let filtered = history_service::filter_items(&history, "EGLL");

        assert_eq!(filtered.len(), 3); // EGLL appears in arrival for items 3,4 and departure for item 2
        assert!(filtered
            .iter()
            .any(|item| item.id == "2" && item.departure_icao == "EGLL"));
        assert!(filtered
            .iter()
            .any(|item| item.id == "3" && item.arrival_icao == "EGLL"));
        assert!(filtered
            .iter()
            .any(|item| item.id == "4" && item.arrival_icao == "EGLL"));
    }

    #[test]
    fn test_filter_items_by_aircraft_name() {
        let history = create_test_history();
        let filtered = history_service::filter_items(&history, "Boeing");

        assert_eq!(filtered.len(), 2);
        assert!(filtered[0].aircraft_name.contains("Boeing"));
        assert!(filtered[1].aircraft_name.contains("Boeing"));
    }

    #[test]
    fn test_filter_items_by_date() {
        let history = create_test_history();
        let filtered = history_service::filter_items(&history, "2024-01");

        assert_eq!(filtered.len(), 2);
        assert!(filtered[0].date.starts_with("2024-01"));
        assert!(filtered[1].date.starts_with("2024-01"));
    }

    #[test]
    fn test_filter_items_case_insensitive() {
        let history = create_test_history();
        let filtered = history_service::filter_items(&history, "boeing");

        assert_eq!(filtered.len(), 2);
        assert!(filtered[0].aircraft_name.contains("Boeing"));
        assert!(filtered[1].aircraft_name.contains("Boeing"));
    }

    #[test]
    fn test_filter_items_partial_match() {
        let history = create_test_history();
        let filtered = history_service::filter_items(&history, "737");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].aircraft_name, "Boeing 737-800");
    }

    #[test]
    fn test_filter_items_no_matches() {
        let history = create_test_history();
        let filtered = history_service::filter_items(&history, "nonexistent");

        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_filter_items_empty_history() {
        let history = vec![];
        let filtered = history_service::filter_items(&history, "anything");

        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_sort_items_by_departure_ascending() {
        let mut history = create_test_history();
        history_service::sort_items(&mut history, "departure", true);

        assert_eq!(history[0].departure_icao, "EGLL");
        assert_eq!(history[1].departure_icao, "KJFK");
        assert_eq!(history[2].departure_icao, "KJFK");
        assert_eq!(history[3].departure_icao, "KLAX");
    }

    #[test]
    fn test_sort_items_by_departure_descending() {
        let mut history = create_test_history();
        history_service::sort_items(&mut history, "departure", false);

        assert_eq!(history[0].departure_icao, "KLAX");
        assert_eq!(history[1].departure_icao, "KJFK");
        assert_eq!(history[2].departure_icao, "KJFK");
        assert_eq!(history[3].departure_icao, "EGLL");
    }

    #[test]
    fn test_sort_items_by_arrival_ascending() {
        let mut history = create_test_history();
        history_service::sort_items(&mut history, "arrival", true);

        assert_eq!(history[0].arrival_icao, "EGLL");
        assert_eq!(history[1].arrival_icao, "EGLL");
        assert_eq!(history[2].arrival_icao, "KJFK");
        assert_eq!(history[3].arrival_icao, "KLAX");
    }

    #[test]
    fn test_sort_items_by_arrival_descending() {
        let mut history = create_test_history();
        history_service::sort_items(&mut history, "arrival", false);

        assert_eq!(history[0].arrival_icao, "KLAX");
        assert_eq!(history[1].arrival_icao, "KJFK");
        assert_eq!(history[2].arrival_icao, "EGLL");
        assert_eq!(history[3].arrival_icao, "EGLL");
    }

    #[test]
    fn test_sort_items_by_aircraft_ascending() {
        let mut history = create_test_history();
        history_service::sort_items(&mut history, "aircraft", true);

        assert_eq!(history[0].aircraft_name, "Airbus A320");
        assert_eq!(history[1].aircraft_name, "Boeing 737-800");
        assert_eq!(history[2].aircraft_name, "Boeing 747-400");
        assert_eq!(history[3].aircraft_name, "Cessna 172");
    }

    #[test]
    fn test_sort_items_by_aircraft_descending() {
        let mut history = create_test_history();
        history_service::sort_items(&mut history, "aircraft", false);

        assert_eq!(history[0].aircraft_name, "Cessna 172");
        assert_eq!(history[1].aircraft_name, "Boeing 747-400");
        assert_eq!(history[2].aircraft_name, "Boeing 737-800");
        assert_eq!(history[3].aircraft_name, "Airbus A320");
    }

    #[test]
    fn test_sort_items_by_date_ascending() {
        let mut history = create_test_history();
        history_service::sort_items(&mut history, "date", true);

        assert_eq!(history[0].date, "2024-01-05");
        assert_eq!(history[1].date, "2024-01-15");
        assert_eq!(history[2].date, "2024-02-20");
        assert_eq!(history[3].date, "2024-03-10");
    }

    #[test]
    fn test_sort_items_by_date_descending() {
        let mut history = create_test_history();
        history_service::sort_items(&mut history, "date", false);

        assert_eq!(history[0].date, "2024-03-10");
        assert_eq!(history[1].date, "2024-02-20");
        assert_eq!(history[2].date, "2024-01-15");
        assert_eq!(history[3].date, "2024-01-05");
    }

    #[test]
    fn test_sort_items_unknown_column() {
        let original_history = create_test_history();
        let mut history = original_history.clone();

        history_service::sort_items(&mut history, "unknown_column", true);

        // Should remain unchanged
        assert_eq!(history, original_history);
    }

    #[test]
    fn test_sort_items_empty_history() {
        let mut history: Vec<ListItemHistory> = vec![];
        history_service::sort_items(&mut history, "departure", true);

        assert_eq!(history.len(), 0);
    }

    #[test]
    fn test_sort_items_single_item() {
        let mut history = vec![ListItemHistory {
            id: "1".to_string(),
            departure_icao: "KJFK".to_string(),
            arrival_icao: "KLAX".to_string(),
            aircraft_name: "Boeing 737-800".to_string(),
            date: "2024-01-15".to_string(),
        }];

        history_service::sort_items(&mut history, "departure", true);

        assert_eq!(history.len(), 1);
        assert_eq!(history[0].departure_icao, "KJFK");
    }

    #[test]
    fn test_combined_filter_and_sort() {
        let history = create_test_history();
        let filtered = history_service::filter_items(&history, "KJFK");

        assert_eq!(filtered.len(), 3); // KJFK matches 3 items

        let mut filtered_vec = filtered;
        history_service::sort_items(&mut filtered_vec, "date", true);

        // Should be sorted by date ascending
        assert_eq!(filtered_vec[0].date, "2024-01-05"); // Item 4
        assert_eq!(filtered_vec[1].date, "2024-01-15"); // Item 1
        assert_eq!(filtered_vec[2].date, "2024-02-20"); // Item 2
    }

    #[test]
    fn test_stability_of_sort() {
        // Create items with same departure but different arrival
        let mut history = vec![
            ListItemHistory {
                id: "1".to_string(),
                departure_icao: "KJFK".to_string(),
                arrival_icao: "KLAX".to_string(),
                aircraft_name: "Boeing 737-800".to_string(),
                date: "2024-01-15".to_string(),
            },
            ListItemHistory {
                id: "2".to_string(),
                departure_icao: "KJFK".to_string(),
                arrival_icao: "EGLL".to_string(),
                aircraft_name: "Airbus A320".to_string(),
                date: "2024-01-15".to_string(),
            },
        ];

        // Sort by departure (should maintain relative order for equal elements)
        history_service::sort_items(&mut history, "departure", true);

        assert_eq!(history[0].departure_icao, "KJFK");
        assert_eq!(history[1].departure_icao, "KJFK");
        // The relative order should be maintained (stable sort)
        assert_eq!(history[0].arrival_icao, "KLAX");
        assert_eq!(history[1].arrival_icao, "EGLL");
    }
}
