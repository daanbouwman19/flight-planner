use std::time::{Duration, Instant};

use egui::{TextEdit, Ui};

use crate::gui::ui::Gui;

impl Gui<'_> {
    /// Updates the search bar UI component with improved encapsulation.
    /// This method demonstrates cleaner state management through getter/setter methods.
    ///
    /// # Arguments
    ///
    /// * `ui` - The UI context.
    pub fn update_search_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Search:");
            
            // Create a local copy for the text edit to avoid borrowing issues
            let mut current_query = self.get_search_query().to_string();
            let response = ui.add(
                TextEdit::singleline(&mut current_query).hint_text("Type to search..."),
            );

            // Only trigger search when the text actually changes
            if response.changed() {
                self.update_search_query(current_query);
            }
        });

        // Handle debounced search execution using encapsulated state access
        self.handle_debounced_search_execution();
    }
    
    /// Handles the debounced search execution logic.
    /// This separates the timing logic from the UI rendering.
    fn handle_debounced_search_execution(&mut self) {
        if self.is_search_pending() {
            if let Some(last_request_time) = self.get_last_search_request() {
                // Check if enough time has passed since the last search request (300ms debounce)
                if last_request_time.elapsed() >= Duration::from_millis(300) {
                    self.handle_search();
                    self.set_search_pending(false);
                }
            }
        }
    }
}

/// Pure component function for rendering search bar.
/// This function demonstrates the future direction for component separation.
/// Currently not used to avoid breaking changes, but shows the pattern.
///
/// # Arguments
///
/// * `ui` - The UI context
/// * `search_query` - Mutable reference to the search query string
/// * `search_state` - Current search state for debouncing
///
/// # Returns
///
/// Returns true if the search query changed.
#[allow(dead_code)]
pub fn render_search_component(
    ui: &mut Ui,
    search_query: &mut String,
    search_pending: &mut bool,
    last_search_request: &mut Option<Instant>
) -> bool {
    let mut query_changed = false;
    
    ui.horizontal(|ui| {
        ui.label("Search:");
        let response = ui.add(
            TextEdit::singleline(search_query).hint_text("Type to search..."),
        );

        // Only trigger search when the text actually changes
        if response.changed() {
            // Set up debouncing: mark that a search is pending and record the time
            *search_pending = true;
            *last_search_request = Some(Instant::now());
            query_changed = true;
        }
    });
    
    query_changed
}
