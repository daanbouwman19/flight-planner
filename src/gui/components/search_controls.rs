use crate::gui::events::Event;
use egui::Ui;

const SEARCH_HINT_TEXT: &str = "Filter items...";
const CLEAR_BUTTON_TOOLTIP: &str = "Clear current search filter";

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
    pub fn render(vm: &mut SearchControlsViewModel, ui: &mut Ui, events: &mut Vec<Event>) {
        ui.horizontal(|ui| {
            ui.label("🔍");

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
                    .desired_width(ui.available_width() - reserved_width),
            );

            if response.changed() {
                // The query in the parent state is already updated via the mutable reference.
                // Send an event to notify the parent to react (e.g., filter).
                events.push(Event::SearchQueryChanged);
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
                events.push(Event::ClearSearch);
                // Return focus to the search input so the user can type immediately.
                response.request_focus();
            }
        });
    }
}
