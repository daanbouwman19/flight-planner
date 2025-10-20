use eframe::{egui, App};
use flight_planner::gui::data::{ListItemAircraft, ListItemAirport, TableItem};
use flight_planner::gui::events::Event;
use flight_planner::gui::services::popup_service::DisplayMode;
use flight_planner::gui::ui::Gui;
use flight_planner::models::{Aircraft, Airport};
use flight_planner::test_helpers;
use std::sync::Arc;
use std::time::Duration;

#[test]
fn test_search_functionality() {
    let db_pool = test_helpers::setup_database();
    let ctx = egui::Context::default();
    let cc = eframe::CreationContext::_new_kittest(ctx.clone());
    let mut gui = Gui::new(&cc, db_pool.clone()).unwrap();
    let mut frame = eframe::Frame::_new_kittest();

    // Manually populate the items list for the test
    let aircraft = Arc::new(Aircraft {
        id: 1,
        manufacturer: "TestAir".to_string(),
        variant: "T-1".to_string(),
        icao_code: "TEST".to_string(),
        flown: 0,
        aircraft_range: 1000,
        category: "A".to_string(),
        cruise_speed: 400,
        date_flown: None,
        takeoff_distance: Some(4000),
    });
    let airport = Arc::new(Airport {
        ID: 1,
        Name: "Airport A".to_string(),
        ICAO: "AAAA".to_string(),
        PrimaryID: None,
        Latitude: 0.0,
        Longtitude: 0.0,
        Elevation: 0,
        TransitionAltitude: None,
        TransitionLevel: None,
        SpeedLimit: None,
        SpeedLimitAltitude: None,
    });
    gui.state.all_items = vec![
        Arc::new(TableItem::Aircraft(ListItemAircraft::new(&aircraft))),
        Arc::new(TableItem::Airport(ListItemAirport {
            name: airport.Name.clone(),
            icao: airport.ICAO.clone(),
            longest_runway_length: "14000".to_string(),
        })),
    ];
    gui.services.popup.set_display_mode(DisplayMode::Other);

    // 1. Initial State: No search query, all items displayed
    assert!(gui.services.search.query().is_empty());
    assert_eq!(
        gui.get_displayed_items().len(),
        gui.state.all_items.len()
    );

    // 2. Typing a search query
    *gui.services.search.query_mut() = "TestAir".to_string();
    gui.handle_events(vec![Event::SearchQueryChanged], &ctx);

    // Wait for debouncing and search to complete
    std::thread::sleep(Duration::from_millis(100));
    let _ = ctx.run(Default::default(), |ctx| {
        gui.update(ctx, &mut frame);
    });
    let _ = ctx.run(Default::default(), |ctx| {
        gui.update(ctx, &mut frame);
    });

    assert!(!gui.services.search.filtered_items().is_empty());
    assert!(gui
        .services
        .search
        .filtered_items()
        .iter()
        .all(|item| format!("{:?}", item).contains("TestAir")));

    // 3. Clearing the search
    *gui.services.search.query_mut() = "".to_string();
    gui.handle_events(vec![Event::ClearSearch], &ctx);
    assert!(gui.services.search.query().is_empty());
    assert!(gui.services.search.filtered_items().is_empty());

    // 4. Searching for a specific airport
    gui.services.popup.set_display_mode(DisplayMode::Airports);
    *gui.services.search.query_mut() = "AAAA".to_string();
    gui.handle_events(vec![Event::SearchQueryChanged], &ctx);
    std::thread::sleep(Duration::from_millis(100));
    let _ = ctx.run(Default::default(), |ctx| {
        gui.update(ctx, &mut frame);
    });
    let _ = ctx.run(Default::default(), |ctx| {
        gui.update(ctx, &mut frame);
    });

    assert_eq!(gui.services.search.filtered_items().len(), 1);
    let item = gui.services.search.filtered_items().first().unwrap();
    if let TableItem::Airport(airport) = &**item {
        assert_eq!(airport.icao, "AAAA");
    } else {
        panic!("Expected an airport item");
    }
}
