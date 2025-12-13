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
                    ui.add(egui::TextEdit::singleline(vm.api_key).hint_text("Enter your AVWX API key..."));
                });

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui
                        .button("Save")
                        .on_hover_text("Save settings to database")
                        .clicked()
                    {
                        events.push(Event::SaveSettings);
                    }
                    if ui
                        .button("Cancel")
                        .on_hover_text("Close without saving")
                        .clicked()
                    {
                        events.push(Event::CloseSettingsPopup);
                    }
                });
            });

        events
    }
}
