use flight_planner::gui::data::TableItem;
use flight_planner::gui::services::AppService;
use flight_planner::gui::services::Services;
use flight_planner::gui::state::ApplicationState;
use flight_planner::gui::ui::Gui;
use flight_planner::test_helpers;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};

pub fn create_test_gui() -> Gui {
    let (route_sender, route_receiver) = mpsc::channel();
    let (search_sender, search_receiver) = mpsc::channel();
    let (weather_sender, weather_receiver) = mpsc::channel();
    let (airport_items_sender, airport_items_receiver) = mpsc::channel();

    let db = test_helpers::setup_database();
    let app_service = AppService::new(db).unwrap();
    let services = Services::new(app_service, "test_api_key".to_string());

    Gui {
        state: ApplicationState::new(),
        services: Some(services),
        startup_receiver: None,
        startup_error: None,
        route_sender,
        route_receiver,
        search_sender,
        search_receiver,
        weather_sender,
        weather_receiver,
        airport_items_sender,
        airport_items_receiver,
        route_update_request: None,
        is_loading_airport_items: false,
        current_route_generation_id: Arc::new(AtomicU64::new(0)),
        scroll_to_top: false,
    }
}

pub fn setup_integration_test_gui() -> Gui {
    let db_pool = test_helpers::setup_database();
    let mut gui = Gui::new(
        &eframe::CreationContext::_new_kittest(egui::Context::default()),
        Some(db_pool),
    )
    .unwrap();

    // Wait for services to initialize
    if let Some(receiver) = &gui.startup_receiver {
        if let Ok(services) = receiver.recv_timeout(Duration::from_secs(30)) {
            gui.services = Some(services.unwrap());
        } else {
            panic!("Failed to initialize services in test");
        }
    }

    gui
}

pub fn perform_background_search(gui: &mut Gui, query: &str) -> Vec<Arc<TableItem>> {
    gui.services
        .as_mut()
        .unwrap()
        .search
        .set_query(query.to_string());

    // Force debounce to pass so should_execute_search returns true
    gui.services.as_mut().unwrap().search.force_search_pending();

    // Use should_execute_search to clear the pending flag if debounce passed
    if !gui
        .services
        .as_mut()
        .unwrap()
        .search
        .should_execute_search()
    {
        panic!("Search should be ready to execute");
    }

    let sender = gui.search_sender.clone();
    gui.services.as_ref().unwrap().search.spawn_search_thread(
        gui.state.all_items.clone(),
        move |filtered_items| {
            sender
                .send(filtered_items)
                .expect("Failed to send search results");
        },
    );

    // Simulate the event loop processing results
    let start = Instant::now();
    let timeout = Duration::from_secs(30);

    // Increment active searches manually because we are bypassing spawn_background_tasks
    // which normally increments it. If we don't, handle_background_task_results might
    // decrement it below zero (if unsigned) or just mismatch.
    gui.services
        .as_mut()
        .unwrap()
        .search
        .increment_active_searches();

    while start.elapsed() < timeout {
        // Pump the event loop handler
        gui.handle_background_task_results(&egui::Context::default());

        // Check if search is complete
        // We consider search complete when filtered items are populated OR pending flag is cleared
        // But simply checking is_searching() (active_searches > 0) is better because
        // handle_background_task_results decrements it when results arrive.
        if !gui.services.as_ref().unwrap().search.is_searching() {
            return gui
                .services
                .as_ref()
                .unwrap()
                .search
                .filtered_items()
                .to_vec();
        }

        std::thread::sleep(Duration::from_millis(50));
    }

    panic!("Search timed out after {:?} waiting for results", timeout);
}
