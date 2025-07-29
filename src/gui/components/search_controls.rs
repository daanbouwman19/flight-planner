use crate::gui::ui::Gui;
use egui::Ui;

pub struct SearchControls;

impl SearchControls {
    /// Renders search controls.
    pub fn render(gui: &mut Gui, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Search:");
            let mut query = gui.get_search_query().to_string();
            let response = ui.text_edit_singleline(&mut query);

            if response.changed() {
                gui.update_search_query(query);
                gui.update_filtered_items();
            }

            if ui.button("🗑 Clear").clicked() {
                gui.update_search_query(String::new());
                gui.update_filtered_items();
            }
        });
    }
}
