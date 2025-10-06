//! This module defines the UI component for weather filtering options.

use crate::gui::state::WeatherFilterState;
use eframe::egui::{self, Ui};

/// A view model for the `WeatherControls` component.
pub struct WeatherControlsViewModel<'a> {
    /// A mutable reference to the weather filter state.
    pub weather_filter: &'a mut WeatherFilterState,
}

/// The UI component for weather filtering options.
pub struct WeatherControls;

impl WeatherControls {
    /// Renders the weather controls.
    ///
    /// # Arguments
    ///
    /// * `vm` - The view model containing the weather filter state.
    /// * `ui` - The `eframe::egui::Ui` to render the component into.
    pub fn render(vm: &mut WeatherControlsViewModel, ui: &mut Ui) {
        ui.collapsing("Weather Filters", |ui| {
            ui.checkbox(&mut vm.weather_filter.enabled, "Enable Weather Filtering");
            ui.add_enabled_ui(vm.weather_filter.enabled, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Max Wind (kts):");
                    ui.text_edit_singleline(&mut vm.weather_filter.max_wind_speed);
                });
                ui.horizontal(|ui| {
                    ui.label("Min Vis (mi):");
                    ui.text_edit_singleline(&mut vm.weather_filter.min_visibility);
                });
                ui.horizontal(|ui| {
                    ui.label("Flight Rules:");
                    ui.text_edit_singleline(&mut vm.weather_filter.flight_rules);
                });
            });
        });
    }
}