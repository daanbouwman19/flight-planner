//! This module defines the UI component for weather filtering options.

use crate::gui::state::{FlightRules, WeatherFilterState};
use eframe::egui::Ui;
use strum::IntoEnumIterator;

/// A view model for the `WeatherControls` component.
pub struct WeatherControlsViewModel<'a> {
    /// A mutable reference to the weather filter state.
    pub weather_filter: &'a mut WeatherFilterState,
}

/// The UI component for weather filtering options.
pub struct WeatherControls;

impl WeatherControls {
    /// Renders the weather controls and returns `true` if any filter has changed.
    ///
    /// # Arguments
    ///
    /// * `vm` - The view model containing the weather filter state.
    /// * `ui` - The `eframe::egui::Ui` to render the component into.
    ///
    /// # Returns
    ///
    /// `true` if any of the weather filter settings were changed by the user.
    pub fn render(vm: &mut WeatherControlsViewModel, ui: &mut Ui) -> bool {
        let mut changed = false;
        ui.collapsing("Weather Filters", |ui| {
            if ui.checkbox(&mut vm.weather_filter.enabled, "Enable Weather Filtering").changed() {
                changed = true;
            }
            ui.add_enabled_ui(vm.weather_filter.enabled, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Max Wind (kts):");
                    if ui.text_edit_singleline(&mut vm.weather_filter.max_wind_speed).changed() {
                        changed = true;
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Min Wind (kts):");
                    if ui.text_edit_singleline(&mut vm.weather_filter.min_wind_speed).changed() {
                        changed = true;
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Min Vis (mi):");
                    if ui.text_edit_singleline(&mut vm.weather_filter.min_visibility).changed() {
                        changed = true;
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("Flight Rules:");
                    let original_rule = vm.weather_filter.flight_rules.clone();
                    egui::ComboBox::from_id_salt("flight_rules_combo")
                        .selected_text(original_rule.to_string())
                        .show_ui(ui, |ui| {
                            for rule in FlightRules::iter() {
                                ui.selectable_value(
                                    &mut vm.weather_filter.flight_rules,
                                    rule.clone(),
                                    rule.to_string(),
                                );
                            }
                        });
                    if vm.weather_filter.flight_rules != original_rule {
                        changed = true;
                    }
                });
            });
        });
        changed
    }
}