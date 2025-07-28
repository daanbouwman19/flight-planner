use crate::gui::ui::Gui;
use eframe::egui::{self, TextEdit};
use std::time::{Duration, Instant};

impl Gui<'_> {
    /// Updates the search bar UI component.
    ///
    /// # Arguments
    ///
    /// * `ui` - The UI context.
    pub(super) fn update_search_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Search:");
            let response = ui.add(
                TextEdit::singleline(&mut self.state.search_state.query)
                    .hint_text("Type to search..."),
            );

            // Only trigger search when the text actually changes
            if response.changed() {
                // Set up debouncing: mark that a search is pending and record the time
                self.state.search_state.search_pending = true;
                self.state.search_state.last_search_request = Some(Instant::now());
            }
        });

        // Handle debounced search execution
        if self.state.search_state.search_pending {
            if let Some(last_request_time) = self.state.search_state.last_search_request {
                // Check if enough time has passed since the last search request (300ms debounce)
                if last_request_time.elapsed() >= Duration::from_millis(300) {
                    self.handle_search();
                    self.state.search_state.search_pending = false;
                }
            }
        }
    }
}
