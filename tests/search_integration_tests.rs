// Integration test for verifying the search functionality works correctly after the refactoring

use flight_planner::test_helpers;
use flight_planner::gui::events::Event;
use flight_planner::gui::ui::Gui;

fn setup_gui() -> Gui {
    let database_pool = test_helpers::setup_database();
    Gui::new(
        &eframe::CreationContext::_new_kittest(egui::Context::default()),
        database_pool,
    )
    .unwrap()
}

#[test]
fn test_search_query_changed_event_updates_search_service() {
    let mut gui = setup_gui();

    // Simulate updating the search query through the view model
    {
        let search_query = gui.services.search.query_mut();
        search_query.clear();
        search_query.push_str("test query");
    }

    // Trigger the SearchQueryChanged event
    gui.handle_events(vec![Event::SearchQueryChanged]);

    // Verify that the search service was properly updated
    assert_eq!(gui.services.search.query(), "test query");
    assert!(gui.services.search.is_search_pending());
    assert!(gui.services.search.last_search_request().is_some());
}

#[test]
fn test_clear_search_event_clears_search_service() {
    let mut gui = setup_gui();

    // Set up some search state first
    gui.services.search.update_query("some query".to_string());
    assert!(!gui.services.search.query().is_empty());

    // Simulate clearing the query through the view model
    {
        let search_query = gui.services.search.query_mut();
        search_query.clear();
    }

    // Trigger the ClearSearch event
    gui.handle_events(vec![Event::ClearSearch]);

    // Verify that the search service was properly cleared
    assert!(gui.services.search.query().is_empty());
    assert!(!gui.services.search.is_search_pending());
    assert!(gui.services.search.last_search_request().is_none());
}

#[test]
fn test_search_functionality_end_to_end() {
    let mut gui = setup_gui();

    // Set up some test data in all_items
    gui.update_displayed_items(); // This should populate some data

    // Simulate a search query being entered
    {
        let search_query = gui.services.search.query_mut();
        search_query.clear();
        search_query.push_str("test");
    }

    // Trigger the search
    gui.handle_events(vec![Event::SearchQueryChanged]);

    // Verify search state
    assert_eq!(gui.services.search.query(), "test");
    assert!(gui.services.search.is_search_pending());

    // Force execute search manually (in real app this would be done by the background thread)
    let all_items = gui.state.all_items.clone();
    let filtered =
        flight_planner::gui::services::search_service::SearchService::filter_items_static(
            &all_items, "test",
        );
    gui.services.search.set_filtered_items(filtered);

    // Verify that get_displayed_items returns filtered results when there's a query
    let displayed_items = gui.get_displayed_items();
    // Should return filtered items since query is not empty
    assert_eq!(
        displayed_items.len(),
        gui.services.search.filtered_items().len()
    );
}
