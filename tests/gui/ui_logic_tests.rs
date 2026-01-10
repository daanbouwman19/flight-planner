use super::helpers::create_test_gui;
use flight_planner::gui::components::table_display::TableDisplay;
use flight_planner::gui::data::{ListItemAirport, TableItem};
use flight_planner::gui::services::popup_service::DisplayMode;
use flight_planner::models::Aircraft;
use std::sync::Arc;

// Helper to create a basic Gui instance

#[test]
fn test_table_display_calculate_default_widths_into() {
    let display_mode = DisplayMode::RandomRoutes;
    let available_width = 1000.0;
    let mut buffer = [0.0; 8];

    // From implementations:
    // RandomRoutes has 7 columns.
    // Fixed width = 80*2 + 80 + 100 = 340.
    // Flex = 660. 3 flex columns -> 220 each.
    // Expected widths: [220, 220, 80, 220, 80, 80, 100]

    let count =
        TableDisplay::calculate_default_widths_into(&display_mode, available_width, &mut buffer);

    assert_eq!(count, 7);
    assert_eq!(buffer[0], 220.0);
    assert_eq!(buffer[6], 100.0);
}

#[test]
fn test_table_display_should_load_more_routes() {
    // Parameters: item_count, scroll_position, content_height, viewport_height

    // Case 1: Not enough items
    assert!(!TableDisplay::should_load_more_routes(
        5, 0.0, 1000.0, 800.0
    ));

    // Case 2: Enough items, but not scrolled down enough
    // max_scroll = 2000 - 800 = 1200.
    // distance_from_bottom = 1200 - 0 = 1200 > 200 (threshold)
    assert!(!TableDisplay::should_load_more_routes(
        20, 0.0, 2000.0, 800.0
    ));

    // Case 3: Scrolled near bottom
    // max_scroll = 1200
    // distance = 1200 - 1100 = 100 < 200 (threshold)
    assert!(TableDisplay::should_load_more_routes(
        20, 1100.0, 2000.0, 800.0
    ));

    // Case 4: No scrollable content (content < viewport)
    // max_scroll = 0
    assert!(!TableDisplay::should_load_more_routes(
        20, 0.0, 500.0, 800.0
    ));
}

#[test]
fn test_gui_set_all_items() {
    let mut gui = create_test_gui();
    let item = Arc::new(TableItem::Airport(ListItemAirport::new(
        "A".to_string(),
        "ICAO".to_string(),
        "1000".to_string(),
    )));
    let items = vec![item];

    gui.set_all_items(items.clone());

    assert_eq!(gui.state.all_items.len(), 1);
    // Should also trigger update_filtered_items, so if query is empty, filtered should be empty?
    // Wait, update_filtered_items sets services.search.filtered_items.
    // But get_displayed_items returns filtered if query not empty, else all_items.

    assert_eq!(gui.get_displayed_items().len(), 1);
}

#[test]
fn test_gui_update_filtered_items() {
    let mut gui = create_test_gui();
    let item1 = Arc::new(TableItem::Airport(ListItemAirport::new(
        "Alpha".to_string(),
        "AAAA".to_string(),
        "1000".to_string(),
    )));
    let item2 = Arc::new(TableItem::Airport(ListItemAirport::new(
        "Beta".to_string(),
        "BBBB".to_string(),
        "1000".to_string(),
    )));
    gui.set_all_items(vec![item1, item2]);

    // 1. Set query "Alpha" directly in service (simulating UI input)
    if let Some(services) = &mut gui.services {
        services.search.set_query("Alpha".to_string());
    }

    // 2. Call update_filtered_items (Gui method)
    gui.update_filtered_items();

    // 3. Verify filtered items
    if let Some(services) = &gui.services {
        let filtered = services.search.filtered_items();
        assert_eq!(filtered.len(), 1);
        match filtered[0].as_ref() {
            TableItem::Airport(a) => assert_eq!(a.name, "Alpha"),
            _ => panic!("Wrong item type"),
        }
    }

    // 4. Test empty query
    if let Some(services) = &mut gui.services {
        services.search.set_query("   ".to_string()); // Empty/whitespace
    }
    gui.update_filtered_items();

    if let Some(services) = &gui.services {
        let filtered = services.search.filtered_items();
        assert!(filtered.is_empty());
    }
}

#[test]
fn test_gui_is_route_mode() {
    let mut gui = create_test_gui();

    // Default is usually RandomRoutes (route mode) or Statistics?
    // Let's set specific modes.
    if let Some(services) = &mut gui.services {
        services.popup.set_display_mode(DisplayMode::RandomRoutes);
    }
    assert!(gui.is_route_mode());

    if let Some(services) = &mut gui.services {
        services.popup.set_display_mode(DisplayMode::Statistics);
    }
    assert!(!gui.is_route_mode());
}

