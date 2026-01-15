use eframe::egui;

use crate::gui::events::{AppEvent, DataEvent, UiEvent};

pub struct SettingsPopupViewModel<'a> {
    pub api_key: &'a mut String,
}

pub struct SettingsPopup;

impl SettingsPopup {
    #[cfg(not(tarpaulin_include))]
    pub fn render(vm: &mut SettingsPopupViewModel, ctx: &egui::Context) -> Vec<AppEvent> {
        let mut events = Vec::new();

        egui::Window::new("Settings")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("AVWX API Key:")
                        .on_hover_text("Required for real-time weather data");

                    let id = ui.make_persistent_id("settings_show_password");
                    let mut show_password = ui.data(|d| d.get_temp(id).unwrap_or(false));

                    let has_text = !vm.api_key.is_empty();
                    let response = ui.add(
                        egui::TextEdit::singleline(vm.api_key)
                            .password(!show_password)
                            .hint_text("Enter key...")
                            .desired_width(250.0),
                    );

                    let clear_button_size = 20.0;
                    if has_text {
                        if ui
                            .add_sized(
                                [clear_button_size, clear_button_size],
                                egui::Button::new("×").small().frame(false),
                            )
                            .on_hover_text("Clear API Key")
                            .clicked()
                        {
                            vm.api_key.clear();
                            response.request_focus();
                        }
                    } else {
                        ui.allocate_exact_size(
                            egui::Vec2::new(clear_button_size, clear_button_size),
                            egui::Sense::hover(),
                        );
                    }

                    let toggle_tooltip = if show_password {
                        "Hide API Key"
                    } else {
                        "Show API Key"
                    };

                    if ui
                        .add(egui::Button::new("👁").selected(show_password))
                        .on_hover_text(toggle_tooltip)
                        .clicked()
                    {
                        show_password = !show_password;
                        ui.data_mut(|d| d.insert_temp(id, show_password));
                    }
                });

                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.label("Don't have a key?");
                    ui.hyperlink_to("Get one at avwx.rest ➜", "https://account.avwx.rest/")
                        .on_hover_text("Opens in your default browser");
                });

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui
                        .button("💾 Save")
                        .on_hover_text("Save changes and close")
                        .clicked()
                    {
                        events.push(AppEvent::Data(DataEvent::SaveSettings));
                    }
                    if ui
                        .button("❌ Cancel")
                        .on_hover_text("Discard changes and close")
                        .clicked()
                    {
                        events.push(AppEvent::Ui(UiEvent::CloseSettingsPopup));
                    }
                });
            });

        events
    }
}
