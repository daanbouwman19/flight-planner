use super::helpers::create_test_gui;
use flight_planner::gui::services::popup_service::DisplayMode;

#[test]
fn test_reset_confirm_mode_resets_on_display_mode_change() {
    let mut gui = create_test_gui();

    // Set initial state
    gui.state.reset_confirm_mode = true;

    // Change display mode
    gui.process_display_mode_change(DisplayMode::Statistics);

    // Verify it was reset
    assert!(!gui.state.reset_confirm_mode);
}
