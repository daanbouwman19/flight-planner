use flight_planner::gui::data::{ListItemAirport, ListItemAircraft, ListItemHistory, TableItem};
use flight_planner::gui::services::SearchService;
use flight_planner::gui::state::ViewState;
use std::sync::Arc;

fn create_mock_airport(name: &str, icao: &str) -> Arc<TableItem> {
    Arc::new(TableItem::Airport(ListItemAirport {
        name: name.to_string(),
        icao: icao.to_string(),
        longest_runway_length: "5000ft".to_string(),
    }))
}

fn create_mock_aircraft(manufacturer: &str, variant: &str) -> Arc<TableItem> {
    Arc::new(TableItem::Aircraft(ListItemAircraft {
        id: 1,
        manufacturer: manufacturer.to_string(),
        variant: variant.to_string(),
        icao_code: "B738".to_string(),
        flown: 0,
        range: "3000 NM".to_string(),
        category: "Jet".to_string(),
        cruise_speed: "450 knots".to_string(),
        date_flown: "".to_string(),
    }))
}

fn create_mock_history(departure: &str, arrival: &str) -> Arc<TableItem> {
    Arc::new(TableItem::History(ListItemHistory {
        id: "1".to_string(),
        departure_icao: departure.to_string(),
        arrival_icao: arrival.to_string(),
        aircraft_name: "Boeing 737-800".to_string(),
        date: "2023-01-01".to_string(),
    }))
}

#[test]
fn test_filter_items_empty_query() {
    let items = vec![
        create_mock_airport("London Heathrow", "EGLL"),
        create_mock_airport("Paris Charles de Gaulle", "LFPG"),
    ];
    let mut view_state = ViewState::default();
    SearchService::filter_items(&mut view_state, &items);
    assert_eq!(view_state.filtered_items.len(), 2);
}

#[test]
fn test_filter_items_airport_name() {
    let items = vec![
        create_mock_airport("London Heathrow", "EGLL"),
        create_mock_airport("Paris Charles de Gaulle", "LFPG"),
    ];
    let mut view_state = ViewState {
        table_search: "london".to_string(),
        ..Default::default()
    };
    SearchService::filter_items(&mut view_state, &items);
    assert_eq!(view_state.filtered_items.len(), 1);
    let item = view_state.filtered_items.first().unwrap();
    if let TableItem::Airport(airport) = &**item {
        assert_eq!(airport.name, "London Heathrow");
    } else {
        panic!("Expected an airport item");
    }
}

#[test]
fn test_filter_items_case_insensitivity() {
    let items = vec![
        create_mock_airport("London Heathrow", "EGLL"),
        create_mock_airport("Paris Charles de Gaulle", "LFPG"),
    ];
    let mut view_state_lower = ViewState {
        table_search: "london".to_string(),
        ..Default::default()
    };
    SearchService::filter_items(&mut view_state_lower, &items);
    let mut view_state_upper = ViewState {
        table_search: "LONDON".to_string(),
        ..Default::default()
    };
    SearchService::filter_items(&mut view_state_upper, &items);
    let mut view_state_mixed = ViewState {
        table_search: "LoNdOn".to_string(),
        ..Default::default()
    };
    SearchService::filter_items(&mut view_state_mixed, &items);
    assert_eq!(view_state_lower.filtered_items.len(), 1);
    assert_eq!(view_state_upper.filtered_items.len(), 1);
    assert_eq!(view_state_mixed.filtered_items.len(), 1);
}

#[test]
fn test_filter_items_no_match() {
    let items = vec![
        create_mock_airport("London Heathrow", "EGLL"),
        create_mock_airport("Paris Charles de Gaulle", "LFPG"),
    ];
    let mut view_state = ViewState {
        table_search: "nonexistent".to_string(),
        ..Default::default()
    };
    SearchService::filter_items(&mut view_state, &items);
    assert!(view_state.filtered_items.is_empty());
}

#[test]
fn test_filter_items_airport_icao() {
    let items = vec![
        create_mock_airport("London Heathrow", "EGLL"),
        create_mock_airport("Paris Charles de Gaulle", "LFPG"),
    ];
    let mut view_state = ViewState {
        table_search: "EGLL".to_string(),
        ..Default::default()
    };
    SearchService::filter_items(&mut view_state, &items);
    assert_eq!(view_state.filtered_items.len(), 1);
    let item = view_state.filtered_items.first().unwrap();
    if let TableItem::Airport(airport) = &**item {
        assert_eq!(airport.icao, "EGLL");
    } else {
        panic!("Expected an airport item");
    }
}

#[test]
fn test_filter_items_multiple_types() {
    let items = vec![
        create_mock_airport("London Heathrow", "EGLL"),
        create_mock_aircraft("Boeing", "737-800"),
        create_mock_history("EGLL", "LFPG"),
    ];
    let mut view_state = ViewState {
        table_search: "EGLL".to_string(),
        ..Default::default()
    };
    SearchService::filter_items(&mut view_state, &items);
    assert_eq!(view_state.filtered_items.len(), 2);
}

#[test]
fn test_filter_items_aircraft_manufacturer() {
    let items = vec![
        create_mock_aircraft("Boeing", "737-800"),
        create_mock_aircraft("Airbus", "A320"),
    ];
    let mut view_state = ViewState {
        table_search: "Boeing".to_string(),
        ..Default::default()
    };
    SearchService::filter_items(&mut view_state, &items);
    assert_eq!(view_state.filtered_items.len(), 1);
}

#[test]
fn test_filter_items_history_date() {
    let items = vec![
        create_mock_history("EGLL", "LFPG"),
        create_mock_history("EDDF", "LEMD"),
    ];
    let mut view_state = ViewState {
        table_search: "2023-01-01".to_string(),
        ..Default::default()
    };
    SearchService::filter_items(&mut view_state, &items);
    assert_eq!(view_state.filtered_items.len(), 2);
}

#[test]
fn test_filter_items_partial_match() {
    let items = vec![
        create_mock_airport("London Heathrow", "EGLL"),
        create_mock_airport("Paris Charles de Gaulle", "LFPG"),
    ];
    let mut view_state = ViewState {
        table_search: "EGLL".to_string(),
        ..Default::default()
    };
    SearchService::filter_items(&mut view_state, &items);
    assert_eq!(view_state.filtered_items.len(), 1);
}

#[test]
fn test_filter_items_partial_match_name() {
    let items = vec![
        create_mock_airport("London Heathrow", "EGLL"),
        create_mock_airport("Paris Charles de Gaulle", "LFPG"),
    ];
    let mut view_state = ViewState {
        table_search: "lon".to_string(),
        ..Default::default()
    };
    SearchService::filter_items(&mut view_state, &items);
    assert_eq!(view_state.filtered_items.len(), 1);
}
