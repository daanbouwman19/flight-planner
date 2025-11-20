// Integration test for verifying the search functionality works correctly after the refactoring

use flight_planner::gui::events::Event;
use flight_planner::gui::ui::Gui;
use flight_planner::test_helpers;

fn setup_gui() -> Gui {
    let _database_pool = test_helpers::setup_database(); // Keep for now, might be needed for other tests or future changes
    let mut gui = Gui::new(&eframe::CreationContext::_new_kittest(
        egui::Context::default(),
    ))
    .unwrap();

    // Wait for services to initialize
    if let Some(receiver) = &gui.startup_receiver {
        use std::time::Duration;
        if let Ok(services) = receiver.recv_timeout(Duration::from_secs(30)) {
            gui.services = Some(services.unwrap());
        } else {
            panic!("Failed to initialize services in test");
        }
    }

    gui
}

#[test]
fn test_search_query_changed_event_updates_search_service() {
    let mut gui = setup_gui();

    // Simulate updating the search query through the view model
    {
        let search_query = gui.services.as_mut().unwrap().search.query_mut();
        search_query.clear();
        search_query.push_str("test query");
    }

    // Trigger the SearchQueryChanged event
    gui.handle_events(vec![Event::SearchQueryChanged], &egui::Context::default());

    // Verify that the search service was properly updated
    assert_eq!(gui.services.as_ref().unwrap().search.query(), "test query");
    assert!(gui.services.as_ref().unwrap().search.is_search_pending());
    assert!(
        gui.services
            .as_ref()
            .unwrap()
            .search
            .last_search_request()
            .is_some()
    );
}

#[test]
fn test_clear_search_event_clears_search_service() {
    let mut gui = setup_gui();

    // Set up some search state first
    gui.services
        .as_mut()
        .unwrap()
        .search
        .update_query("some query".to_string());
    assert!(!gui.services.as_ref().unwrap().search.query().is_empty());

    // Simulate clearing the query through the view model
    {
        let search_query = gui.services.as_mut().unwrap().search.query_mut();
        search_query.clear();
    }

    // Trigger the ClearSearch event
    gui.handle_events(vec![Event::ClearSearch], &egui::Context::default());

    // Verify that the search service was properly cleared
    assert!(gui.services.as_ref().unwrap().search.query().is_empty());
    assert!(!gui.services.as_ref().unwrap().search.is_search_pending());
    assert!(
        gui.services
            .as_ref()
            .unwrap()
            .search
            .last_search_request()
            .is_none()
    );
}

#[test]
fn test_search_functionality_end_to_end() {
    let mut gui = setup_gui();

    // Set up some test data in all_items
    gui.update_displayed_items(); // This should populate some data

    // Simulate a search query being entered
    {
        let search_query = gui.services.as_mut().unwrap().search.query_mut();
        search_query.clear();
        search_query.push_str("test");
    }

    // Trigger the search
    gui.handle_events(vec![Event::SearchQueryChanged], &egui::Context::default());

    // Verify search state
    assert_eq!(gui.services.as_ref().unwrap().search.query(), "test");
    assert!(gui.services.as_ref().unwrap().search.is_search_pending());

    // Force execute search manually (in real app this would be done by the background thread)
    let all_items = gui.state.all_items.clone();
    let filtered =
        flight_planner::gui::services::search_service::SearchService::filter_items_static(
            &all_items, "test",
        );
    gui.services
        .as_mut()
        .unwrap()
        .search
        .set_filtered_items(filtered);

    // Verify that get_displayed_items returns filtered results when there's a query
    let displayed_items = gui.get_displayed_items();
    // Should return filtered items since query is not empty
    assert_eq!(
        displayed_items.len(),
        gui.services.as_ref().unwrap().search.filtered_items().len()
    );
}
