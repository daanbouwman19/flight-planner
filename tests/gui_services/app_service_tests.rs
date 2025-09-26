use flight_planner::test_helpers::setup_database;
use flight_planner::gui::services::app_service::AppService;
use flight_planner::gui::services::popup_service::DisplayMode;
use std::sync::mpsc;
use std::time::Duration;

#[test]
fn test_spawn_route_generation_thread_calls_callback() {
    let db_pool = setup_database();
    let app_service = AppService::new(db_pool).unwrap();
    let (tx, rx) = mpsc::channel();

    app_service.spawn_route_generation_thread(
        DisplayMode::RandomRoutes,
        None,
        None,
        move |routes| {
            tx.send(routes).unwrap();
        },
    );

    let received_routes = rx.recv_timeout(Duration::from_secs(5)).unwrap();
    assert!(!received_routes.is_empty());
}
