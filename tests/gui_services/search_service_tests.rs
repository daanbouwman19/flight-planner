use flight_planner::gui::data::{ListItemAirport, TableItem};
use flight_planner::gui::services::search_service::SearchService;
use std::sync::{Arc, mpsc};
use std::time::Duration;

// Helper to create a test airport item
fn create_airport_item(name: &str, icao: &str) -> Arc<TableItem> {
    Arc::new(TableItem::Airport(ListItemAirport::new(
        name.to_string(),
        icao.to_string(),
        "10000ft".to_string(),
    )))
}

#[test]
fn test_filter_items_static_prioritizes_icao_matches() {
    let items = vec![
        create_airport_item("London Heathrow", "EGLL"),
        create_airport_item("Los Angeles", "KLAX"),
    ];

    // Search for "LAX" - should match KLAX (ICAO) with higher score
    let results = SearchService::filter_items_static(&items, "LAX");
    assert_eq!(results.len(), 1);
    assert_eq!(*results[0], *create_airport_item("Los Angeles", "KLAX"));

    // Search for "London" - should match London Heathrow (name)
    let results = SearchService::filter_items_static(&items, "London");
    assert_eq!(results.len(), 1);
    assert_eq!(*results[0], *create_airport_item("London Heathrow", "EGLL"));
}

#[test]
fn test_filter_items_static_sorts_by_score() {
    let items = vec![
        create_airport_item("LCY Airport", "EGLC"), // Name match, score 1
        create_airport_item("London City", "LCY"),  // ICAO match, score 2
    ];

    let results = SearchService::filter_items_static(&items, "LCY");
    assert_eq!(results.len(), 2);
    // First result should be the ICAO match (score 2)
    assert_eq!(*results[0], *create_airport_item("London City", "LCY"));
    // Second result should be the name match (score 1)
    assert_eq!(*results[1], *create_airport_item("LCY Airport", "EGLC"));
}

#[test]
fn test_filter_items_static_no_matches() {
    let items = vec![
        create_airport_item("Paris Charles de Gaulle", "LFPG"),
        create_airport_item("Tokyo Haneda", "RJTT"),
    ];

    let results = SearchService::filter_items_static(&items, "Berlin");
    assert!(results.is_empty());
}

#[test]
fn test_filter_items_static_empty_query_returns_all() {
    let items = vec![
        create_airport_item("Sydney Kingsford Smith", "YSSY"),
        create_airport_item("Dubai International", "OMDB"),
    ];

    let results = SearchService::filter_items_static(&items, "");
    assert_eq!(results.len(), 2);
}

#[test]
fn test_filter_items_static_case_insensitive() {
    let items = vec![
        create_airport_item("Amsterdam Schiphol", "EHAM"),
        create_airport_item("Frankfurt Airport", "EDDF"),
    ];

    // Case-insensitive ICAO search
    let results = SearchService::filter_items_static(&items, "eham");
    assert_eq!(results.len(), 1);
    assert_eq!(
        *results[0],
        *create_airport_item("Amsterdam Schiphol", "EHAM")
    );

    // Case-insensitive name search
    let results = SearchService::filter_items_static(&items, "frankfurt");
    assert_eq!(results.len(), 1);
    assert_eq!(
        *results[0],
        *create_airport_item("Frankfurt Airport", "EDDF")
    );
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
