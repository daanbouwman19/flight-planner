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
            departure_info: format!("Departure Airport ({})", departure),
            arrival_icao: arrival.to_string(),
            arrival_info: format!("Arrival Airport ({})", arrival),
            aircraft_name: aircraft.to_string(),
            aircraft_id: 1, // Dummy ID for search tests
            date: date.to_string(),
        }))
    }

    #[test]
    fn test_filter_items_parameterized() {
        // Arrange
        let items = vec![
            create_airport_item("London Heathrow", "EGLL", "3902 ft"),
            create_airport_item("Charles de Gaulle", "LFPG", "4215 ft"),
            create_airport_item("JFK International", "KJFK", "4423 ft"),
        ];

        // Test cases: (query, expected_count, optional_expected_icao)
        let test_cases = vec![
            // Empty query
            ("", 3, None),
            // Name match (case sensitive in input, insensitive in search)
            ("london", 1, Some("EGLL")),
            // Case insensitive variants
            ("LONDON", 1, Some("EGLL")),
            ("LoNdOn", 1, Some("EGLL")),
            // No matches
            ("nonexistent", 0, None),
            // ICAO match
            ("EGLL", 1, Some("EGLL")),
            ("KJFK", 1, Some("KJFK")),
        ];

        for (query, expected_count, expected_icao) in test_cases {
            // Act
            let result = SearchService::filter_items_static(&items, query);

            // Assert
            assert_eq!(
                result.len(),
                expected_count,
                "Failed count check for query: '{}'",
                query
            );

            if let Some(expected) = expected_icao {
                // We don't need to check result.is_empty() here because expected_count > 0
                // for all cases with an expected_icao, and we already asserted the length.
                if let TableItem::Airport(airport) = result[0].as_ref() {
                    assert_eq!(
                        airport.icao, expected,
                        "Failed ICAO check for query: '{}'",
                        query
                    );
                } else {
                    panic!(
                        "Expected an airport item for query: '{}', but got {:?}",
                        query, result[0]
                    );
                }
            }
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
            panic!("Expected TableItem::Airport, but got {:?}", results[0]);
        }

        // Search for "London" - should match London Heathrow (name)
        let results = SearchService::filter_items_static(&items, "London");
        assert_eq!(results.len(), 1);
        if let TableItem::Airport(airport) = results[0].as_ref() {
            assert_eq!(airport.name, "London Heathrow");
        } else {
            panic!("Expected TableItem::Airport, but got {:?}", results[0]);
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
            panic!("Expected TableItem::Airport, but got {:?}", results[0]);
        }
        // Second result should be the name match (score 1)
        if let TableItem::Airport(airport) = results[1].as_ref() {
            assert_eq!(airport.icao, "EGLC");
        } else {
            panic!("Expected TableItem::Airport, but got {:?}", results[1]);
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

    #[test]
    fn test_parallel_search_execution() {
        // Create 6000 items to trigger parallel threshold (5000)
        let mut items = Vec::new();
        for i in 0..6000 {
            items.push(create_airport_item(
                &format!("Airport {}", i),
                &format!("A{:03}", i),
                "10000ft",
            ));
        }

        // Search for a specific one - use A5999 to avoid matching A5990-A5998
        let result = SearchService::filter_items_static(&items, "A5999");
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_search_service_state_management() {
        let mut service = SearchService::new();

        service.set_query("test".to_string());
        assert_eq!(service.query(), "test");
        assert!(service.is_search_pending());

        service.clear_query();
        assert_eq!(service.query(), "");
        assert!(!service.is_search_pending());

        service.update_query("hello".to_string());
        assert_eq!(service.query(), "hello");
        assert!(service.is_search_pending());
    }

    #[test]
    fn test_debouncing_logic() {
        let mut service = SearchService::new();

        service.set_query("test".to_string());
        // Should not execute immediately
        assert!(!service.should_execute_search());

        // Force it (as if time passed)
        service.force_search_pending();
        assert!(service.should_execute_search());
        // Should reset pending flag
        assert!(!service.is_search_pending());
    }

    #[test]
    fn test_results_management() {
        let mut service = SearchService::new();
        let items = vec![create_airport_item("Test", "TEST", "0ft")];

        service.set_filtered_items(items);
        assert!(service.has_results());
        assert_eq!(service.result_count(), 1);
        assert_eq!(service.filtered_items().len(), 1);
    }

    #[test]
    fn test_search_stability_sequential() {
        // Create 2 items that will have the same score
        // Item A (index 0)
        let item1 = create_airport_item("Stable Match", "STM1", "1000ft");
        // Item B (index 1)
        let item2 = create_airport_item("Stable Match", "STM2", "1000ft");

        let items = vec![item1.clone(), item2.clone()];

        // Search for "Stable Match" - should match both equally (score for name match)
        let result = SearchService::filter_items_static(&items, "Stable Match");

        assert_eq!(result.len(), 2);

        // Verify stability: item1 (STM1) should come before item2 (STM2)
        let airport1 = if let TableItem::Airport(a) = result[0].as_ref() {
            a
        } else {
            panic!("Expected airport item at index 0")
        };
        let airport2 = if let TableItem::Airport(a) = result[1].as_ref() {
            a
        } else {
            panic!("Expected airport item at index 1")
        };

        assert_eq!(airport1.icao, "STM1", "First item should be STM1 (index 0)");
        assert_eq!(
            airport2.icao, "STM2",
            "Second item should be STM2 (index 1)"
        );
    }

    #[test]
    fn test_search_stability_parallel() {
        // Create > 5000 items to trigger parallel search
        let mut items = Vec::with_capacity(6000);

        // Add padding items
        for i in 0..3000 {
            items.push(create_airport_item(&format!("Pad {}", i), "PAD", "0ft"));
        }

        // Add our target items in specific order
        let item1 = create_airport_item("Parallel Match", "PM1", "1000ft");
        items.push(item1.clone());

        let item2 = create_airport_item("Parallel Match", "PM2", "1000ft");
        items.push(item2.clone());

        // Add more padding
        for i in 0..3000 {
            items.push(create_airport_item(&format!("Pad {}", i), "PAD", "0ft"));
        }

        // Search for "Parallel Match"
        let result = SearchService::filter_items_static(&items, "Parallel Match");

        assert_eq!(result.len(), 2);

        // Verify stability: item1 (PM1) should come before item2 (PM2)
        // Since result order is determined by score then index
        let airport1 = if let TableItem::Airport(a) = result[0].as_ref() {
            a
        } else {
            panic!("Expected airport item at index 0")
        };
        let airport2 = if let TableItem::Airport(a) = result[1].as_ref() {
            a
        } else {
            panic!("Expected airport item at index 1")
        };

        assert_eq!(
            airport1.icao, "PM1",
            "First item should be PM1 (earlier index)"
        );
        assert_eq!(
            airport2.icao, "PM2",
            "Second item should be PM2 (later index)"
        );
    }
}
