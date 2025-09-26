#[cfg(test)]
mod tests {
    use flight_planner::gui::data::{ListItemAirport, ListItemHistory, TableItem};
    use flight_planner::gui::services::SearchService;
    use std::sync::{Arc, mpsc};
    use std::time::Duration;

    /// Helper function to create a test airport table item.
    fn create_airport_item(name: &str, icao: &str, runway_length: &str) -> Arc<TableItem> {
        Arc::new(TableItem::Airport(ListItemAirport::new(
            name.to_string(),
            icao.to_string(),
            runway_length.to_string(),
        )))
    }

    /// Helper function to create a history table item.
    fn create_history_item(
        id: &str,
        departure: &str,
        arrival: &str,
        aircraft: &str,
        date: &str,
    ) -> Arc<TableItem> {
        Arc::new(TableItem::History(ListItemHistory {
            id: id.to_string(),
            departure_icao: departure.to_string(),
            arrival_icao: arrival.to_string(),
            aircraft_name: aircraft.to_string(),
            date: date.to_string(),
        }))
    }

    #[test]
    fn test_filter_items_empty_query_returns_all_items() {
        // Arrange
        let items = vec![
            create_airport_item("London Heathrow", "EGLL", "3902 ft"),
            create_airport_item("Charles de Gaulle", "LFPG", "4215 ft"),
            create_airport_item("JFK International", "KJFK", "4423 ft"),
        ];

        // Act
        let result = SearchService::filter_items_static(&items, "");

        // Assert
        assert_eq!(result.len(), 3);
        // Since TableItem doesn't implement PartialEq, we verify the contents manually
        assert_eq!(result.len(), items.len());
    }

    #[test]
    fn test_filter_items_with_query_filters_correctly() {
        // Arrange
        let items = vec![
            create_airport_item("London Heathrow", "EGLL", "3902 ft"),
            create_airport_item("Charles de Gaulle", "LFPG", "4215 ft"),
            create_airport_item("JFK International", "KJFK", "4423 ft"),
        ];

        // Act
        let result = SearchService::filter_items_static(&items, "london");

        // Assert
        assert_eq!(result.len(), 1);
        if let TableItem::Airport(airport) = result[0].as_ref() {
            assert_eq!(airport.name, "London Heathrow");
        } else {
            panic!("Expected airport item");
        }
    }

    #[test]
    fn test_filter_items_case_insensitive() {
        // Arrange
        let items = vec![
            create_airport_item("London Heathrow", "EGLL", "3902 ft"),
            create_airport_item("Charles de Gaulle", "LFPG", "4215 ft"),
        ];

        // Act
        let result_lower = SearchService::filter_items_static(&items, "london");
        let result_upper = SearchService::filter_items_static(&items, "LONDON");
        let result_mixed = SearchService::filter_items_static(&items, "LoNdOn");

        // Assert
        assert_eq!(result_lower.len(), 1);
        assert_eq!(result_upper.len(), 1);
        assert_eq!(result_mixed.len(), 1);
    }

