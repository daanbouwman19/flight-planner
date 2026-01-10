use flight_planner::gui::events::Event;
use flight_planner::gui::services::AppService;
use flight_planner::gui::services::Services;
use flight_planner::gui::services::popup_service::DisplayMode;
use flight_planner::gui::state::ApplicationState;
use flight_planner::gui::ui::Gui;
use flight_planner::test_helpers;
use std::sync::mpsc;

fn create_test_gui() -> Gui {
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
        current_route_generation_id: 0,
        scroll_to_top: false,
    }
}

#[test]
fn test_scroll_to_top_event() {
    let mut gui = create_test_gui();
    let ctx = egui::Context::default();

    // Verify initial state
    assert!(
        !gui.scroll_to_top,
        "scroll_to_top should be false initially"
    );

    // Trigger ScrollTableToTop event
    gui.handle_event(Event::ScrollTableToTop, &ctx);

    // Verify state change
    assert!(
        gui.scroll_to_top,
        "scroll_to_top should be true after ScrollTableToTop event"
    );
}

#[test]
fn test_scroll_to_top_on_display_mode_change() {
    let mut gui = create_test_gui();

    // Verify initial state
    assert!(
        !gui.scroll_to_top,
        "scroll_to_top should be false initially"
    );

    // Change display mode
    gui.process_display_mode_change(DisplayMode::History);

    // Verify state change
    assert!(
        gui.scroll_to_top,
        "scroll_to_top should be true after display mode change"
    );
}
