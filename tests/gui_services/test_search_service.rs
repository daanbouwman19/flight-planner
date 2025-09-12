#[cfg(test)]
mod tests {
    use flight_planner::gui::data::{ListItemAirport, ListItemHistory, TableItem};
    use flight_planner::gui::services::SearchService;
    use std::sync::Arc;

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
    fn test_filter_items_history_by_icao() {
        // Arrange
        let items = vec![
            create_history_item("1", "EGLL", "LFPG", "Boeing 737", "2023-01-01"),
            create_history_item("2", "KJFK", "EGLL", "Airbus A320", "2023-01-02"),
        ];

        // Act
        let result = SearchService::filter_items(&items, "EGLL");

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
        let result = SearchService::filter_items(&items, "Boeing");

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
        let result = SearchService::filter_items(&items, "2023-01-01");

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
        let result = SearchService::filter_items(&items, "EGLL");

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
        let result = SearchService::filter_items(&items, "lon");

        // Assert
        assert_eq!(result.len(), 2); // Should match both London airports
    }
}