#[test]
fn test_gui_process_display_mode_change() {
    // This method does a lot, including spawning threads for Airports mode.
    // We'll test a simpler mode like Statistics or Other.
    let mut gui = create_test_gui();

    // Test switching to Statistics
    // Need to ensure DB has some data so stats are calculated?
    // setup_test_db inserts basic data.

    gui.process_display_mode_change(DisplayMode::Statistics);
    assert!(gui.state.statistics.is_some());

    // Test switching to RandomAirports
    gui.process_display_mode_change(DisplayMode::RandomAirports);
    assert!(!gui.state.all_items.is_empty()); // Should generate some random airports from DB (if DB populated)

    // Test switching to Other (Aircraft)
    gui.process_display_mode_change(DisplayMode::Other);
    assert!(!gui.state.all_items.is_empty());
    // Check item type
    match gui.state.all_items[0].as_ref() {
        TableItem::Aircraft(_) => {}
        _ => panic!("Expected Aircraft items"),
    }

    // Test switching to Airports (triggers thread)
    // We need to wait for the thread to send results back?
    // The method spawns a thread which sends to `airport_items_sender`.
    // The `Gui` struct has `airport_items_receiver`.
    // BUT `process_display_mode_change` DOES NOT receive. `Gui::update` receives.
    // So we just check that loading state is true, and maybe check the receiver if we can access it (we can't easily access the receiver since it's moved into Gui).
    // Actually `create_test_gui` moves the receiver into Gui.
    // We can't really verify the result unless we call `handle_background_task_results` but that's private or excluded?
    // Let's at least trigger the code path.
    gui.process_display_mode_change(DisplayMode::Airports);
    assert!(gui.is_loading_airport_items);

    // Test default case (RandomRoutes) - calls update_displayed_items
    gui.process_display_mode_change(DisplayMode::RandomRoutes);
    // update_displayed_items logic is excluded?
    // Just verify internal state change if any.
    // RandomRoutes in update_displayed_items generally shows routes.
    // Since we don't have routes generated, it might be empty.
    // But we hit the branch.
}

#[test]
fn test_gui_update_displayed_items() {
    let mut gui = create_test_gui();

    // 1. Test RandomRoutes - with no routes, should be empty
    if let Some(services) = &mut gui.services {
        services.popup.set_display_mode(DisplayMode::RandomRoutes);
        services.app.clear_route_items(); // Ensure empty
    }
    gui.update_displayed_items();
    assert!(gui.state.all_items.is_empty());

    // 2. Test RandomAirports - should load from DB
    if let Some(services) = &mut gui.services {
        services.popup.set_display_mode(DisplayMode::RandomAirports);
    }
    gui.update_displayed_items();
    assert!(!gui.state.all_items.is_empty());
    match gui.state.all_items[0].as_ref() {
        TableItem::Airport(_) => {}
        _ => panic!("Expected Airport items"),
    }

    // 3. Test Statistics - should return empty (handled elsewhere usually, but method has branch)
    if let Some(services) = &mut gui.services {
        services.popup.set_display_mode(DisplayMode::Statistics);
    }
    gui.update_displayed_items();
    assert!(gui.state.all_items.is_empty());

    // 4. Test History - should load history items
    if let Some(services) = &mut gui.services {
        services.popup.set_display_mode(DisplayMode::History);
    }
    gui.update_displayed_items();
    assert!(gui.state.all_items.is_empty()); // Empty initially
}

#[test]
fn test_gui_route_mode_helpers() {
    let mut gui = create_test_gui();

    // 1. get_appropriate_route_mode
    // No selected aircraft -> RandomRoutes
    assert_eq!(gui.get_appropriate_route_mode(), DisplayMode::RandomRoutes);

    // With selected aircraft -> SpecificAircraftRoutes
    let aircraft = Arc::new(Aircraft {
        id: 1,
        manufacturer: "Test".to_string(),
        variant: "Test".to_string(),
        icao_code: "TEST".to_string(),
        flown: 0,
        aircraft_range: 1000,
        category: "A".to_string(),
        cruise_speed: 100,
        date_flown: None,
        takeoff_distance: None,
    });
    gui.state.selected_aircraft = Some(aircraft);

    assert_eq!(
        gui.get_appropriate_route_mode(),
        DisplayMode::SpecificAircraftRoutes
    );

    // 2. maybe_switch_to_route_mode
    // Reset mode to something non-route
    if let Some(services) = &mut gui.services {
        services.popup.set_display_mode(DisplayMode::Statistics);
    }

    // Call with false -> no change
    gui.maybe_switch_to_route_mode(false);
    if let Some(services) = &gui.services {
        assert_eq!(*services.popup.display_mode(), DisplayMode::Statistics);
    }

    // Call with true -> changes to SpecificAircraftRoutes (since aircraft selected)
    gui.maybe_switch_to_route_mode(true);
    if let Some(services) = &gui.services {
        assert_eq!(
            *services.popup.display_mode(),
            DisplayMode::SpecificAircraftRoutes
        );
    }
}

#[test]
fn test_calculate_default_widths() {
    let available_width = 1000.0;
    let mut buffer = [0.0; 8];

    // Test Route modes
    let count = TableDisplay::calculate_default_widths_into(
        &DisplayMode::RandomRoutes,
        available_width,
        &mut buffer,
    );
    assert_eq!(count, 7);

    // Logic check: Sum of first count items should be close to calculated logic
    // Implementation details: exact = 80*2 + 80 + 100 = 340. Flex = 660. 3 flex cols => 220 each.
    // [220, 220, 80, 220, 80, 80, 100]
    assert_eq!(buffer[0], 220.0);
    assert_eq!(buffer[2], 80.0);

    // Test Airports mode
    let count = TableDisplay::calculate_default_widths_into(
        &DisplayMode::RandomAirports,
        available_width,
        &mut buffer,
    );
    // Airports: ICAO, Name, Runway Length => 3 cols
    assert_eq!(count, 3);
    assert!(buffer[0] > 0.0);

    // Test History mode
    let count = TableDisplay::calculate_default_widths_into(
        &DisplayMode::History,
        available_width,
        &mut buffer,
    );
    // History: Aircraft, From, To, Date => 4 cols
    assert_eq!(count, 4);
}
