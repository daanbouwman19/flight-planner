#[cfg(test)]
mod tests {
    use flight_planner::gui::data::{
        ListItemAircraft, ListItemAirport, ListItemHistory, TableItem,
    };

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
            arrival_icao: arrival.to_string(),
            aircraft_name: aircraft.to_string(),
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
    fn test_matches_query_basic_case_insensitive() {
        let airport = create_airport_item("London Heathrow", "EGLL", "3902 ft");

        assert!(airport.matches_query("london"));
        assert!(airport.matches_query("LONDON"));
        assert!(airport.matches_query("LoNdOn"));
        assert!(airport.matches_query("heathrow"));
        assert!(airport.matches_query("EGLL"));
        assert!(airport.matches_query("egll"));
    }

    #[test]
    fn test_matches_query_unicode_correctness() {
        // Test Unicode case-insensitive matching
        let airport = create_airport_item("München Franz Josef Strauß", "EDDM", "4000 m");

        // Test German ß character - this does NOT convert to SS in standard to_lowercase()
        // Rust's to_lowercase() keeps ß as ß when lowercased
        assert!(airport.matches_query("strauß"));
        assert!(airport.matches_query("Strauß"));
        // Note: "STRAUSS" will NOT match "strauß" because ß != ss in Unicode case folding

        // Test ü character and its variations
        assert!(airport.matches_query("münchen"));
        assert!(airport.matches_query("MÜNCHEN"));
        assert!(airport.matches_query("München"));

        // Test partial matches with Unicode
        assert!(airport.matches_query("franz"));
        assert!(airport.matches_query("FRANZ"));
        assert!(airport.matches_query("josef"));
        assert!(airport.matches_query("JOSEF"));
    }

    #[test]
    fn test_matches_query_unicode_edge_cases() {
        // Test with Turkish İ/i case conversion
        let airport = create_airport_item("İstanbul Airport", "LTFM", "3750 m");

        assert!(airport.matches_query("İstanbul"));
        // Note: İstanbul.to_lowercase() is "i̇stanbul" (with combining dot), not "istanbul"
        // so "istanbul" won't match "İstanbul" in standard Unicode case folding

        // Test with Greek characters
        let greek_airport = create_airport_item("Αθήνα Athens", "LGAV", "3800 m");
        assert!(greek_airport.matches_query("αθήνα"));
        assert!(greek_airport.matches_query("ΑΘΉΝΑ"));
        assert!(greek_airport.matches_query("athens"));
        assert!(greek_airport.matches_query("ATHENS"));
    }

    #[test]
    fn test_matches_query_various_fields() {
        let history = create_history_item("123", "EDDM", "EGLL", "Airbus A380", "2024-01-15");

        // Test matching different fields - NOTE: ID is not searched in History items
        // Only departure_icao, arrival_icao, aircraft_name, and date are searched
        assert!(history.matches_query("eddm"));
        assert!(history.matches_query("EGLL"));
        assert!(history.matches_query("airbus"));
        assert!(history.matches_query("A380"));
        assert!(history.matches_query("2024"));
        assert!(history.matches_query("01-15"));
    }

    #[test]
    fn test_matches_query_aircraft_fields() {
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
        assert!(aircraft.matches_query("boeing"));
        assert!(aircraft.matches_query("BOEING"));
        assert!(aircraft.matches_query("737"));
        assert!(aircraft.matches_query("800"));
        assert!(aircraft.matches_query("B738"));
        assert!(aircraft.matches_query("b738"));
        assert!(aircraft.matches_query("medium"));
        assert!(aircraft.matches_query("2024-03"));

        // Note: range and cruise_speed fields are NOT searched according to the implementation
    }

    #[test]
    fn test_matches_query_empty_and_nonmatching() {
        let airport = create_airport_item("London Heathrow", "EGLL", "3902 ft");

        // Empty query should match (returns true for empty queries)
        assert!(airport.matches_query(""));

        // Non-matching queries
        assert!(!airport.matches_query("paris"));
        assert!(!airport.matches_query("LFPG"));
        assert!(!airport.matches_query("nonexistent"));
        assert!(!airport.matches_query("1234567890"));
    }

    #[test]
    fn test_matches_query_lower_optimization() {
        let airport = create_airport_item("Test Airport", "TEST", "1000 ft");

        // Test that the optimized version works the same as the regular version
        let query = "Test";
        let query_lower = query.to_lowercase();

        assert_eq!(
            airport.matches_query(query),
            airport.matches_query_lower(&query_lower)
        );

        // Test with Unicode characters
        let unicode_airport = create_airport_item("Zürich Airport", "LSZH", "3700 m");
        let unicode_query = "ZÜRICH";
        let unicode_query_lower = unicode_query.to_lowercase();

        assert_eq!(
            unicode_airport.matches_query(unicode_query),
            unicode_airport.matches_query_lower(&unicode_query_lower)
        );
    }

    #[test]
    fn test_matches_query_accent_variations() {
        // Test airports with accented characters
        let airport = create_airport_item("São Paulo-Guarulhos", "SBGR", "3700 m");

        assert!(airport.matches_query("são"));
        assert!(airport.matches_query("SÃO"));
        assert!(airport.matches_query("paulo"));
        assert!(airport.matches_query("PAULO"));
        assert!(airport.matches_query("guarulhos"));
        assert!(airport.matches_query("GUARULHOS"));

        // Test French accents
        let french_airport = create_airport_item("Aéroport de Paris", "LFPG", "4200 m");
        assert!(french_airport.matches_query("aéroport"));
        assert!(french_airport.matches_query("AÉROPORT"));
        assert!(french_airport.matches_query("paris"));
        assert!(french_airport.matches_query("PARIS"));
    }

    #[test]
    fn test_regression_gross_strauss_example() {
        // This test demonstrates the Unicode correctness fix
        // The original issue was that ASCII-only case handling missed Unicode characters

        let airport = create_airport_item("Flughafen Groß-Gerau", "TEST", "1000 m");

        // Test Unicode-aware case insensitive search
        assert!(airport.matches_query("groß"));
        // Note: "groß" and "gross" are different characters in Unicode.
        // The ß character doesn't automatically convert to "ss" in standard case folding.

        // Test that the search handles each character set correctly
        let item_with_gross = create_airport_item("Airport GROSS", "TEST", "1000 m");
        assert!(item_with_gross.matches_query("gross"));
        assert!(item_with_gross.matches_query("GROSS"));

        // Test Unicode case folding with ß character
        let item_with_sz = create_airport_item("Flughafen Weiß", "TEST", "1000 m");
        assert!(item_with_sz.matches_query("weiß"));
        assert!(item_with_sz.matches_query("WEIß"));

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
        assert!(airport.matches_query("zürich"));
        assert!(airport.matches_query("ZÜRICH"));
        assert!(airport.matches_query("Zürich"));

        // Test with Cyrillic characters
        let cyrillic_airport = create_airport_item("Москва", "UUEE", "3500 m");
        assert!(cyrillic_airport.matches_query("москва"));
        assert!(cyrillic_airport.matches_query("МОСКВА"));

        // The old ASCII-only implementation would have failed these tests
        // because it only handled ASCII a-z, A-Z transformations
    }
}
