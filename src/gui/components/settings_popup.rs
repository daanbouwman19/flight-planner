use eframe::egui;

use crate::gui::events::{AppEvent, DataEvent, UiEvent};
use crate::gui::icons;

pub struct SettingsPopupViewModel<'a> {
    pub api_key: &'a mut String,
}

pub struct SettingsPopup;

impl SettingsPopup {
    #[cfg(not(tarpaulin_include))]
    pub fn render(vm: &mut SettingsPopupViewModel, ctx: &egui::Context) -> Vec<AppEvent> {
        let mut events = Vec::new();
        let mut open = true;

        egui::Window::new("Settings")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
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

                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        events.push(AppEvent::Data(DataEvent::SaveSettings));
                    }

                    let clear_button_size = 20.0;
                    if has_text {
                        if ui
                            .add_sized(
                                [clear_button_size, clear_button_size],
                                egui::Button::new(icons::ICON_CLEAR).small().frame(false),
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

                    let toggle_icon = if show_password {
                        icons::ICON_EYE_SLASH
                    } else {
                        icons::ICON_EYE
                    };

                    if ui
                        .add(egui::Button::new(toggle_icon).selected(show_password))
                        .on_hover_text(toggle_tooltip)
                        .clicked()
                    {
                        show_password = !show_password;
                        ui.data_mut(|d| d.insert_temp(id, show_password));
                    }

                    if has_text {
                        let copy_id = ui.make_persistent_id("settings_copy_key");
                        let now = ui.input(|i| i.time);
                        let copied_at: Option<f64> = ui.data(|d| d.get_temp(copy_id));

                        let tooltip = if let Some(t) = copied_at
                            && now - t < 2.0
                        {
                            format!("{} Copied!", icons::ICON_CHECK)
                        } else {
                            "Copy API Key".to_string()
                        };

                        if ui
                            .add(egui::Button::new(icons::ICON_CLIPBOARD))
                            .on_hover_text(tooltip)
                            .clicked()
                        {
                            ui.output_mut(|o| {
                                o.commands
                                    .push(egui::OutputCommand::CopyText(vm.api_key.clone()))
                            });
                            ui.data_mut(|d| d.insert_temp(copy_id, now));
                        }
                    }
                });

                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.label("Don't have a key?");
                    ui.hyperlink_to(
                        format!("Get one at avwx.rest {}", icons::ICON_EXTERNAL_LINK),
                        "https://account.avwx.rest/",
                    )
                    .on_hover_text("Opens in your default browser");
                });

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    #[cfg(target_os = "macos")]
                    let save_tooltip = "Save changes and close (Cmd+Enter)";
                    #[cfg(not(target_os = "macos"))]
                    let save_tooltip = "Save changes and close (Ctrl+Enter)";

                    if ui
                        .button(format!("{} Save", icons::ICON_SAVE))
                        .on_hover_text(save_tooltip)
                        .clicked()
                        || ui.input_mut(|i| {
                            i.consume_key(egui::Modifiers::COMMAND, egui::Key::Enter)
                        })
                    {
                        events.push(AppEvent::Data(DataEvent::SaveSettings));
                    }
                    if ui
                        .button(format!("{} Cancel", icons::ICON_CLOSE))
                        .on_hover_text("Discard changes and close (Esc)")
                        .clicked()
                        || ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
                    {
                        events.push(AppEvent::Ui(UiEvent::CloseSettingsPopup));
                    }
                });
            });

        if !open {
            events.push(AppEvent::Ui(UiEvent::CloseSettingsPopup));
        }

        events
    }
}
