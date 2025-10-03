use crate::gui::events::Event;
use egui::Ui;

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
    ///
    /// # Returns
    ///
    /// A `Vec<Event>` containing `SearchQueryChanged` or `ClearSearch` events.
    pub fn render(vm: &mut SearchControlsViewModel, ui: &mut Ui) -> Vec<Event> {
        let mut events = Vec::new();
        ui.horizontal(|ui| {
            ui.label("Search:");
            let response = ui.text_edit_singleline(vm.query);

            if response.changed() {
                // The query in the parent state is already updated via the mutable reference.
                // Send an event to notify the parent to react (e.g., filter).
                events.push(Event::SearchQueryChanged);
            }

            if ui.button("🗑 Clear").clicked() {
                // Clear the query in the parent state for immediate UI feedback.
                vm.query.clear();
                // Send an event to notify the parent to react (e.g., clear filters).
                events.push(Event::ClearSearch);
            }
        });
        events
    }
}
