use crate::gui::events::Event;
use egui::Ui;

// --- View Model ---

/// View-model for the `SearchControls` component.
pub struct SearchControlsViewModel<'a> {
    pub query: &'a mut String,
}

// --- Component ---

pub struct SearchControls;

impl SearchControls {
    /// Renders search controls.
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
