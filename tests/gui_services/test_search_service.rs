#[cfg(test)]
mod tests {
    use flight_planner::gui::data::{
        ListItemAircraft, ListItemAirport, ListItemHistory, TableItem,
    };
    use flight_planner::gui::services::SearchService;
    use flight_planner::models::Aircraft;
    use std::sync::Arc;

    /// Helper function to create a test airport table item.
    fn create_airport_item(name: &str, icao: &str, runway_length: &str) -> Arc<TableItem> {
        Arc::new(TableItem::Airport(ListItemAirport::new(
            name.to_string(),
            icao.to_string(),
            runway_length.to_string(),
        )))
    }

    /// Helper function to create a test aircraft for testing.
    fn create_test_aircraft(
        id: i32,
        manufacturer: &str,
        variant: &str,
        icao_code: &str,
        flown: i32,
    ) -> Aircraft {
        Aircraft {
            id,
            manufacturer: manufacturer.to_string(),
            variant: variant.to_string(),
            icao_code: icao_code.to_string(),
            flown,
            aircraft_range: 1500,
            category: "Commercial".to_string(),
            cruise_speed: 450,
            date_flown: None,
            takeoff_distance: Some(1000),
        }
    }

    /// Helper function to create an aircraft table item.
    fn create_aircraft_item(aircraft: &Aircraft) -> Arc<TableItem> {
        Arc::new(TableItem::Aircraft(ListItemAircraft::from_aircraft(
            aircraft,
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
        let result = SearchService::filter_items(&items, "");

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
        let result = SearchService::filter_items(&items, "london");

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
        let result_lower = SearchService::filter_items(&items, "london");
        let result_upper = SearchService::filter_items(&items, "LONDON");
        let result_mixed = SearchService::filter_items(&items, "LoNdOn");

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
        let result = SearchService::filter_items(&items, "nonexistent");

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
        let result = SearchService::filter_items(&items, "EGLL");

        // Assert
        assert_eq!(result.len(), 1);
        if let TableItem::Airport(airport) = result[0].as_ref() {
            assert_eq!(airport.icao, "EGLL");
        } else {
            panic!("Expected airport item");
        }
    }

    #[test]
    fn test_filter_items_aircraft_by_manufacturer() {
        // Arrange
        let aircraft1 = create_test_aircraft(1, "Boeing", "737-800", "B738", 0);
        let aircraft2 = create_test_aircraft(2, "Airbus", "A320", "A320", 1);
        let items = vec![
            create_aircraft_item(&aircraft1),
            create_aircraft_item(&aircraft2),
        ];

        // Act
        let result = SearchService::filter_items(&items, "boeing");

        // Assert
        assert_eq!(result.len(), 1);
        if let TableItem::Aircraft(aircraft) = result[0].as_ref() {
            assert_eq!(aircraft.get_manufacturer(), "Boeing");
        } else {
            panic!("Expected aircraft item");
        }
    }

    #[test]
    fn test_filter_items_aircraft_by_variant() {
        // Arrange
        let aircraft1 = create_test_aircraft(1, "Boeing", "737-800", "B738", 0);
        let aircraft2 = create_test_aircraft(2, "Boeing", "747-400", "B744", 1);
        let items = vec![
            create_aircraft_item(&aircraft1),
            create_aircraft_item(&aircraft2),
        ];

        // Act
        let result = SearchService::filter_items(&items, "737");

        // Assert
        assert_eq!(result.len(), 1);
        if let TableItem::Aircraft(aircraft) = result[0].as_ref() {
            assert_eq!(aircraft.get_variant(), "737-800");
        } else {
            panic!("Expected aircraft item");
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
        let result = SearchService::filter_items(&items, "EGLL");

        // Assert
        // History items don't support search, so no results expected
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_filter_items_mixed_types() {
        // Arrange
        let aircraft = create_test_aircraft(1, "Boeing", "737-800", "B738", 0);
        let items = vec![
            create_airport_item("London Heathrow", "EGLL", "3902 ft"),
            create_aircraft_item(&aircraft),
            create_history_item("1", "EGLL", "LFPG", "Boeing 737", "2023-01-01"),
        ];

        // Act
        let result = SearchService::filter_items(&items, "boeing");

        // Assert
        // Only aircraft item should match (history search is not implemented)
        assert_eq!(result.len(), 1);
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
        let result = SearchService::filter_items(&items, "lon");

        // Assert
        assert_eq!(result.len(), 2); // Should match both London airports
    }
}
