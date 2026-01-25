#[cfg(test)]
mod tests {
    use flight_planner::gui::data::{
        ListItemAircraft, ListItemAirport, ListItemHistory, TableItem,
    };
    use flight_planner::traits::Searchable;

    /// Helper function to create a test airport table item.
    fn create_airport_item(name: &str, icao: &str, runway_length: &str) -> TableItem {
        TableItem::Airport(ListItemAirport::new(
            name.to_string(),
            icao.to_string(),
            runway_length.to_string(),
        ))
    }

    /// Helper function to create a test history table item.
    fn create_history_item(
        id: &str,
        departure: &str,
        arrival: &str,
        aircraft: &str,
        date: &str,
    ) -> TableItem {
        TableItem::History(ListItemHistory {
            id: id.to_string(),
            departure_icao: departure.to_string(),
            departure_info: format!("Departure Airport ({})", departure),
            arrival_icao: arrival.to_string(),
            arrival_info: format!("Arrival Airport ({})", arrival),
            aircraft_name: aircraft.to_string(),
            aircraft_id: 1, // Dummy ID for table item tests
            date: date.to_string(),
        })
    }

    /// Helper function to create a test aircraft table item.
    fn create_aircraft_item(
        manufacturer: &str,
        variant: &str,
        icao_code: &str,
        range: &str,
        category: &str,
        cruise_speed: &str,
        date_flown: &str,
    ) -> TableItem {
        TableItem::Aircraft(ListItemAircraft {
            id: 1,
            manufacturer: manufacturer.to_string(),
            variant: variant.to_string(),
            icao_code: icao_code.to_string(),
            flown: 0,
            range: range.to_string(),
            category: category.to_string(),
            cruise_speed: cruise_speed.to_string(),
            date_flown: date_flown.to_string(),
        })
    }

    #[test]
    fn test_search_score_basic_case_insensitive() {
        let airport = create_airport_item("London Heathrow", "EGLL", "3902 ft");

        assert!(airport.search_score("london") > 0);
        assert!(airport.search_score("LONDON") > 0);
        assert!(airport.search_score("LoNdOn") > 0);
        assert!(airport.search_score("heathrow") > 0);
        assert!(airport.search_score("EGLL") > 0);
        assert!(airport.search_score("egll") > 0);
    }

    #[test]
    fn test_search_score_unicode_correctness() {
        // Test Unicode case-insensitive matching
        let airport = create_airport_item("München Franz Josef Strauß", "EDDM", "4000 m");

        // Test German ß character - this does NOT convert to SS in standard to_lowercase()
        // Rust's to_lowercase() keeps ß as ß when lowercased
        assert!(airport.search_score("strauß") > 0);
        assert!(airport.search_score("Strauß") > 0);
        // Note: "STRAUSS" will NOT match "strauß" because ß != ss in Unicode case folding

        // Test ü character and its variations
        assert!(airport.search_score("münchen") > 0);
        assert!(airport.search_score("MÜNCHEN") > 0);
        assert!(airport.search_score("München") > 0);

        // Test partial matches with Unicode
        assert!(airport.search_score("franz") > 0);
        assert!(airport.search_score("FRANZ") > 0);
        assert!(airport.search_score("josef") > 0);
        assert!(airport.search_score("JOSEF") > 0);
    }

    #[test]
    fn test_search_score_unicode_edge_cases() {
        // Test with Turkish İ/i case conversion
        let airport = create_airport_item("İstanbul Airport", "LTFM", "3750 m");

        assert!(airport.search_score("İstanbul") > 0);
        // Note: İstanbul.to_lowercase() is "i̇stanbul" (with combining dot), not "istanbul"
        // so "istanbul" won't match "İstanbul" in standard Unicode case folding

        // Test with Greek characters
        let greek_airport = create_airport_item("Αθήνα Athens", "LGAV", "3800 m");
        assert!(greek_airport.search_score("αθήνα") > 0);
        assert!(greek_airport.search_score("ΑΘΉΝΑ") > 0);
        assert!(greek_airport.search_score("athens") > 0);
        assert!(greek_airport.search_score("ATHENS") > 0);
    }

    #[test]
    fn test_search_score_various_fields() {
        let history = create_history_item("123", "EDDM", "EGLL", "Airbus A380", "2024-01-15");

        // Test matching different fields - NOTE: ID is not searched in History items
        // Only departure_icao, arrival_icao, aircraft_name, and date are searched
        assert!(history.search_score("eddm") > 0);
        assert!(history.search_score("EGLL") > 0);
        assert!(history.search_score("airbus") > 0);
        assert!(history.search_score("A380") > 0);
        assert!(history.search_score("2024") > 0);
        assert!(history.search_score("01-15") > 0);
    }

    #[test]
    fn test_search_score_aircraft_fields() {
        let aircraft = create_aircraft_item(
            "Boeing",
            "737-800",
            "B738",
            "5400 km",
            "Medium",
            "850 km/h",
            "2024-03-15",
        );

        // Test fields that are actually searched in Aircraft items:
        // manufacturer, variant, icao_code, category, date_flown
        assert!(aircraft.search_score("boeing") > 0);
        assert!(aircraft.search_score("BOEING") > 0);
        assert!(aircraft.search_score("737") > 0);
        assert!(aircraft.search_score("800") > 0);
        assert!(aircraft.search_score("B738") > 0);
        assert!(aircraft.search_score("b738") > 0);
        assert!(aircraft.search_score("medium") > 0);
        assert!(aircraft.search_score("2024-03") > 0);

        // Note: range and cruise_speed fields are NOT searched according to the implementation
    }

