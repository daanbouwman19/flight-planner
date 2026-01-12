use super::helpers::create_test_gui;
use flight_planner::gui::events::{AppEvent, UiEvent};
use flight_planner::gui::services::popup_service::DisplayMode;

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
    gui.handle_events(vec![AppEvent::Ui(UiEvent::ScrollTableToTop)], &ctx);

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