    #[test]
    fn test_filter_items_no_matches_returns_empty() {
        // Arrange
        let items = vec![
            create_airport_item("London Heathrow", "EGLL", "3902 ft"),
            create_airport_item("Charles de Gaulle", "LFPG", "4215 ft"),
        ];

        // Act
        let result = SearchService::filter_items_static(&items, "nonexistent");

        // Assert
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_filter_items_matches_icao_codes() {
        // Arrange
        let items = vec![
            create_airport_item("London Heathrow", "EGLL", "3902 ft"),
            create_airport_item("Charles de Gaulle", "LFPG", "4215 ft"),
        ];

        // Act
        let result = SearchService::filter_items_static(&items, "EGLL");

        // Assert
        assert_eq!(result.len(), 1);
        if let TableItem::Airport(airport) = result[0].as_ref() {
            assert_eq!(airport.icao, "EGLL");
        } else {
            panic!("Expected airport item");
        }
    }

    #[test]
    fn test_filter_items_history_by_icao() {
        // Arrange
        let items = vec![
            create_history_item("1", "EGLL", "LFPG", "Boeing 737", "2023-01-01"),
            create_history_item("2", "KJFK", "EGLL", "Airbus A320", "2023-01-02"),
        ];

        // Act
        let result = SearchService::filter_items_static(&items, "EGLL");

        // Assert
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_filter_items_history_by_aircraft() {
        // Arrange
        let items = vec![
            create_history_item("1", "EGLL", "LFPG", "Boeing 737", "2023-01-01"),
            create_history_item("2", "KJFK", "EGLL", "Airbus A320", "2023-01-02"),
        ];

        // Act
        let result = SearchService::filter_items_static(&items, "Boeing");

        // Assert
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_filter_items_history_by_date() {
        // Arrange
        let items = vec![
            create_history_item("1", "EGLL", "LFPG", "Boeing 737", "2023-01-01"),
            create_history_item("2", "KJFK", "EGLL", "Airbus A320", "2023-01-02"),
        ];

        // Act
        let result = SearchService::filter_items_static(&items, "2023-01-01");

        // Assert
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_filter_items_mixed_types() {
        // Arrange
        let items = vec![
            create_airport_item("London Heathrow", "EGLL", "3902 ft"),
            create_airport_item("Paris Charles de Gaulle", "LFPG", "4215 ft"),
            create_history_item("1", "EGLL", "LFPG", "Boeing 737", "2023-01-01"),
        ];

        // Act
        let result = SearchService::filter_items_static(&items, "EGLL");

        // Assert
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_filter_items_partial_matches() {
        // Arrange
        let items = vec![
            create_airport_item("London Heathrow", "EGLL", "3902 ft"),
            create_airport_item("London Gatwick", "EGKK", "3316 ft"),
            create_airport_item("Manchester", "EGCC", "3200 ft"),
        ];

        // Act
        let result = SearchService::filter_items_static(&items, "lon");

        // Assert
        assert_eq!(result.len(), 2); // Should match both London airports
    }

    #[test]
    fn test_filter_items_static_prioritizes_icao_matches() {
        let items = vec![
            create_airport_item("London Heathrow", "EGLL", "12802ft"),
            create_airport_item("Los Angeles", "KLAX", "12091ft"),
        ];

        // Search for "LAX" - should match KLAX (ICAO) with higher score
        let results = SearchService::filter_items_static(&items, "LAX");
        assert_eq!(results.len(), 1);
        if let TableItem::Airport(airport) = results[0].as_ref() {
            assert_eq!(airport.icao, "KLAX");
        } else {
            panic!("Expected airport item");
        }

        // Search for "London" - should match London Heathrow (name)
        let results = SearchService::filter_items_static(&items, "London");
        assert_eq!(results.len(), 1);
        if let TableItem::Airport(airport) = results[0].as_ref() {
            assert_eq!(airport.name, "London Heathrow");
        } else {
            panic!("Expected airport item");
        }
    }

    #[test]
    fn test_filter_items_static_sorts_by_score() {
        let items = vec![
            create_airport_item("LCY Airport", "EGLC", "4948ft"), // Name match, score 1
            create_airport_item("London City", "LCY", "4948ft"),  // ICAO match, score 2
        ];

        let results = SearchService::filter_items_static(&items, "LCY");
        assert_eq!(results.len(), 2);
        // First result should be the ICAO match (score 2)
        if let TableItem::Airport(airport) = results[0].as_ref() {
            assert_eq!(airport.icao, "LCY");
        } else {
            panic!("Expected airport item");
        }
        // Second result should be the name match (score 1)
        if let TableItem::Airport(airport) = results[1].as_ref() {
            assert_eq!(airport.icao, "EGLC");
        } else {
            panic!("Expected airport item");
        }
    }

    #[test]
    fn test_spawn_search_thread_calls_callback() {
        let search_service = SearchService::new();
        let (tx, rx) = mpsc::channel();

        let item1 = Arc::new(TableItem::Airport(ListItemAirport::new(
            "Airport A".to_string(),
            "AAAA".to_string(),
            "10000ft".to_string(),
        )));
        let item2 = Arc::new(TableItem::Airport(ListItemAirport::new(
            "Airport B".to_string(),
            "BBBB".to_string(),
            "12000ft".to_string(),
        )));
        let all_items = vec![item1.clone(), item2.clone()];

        search_service.spawn_search_thread(all_items, move |filtered_items| {
            tx.send(filtered_items)
                .expect("Test channel should accept search results");
        });

        let received_items = rx
            .recv_timeout(Duration::from_secs(5))
            .expect("Test should complete within 5 seconds");
        assert_eq!(received_items.len(), 2);
    }
}
