mod test_helpers;
use flight_planner::gui::ui::{Gui, RouteUpdateAction};
use flight_planner::gui::data::{TableItem, ListItemAirport};
use std::sync::Arc;
use std::time::Duration;

fn setup_gui() -> Gui {
    let db_pool = test_helpers::setup_database();
    Gui::new(
        &eframe::CreationContext::_new_kittest(egui::Context::default()),
        db_pool,
    )
    .unwrap()
}

#[test]
fn test_background_route_generation_sends_results() {
    let mut gui = setup_gui();
    gui.update_routes(RouteUpdateAction::Regenerate);

    let sender = gui.route_sender.clone();
    gui.services.app.spawn_route_generation_thread(
        flight_planner::gui::services::popup_service::DisplayMode::RandomRoutes,
        None,
        None,
        move |routes| {
            sender.send(routes).unwrap();
        },
    );

    let result = gui
        .route_receiver
        .recv_timeout(Duration::from_secs(5))
        .unwrap();

    assert!(!result.is_empty(), "Should receive some routes");
    let first_route = &result[0];
    assert_ne!(
        first_route.departure.ICAO, first_route.destination.ICAO,
        "Departure and destination should be different"
    );
}

#[test]
fn test_background_search_sends_filtered_results() {
    let mut gui = setup_gui();

    // Populate all_items with some test data
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
    gui.state.all_items = vec![item1.clone(), item2.clone()];
    gui.services.search.set_query("Airport A".to_string());
    gui.services.search.force_search_pending();

    let sender = gui.search_sender.clone();
    gui.services.search.spawn_search_thread(gui.state.all_items.clone(), move |filtered_items| {
        sender.send(filtered_items).unwrap();
    });

    let result = gui
        .search_receiver
        .recv_timeout(Duration::from_secs(5))
        .unwrap();

    assert_eq!(result.len(), 1, "Should find exactly one match");
    assert_eq!(
        result[0].as_ref(),
        item1.as_ref(),
        "The found item should be 'Airport A'"
    );
}