    #[test]
    fn test_search_score_empty_and_nonmatching() {
        let airport = create_airport_item("London Heathrow", "EGLL", "3902 ft");

        // Empty query should result in a score. The service layer handles empty queries separately.
        assert!(airport.search_score("") > 0);

        // Non-matching queries
        assert!(airport.search_score("paris") == 0);
        assert!(airport.search_score("LFPG") == 0);
        assert!(airport.search_score("nonexistent") == 0);
        assert!(airport.search_score("1234567890") == 0);
    }

    #[test]
    fn test_search_score_accent_variations() {
        // Test airports with accented characters
        let airport = create_airport_item("São Paulo-Guarulhos", "SBGR", "3700 m");

        assert!(airport.search_score("são") > 0);
        assert!(airport.search_score("SÃO") > 0);
        assert!(airport.search_score("paulo") > 0);
        assert!(airport.search_score("PAULO") > 0);
        assert!(airport.search_score("guarulhos") > 0);
        assert!(airport.search_score("GUARULHOS") > 0);

        // Test French accents
        let french_airport = create_airport_item("Aéroport de Paris", "LFPG", "4200 m");
        assert!(french_airport.search_score("aéroport") > 0);
        assert!(french_airport.search_score("AÉROPORT") > 0);
        assert!(french_airport.search_score("paris") > 0);
        assert!(french_airport.search_score("PARIS") > 0);
    }

    #[test]
    fn test_regression_gross_strauss_example() {
        // This test demonstrates the Unicode correctness fix
        // The original issue was that ASCII-only case handling missed Unicode characters

        let airport = create_airport_item("Flughafen Groß-Gerau", "TEST", "1000 m");

        // Test Unicode-aware case insensitive search
        assert!(airport.search_score("groß") > 0);
        // Note: "groß" and "gross" are different characters in Unicode.
        // The ß character doesn't automatically convert to "ss" in standard case folding.

        // Test that the search handles each character set correctly
        let item_with_gross = create_airport_item("Airport GROSS", "TEST", "1000 m");
        assert!(item_with_gross.search_score("gross") > 0);
        assert!(item_with_gross.search_score("GROSS") > 0);

        // Test Unicode case folding with ß character
        let item_with_sz = create_airport_item("Flughafen Weiß", "TEST", "1000 m");
        assert!(item_with_sz.search_score("weiß") > 0);
        assert!(item_with_sz.search_score("WEIß") > 0);

        // The key point is that this now uses proper Unicode case folding
        // rather than ASCII-only case handling, preventing missed matches
        // for international airport names and data within the same Unicode equivalence class
    }

    #[test]
    fn test_unicode_vs_ascii_case_handling() {
        // This test demonstrates why Unicode-aware case handling is important
        // Previously, the ASCII-only implementation would miss these matches

        // Test that proper case insensitive matching works for Unicode
        let airport = create_airport_item("Zürich", "LSZH", "3700 m");
        assert!(airport.search_score("zürich") > 0);
        assert!(airport.search_score("ZÜRICH") > 0);
        assert!(airport.search_score("Zürich") > 0);

        // Test with Cyrillic characters
        let cyrillic_airport = create_airport_item("Москва", "UUEE", "3500 m");
        assert!(cyrillic_airport.search_score("москва") > 0);
        assert!(cyrillic_airport.search_score("МОСКВА") > 0);

        // The old ASCII-only implementation would have failed these tests
        // because it only handled ASCII a-z, A-Z transformations
    }

    #[test]
    fn test_table_item_columns_and_data() {
        use std::sync::Arc;
        let airport_item = create_airport_item("London", "EGLL", "10000ft");
        assert_eq!(
            airport_item.get_columns(),
            vec!["Name", "ICAO", "Longest Runway"]
        );
        let data = airport_item.get_data();
        assert_eq!(data[0], "London");
        assert_eq!(data[1], "EGLL");
        assert_eq!(data[2], "10000ft");

        let history_item = create_history_item("1", "EGLL", "LFPG", "B737", "2024-01-01");
        assert_eq!(
            history_item.get_columns(),
            vec!["ID", "Departure", "Arrival", "Aircraft", "Date"]
        );
        let data = history_item.get_data();
        assert_eq!(data[1], "EGLL");
        assert_eq!(data[3], "B737");

        let aircraft_item =
            create_aircraft_item("Boeing", "737", "B738", "3000NM", "Jet", "450KT", "Never");
        assert_eq!(aircraft_item.get_columns().len(), 8);
        let data = aircraft_item.get_data();
        assert_eq!(data[0], "Boeing");
        assert_eq!(data[2], "B738");

        let route_item = TableItem::Route(flight_planner::gui::data::ListItemRoute {
            departure: Arc::new(crate::common::create_test_airport(1, "Dep", "DEP")),
            destination: Arc::new(crate::common::create_test_airport(2, "Dest", "DEST")),
            aircraft: Arc::new(crate::common::create_test_aircraft(
                1, "Test", "Test", "TEST",
            )),
            departure_runway_length: 5000,
            destination_runway_length: 6000,
            route_length: 1000.0,
            aircraft_info: "Test AC".to_string().into(),
            departure_info: "Test Dep".to_string().into(),
            destination_info: "Test Dest".to_string().into(),
            distance_str: "1000.0 NM".to_string(),
            created_at: std::time::Instant::now(),
        });
        assert_eq!(route_item.get_columns().len(), 10);
        let data = route_item.get_data();
        assert_eq!(data[2], "5000ft");
        assert_eq!(data[5], "6000ft");
    }
}
