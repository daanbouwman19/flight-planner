use crate::gui::events::Event;
use crate::gui::services::route_popup_service::RoutePopupState;
use crate::gui::services::view_mode_service::DisplayMode;
use eframe::egui::{Context, Window};

/// A view model that provides data for the `RoutePopup` component.
pub struct RoutePopupViewModel<'a> {
    /// The state of the popup, including the selected route and weather data.
    pub popup_state: &'a RoutePopupState,
    /// The current display mode, used to determine which actions are available.
    pub display_mode: &'a DisplayMode,
}

use crate::gui::services::route_popup_service::WeatherState;
use eframe::egui::Ui;

/// A UI component that displays the details of a selected route in a popup window.
pub struct RoutePopup;

impl RoutePopup {
    /// Renders the route details popup window.
    ///
    /// This method only renders the window if `is_alert_visible` is true and a
    /// route is selected. It provides options to the user, such as marking a
    /// route as flown.
    ///
    /// # Arguments
    ///
    /// * `vm` - The `RoutePopupViewModel` containing the necessary data.
    /// * `ctx` - The `egui::Context` for rendering the window.
    ///
    /// # Returns
    ///
    /// A `Vec<Event>` containing any events triggered by user interaction.
    pub fn render(vm: &RoutePopupViewModel, ctx: &Context) -> Vec<Event> {
        let mut events = Vec::new();

        if let Some(route) = vm.popup_state.selected_route.as_ref() {
            let mut is_open = true; // The window is open if we are here
            Window::new("Route Details")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .open(&mut is_open)
                .show(ctx, |ui| {
                    ui.heading(format!(
                        "{} to {}",
                        route.departure.ICAO, route.destination.ICAO
                    ));
                    ui.separator();
                    ui.label(format!("Distance: {} nm", route.route_length));
                    ui.label(format!(
                        "Aircraft: {} {}",
                        route.aircraft.manufacturer, route.aircraft.variant
                    ));
                    ui.separator();

                    // --- Weather Information ---
                    ui.heading("Live Weather (METAR)");
                    ui.separator();
                    Self::render_weather_details(ui, "Departure", &vm.popup_state.departure_weather);
                    ui.separator();
                    Self::render_weather_details(ui, "Destination", &vm.popup_state.destination_weather);
                    ui.separator();

                    ui.horizontal(|ui| {
                        let routes_from_not_flown =
                            matches!(vm.display_mode, DisplayMode::NotFlownRoutes);
                        if routes_from_not_flown && ui.button("✅ Mark as Flown").clicked() {
                            events.push(Event::MarkRouteAsFlown(route.as_ref().clone()));
                            events.push(Event::ClosePopup);
                        }
                        if ui.button("Close").clicked() {
                            events.push(Event::ClosePopup);
                        }
                    });
                });

            if !is_open {
                events.push(Event::ClosePopup);
            }
        }

        events
    }

    /// Renders the weather details for a single airport.
    fn render_weather_details(ui: &mut Ui, label: &str, weather_state: &WeatherState) {
        ui.label(format!("{}:", label));
        match weather_state {
            WeatherState::Idle => {
                ui.label("Weather data not requested.");
            }
            WeatherState::Loading => {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Loading weather...");
                });
            }
            WeatherState::Success(metar) => {
                ui.label(format!("Flight Rules: {}", metar.flight_rules));
                if let Some(wind) = &metar.wind {
                    ui.label(format!("Wind: {} kts", wind.speed_kts));
                }
                if let Some(vis) = &metar.visibility {
                    ui.label(format!("Visibility: {} mi", vis.miles));
                }
            }
            WeatherState::Error(err) => {
                ui.colored_label(egui::Color32::RED, format!("Error: {}", err));
            }
        }
    }
}
