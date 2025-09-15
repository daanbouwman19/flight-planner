use crate::gui::events::Event;
use egui::{Color32, RichText, TextEdit, Ui};

/// A view-model for the `SearchControls` component.
/// This simplifies the data passed to the component.
pub struct SearchControlsViewModel<'a> {
    pub query: &'a mut String,
}

/// The search controls component.
pub struct SearchControls;

impl SearchControls {
    /// Renders the search input and clear button.
    pub fn render(vm: &mut SearchControlsViewModel, ui: &mut Ui) -> Vec<Event> {
        let mut events = Vec::new();
        let mut changed = false;

        ui.horizontal(|ui| {
            ui.label("Search:");
            let response = ui.add(
                TextEdit::singleline(vm.query)
                    .hint_text("Enter search query...")
                    .desired_width(200.0),
            );
            if response.changed() {
                changed = true;
            }
            if ui
                .button(RichText::new("Clear").color(Color32::LIGHT_RED))
                .clicked()
            {
                vm.query.clear();
                events.push(Event::ClearSearch);
            }
        });

        if changed {
            events.push(Event::SearchQueryChanged);
        }

        events
    }
}
