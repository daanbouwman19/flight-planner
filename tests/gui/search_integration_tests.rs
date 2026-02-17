// Integration test for verifying the search functionality works correctly after the refactoring

use super::helpers::{perform_background_search, setup_integration_test_gui};
use flight_planner::gui::events::{AppEvent, UiEvent};

#[test]
fn test_search_query_changed_event_updates_search_service() {
    let mut gui = setup_integration_test_gui();

    // Simulate updating the search query through the view model
    {
        let search_query = gui.services.as_mut().unwrap().search.query_mut();
        search_query.clear();
        search_query.push_str("test query");
    }

    // Trigger the SearchQueryChanged event
    gui.handle_events(
        vec![AppEvent::Ui(UiEvent::SearchQueryChanged)],
        &egui::Context::default(),
    );

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
    let mut gui = setup_integration_test_gui();

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
    gui.handle_events(
        vec![AppEvent::Ui(UiEvent::ClearSearch)],
        &egui::Context::default(),
    );

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
    let mut gui = setup_integration_test_gui();

    // Set up some test data in all_items
    gui.update_displayed_items(); // This should populate some data

    // Use helper to perform background search
    // This replaces manual query setting and event handling + manual filtering
    let filtered_items = perform_background_search(&mut gui, "test");

    // Verify search state
    assert_eq!(gui.services.as_ref().unwrap().search.query(), "test");
    // perform_background_search sets pending to true
    assert!(gui.services.as_ref().unwrap().search.is_search_pending());

    // Manually update the service with the results (mimicking handle_background_task_results)
    gui.services
        .as_mut()
        .unwrap()
        .search
        .set_filtered_items(filtered_items);

    // Verify that get_displayed_items returns filtered results when there's a query
    let displayed_items = gui.get_displayed_items();
    // Should return filtered items since query is not empty
    assert_eq!(
        displayed_items.len(),
        gui.services.as_ref().unwrap().search.filtered_items().len()
    );
}
