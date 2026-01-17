use crate::gui::events::{AppEvent, UiEvent};
use egui::{Key, Modifiers, Ui};

#[cfg(target_os = "macos")]
const SEARCH_HINT_TEXT: &str = "Filter items... (Cmd+F)";
#[cfg(not(target_os = "macos"))]
const SEARCH_HINT_TEXT: &str = "Filter items... (Ctrl+F)";

#[cfg(target_os = "macos")]
const SHORTCUT_TEXT: &str = "Cmd+F";
#[cfg(not(target_os = "macos"))]
const SHORTCUT_TEXT: &str = "Ctrl+F";

const CLEAR_BUTTON_TOOLTIP: &str = "Clear current search filter";
const SEARCH_INPUT_ID: &str = "main_search_input";

// --- View Model ---

/// A view model that provides a mutable reference to the search query string.
///
/// This allows the `SearchControls` component to directly modify the search
/// query state held by the parent.
pub struct SearchControlsViewModel<'a> {
    /// A mutable reference to the search query string.
    pub query: &'a mut String,
}

// --- Component ---

/// A UI component that renders the search input field and a clear button.
pub struct SearchControls;

impl SearchControls {
    /// Renders the search controls, including a text input and a clear button.
    ///
    /// This method modifies the search query directly through the view model and
    /// emits events when the query changes or is cleared.
    ///
    /// # Arguments
    ///
    /// * `vm` - A mutable reference to the `SearchControlsViewModel`.
    /// * `ui` - A mutable reference to the `egui::Ui` context for rendering.
    /// * `events` - A mutable reference to the event buffer.
    #[cfg(not(tarpaulin_include))]
    pub fn render(vm: &mut SearchControlsViewModel, ui: &mut Ui, events: &mut Vec<AppEvent>) {
        // Create the ID in the parent scope to ensure consistency.
        // ui.horizontal creates a new scope, so IDs generated inside it would differ.
        let search_input_id = ui.make_persistent_id(SEARCH_INPUT_ID);

        // Handle keyboard shortcut (Ctrl+F or Cmd+F) to focus search.
        // We check this when the UI is enabled to allow global search focus.
        if ui.is_enabled() {
            let consumed = ui.input_mut(|i| i.consume_key(Modifiers::COMMAND, Key::F));
            if consumed {
                ui.memory_mut(|m| m.request_focus(search_input_id));
            }
        }

        ui.horizontal(|ui| {
            // Search icon with tooltip hinting at the shortcut
            ui.add(egui::Label::new("🔍").sense(egui::Sense::hover()))
                .on_hover_text(format!("Search / Filter ({})", SHORTCUT_TEXT));

            let has_text = !vm.query.is_empty();
            let clear_button_size = 20.0;
            let spacing = 5.0;
            // Reserve space for the clear button if visible, plus standard padding/safety
            let reserved_width = if has_text {
                clear_button_size + spacing
            } else {
                0.0
            } + 10.0;

            let response = ui.add(
                egui::TextEdit::singleline(vm.query)
                    .hint_text(SEARCH_HINT_TEXT)
                    .id(search_input_id)
                    .desired_width(ui.available_width() - reserved_width),
            );

            if response.changed() {
                // The query in the parent state is already updated via the mutable reference.
                // Send an event to notify the parent to react (e.g., filter).
                events.push(AppEvent::Ui(UiEvent::SearchQueryChanged));
            }

            if has_text
                && ui
                    .add_sized(
                        [clear_button_size, clear_button_size],
                        egui::Button::new("×").small().frame(false),
                    )
                    .on_hover_text(CLEAR_BUTTON_TOOLTIP)
                    .clicked()
            {
                // Clear the query in the parent state for immediate UI feedback.
                vm.query.clear();
                // Send an event to notify the parent to react (e.g., clear filters).
                events.push(AppEvent::Ui(UiEvent::ClearSearch));
                // Return focus to the search input so the user can type immediately.
                response.request_focus();
            }
        });
    }
}
