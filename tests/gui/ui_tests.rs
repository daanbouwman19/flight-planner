use flight_planner::gui::data::{ListItemAirport, TableItem};
use flight_planner::gui::ui::{Gui, RouteUpdateAction};
use flight_planner::test_helpers;
use std::sync::Arc;
use std::time::Duration;

fn setup_gui() -> Gui {
    let db_pool = test_helpers::setup_database();
    let mut gui = Gui::new(
        &eframe::CreationContext::_new_kittest(egui::Context::default()),
        Some(db_pool),
    )
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
fn test_background_route_generation_sends_results() {
    let mut gui = setup_gui();
    gui.update_routes(RouteUpdateAction::Regenerate);

    let sender = gui.route_sender.clone();
    gui.services
        .as_ref()
        .unwrap()
        .app
        .spawn_route_generation_thread(
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
    gui.state.all_items = vec![item1.clone(), item2.clone()];
    gui.services
        .as_mut()
        .unwrap()
        .search
        .set_query("Airport A".to_string());
    gui.services.as_mut().unwrap().search.force_search_pending();

    let sender = gui.search_sender.clone();
    gui.services.as_ref().unwrap().search.spawn_search_thread(
        gui.state.all_items.clone(),
        move |filtered_items| {
            sender.send(filtered_items).unwrap();
        },
    );

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
