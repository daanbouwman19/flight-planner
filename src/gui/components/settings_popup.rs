use eframe::egui;

use crate::gui::events::Event;

pub struct SettingsPopupViewModel<'a> {
    pub api_key: &'a mut String,
}

pub struct SettingsPopup;

impl SettingsPopup {
    pub fn render(vm: &mut SettingsPopupViewModel, ctx: &egui::Context) -> Vec<Event> {
        let mut events = Vec::new();

        egui::Window::new("Settings")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("AVWX API Key:");
                    ui.text_edit_singleline(vm.api_key);
                });

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        events.push(Event::SaveSettings);
                    }
                    if ui.button("Cancel").clicked() {
                        events.push(Event::CloseSettingsPopup);
                    }
                });
            });

        events
    }
}
