use crate::gui::data::ListItemRoute;
use crate::gui::events::{AppEvent, DataEvent, UiEvent};
use crate::gui::services::popup_service::DisplayMode;
use crate::models::weather::{FlightRules, Metar};
use eframe::egui::{Context, Window};

/// A view model that provides data for the `RoutePopup` component.
pub struct RoutePopupViewModel<'a> {
    /// A flag indicating whether the popup should be visible.
    pub is_alert_visible: bool,
    /// The route to be displayed in the popup.
    pub selected_route: Option<&'a ListItemRoute>,
    /// The current display mode, used to determine which actions are available.
    pub display_mode: &'a DisplayMode,
    /// METAR data for the departure airport.
    pub departure_metar: Option<&'a Metar>,
    /// METAR data for the destination airport.
    pub destination_metar: Option<&'a Metar>,
    /// Error message for departure weather.
    pub departure_weather_error: Option<&'a str>,
    /// Error message for destination weather.
    pub destination_weather_error: Option<&'a str>,
}

/// A UI component that displays the details of a selected route in a popup window.
pub struct RoutePopup;

#[cfg(not(tarpaulin_include))]
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
    #[cfg(not(tarpaulin_include))]
    pub fn render(vm: &RoutePopupViewModel, ctx: &Context) -> Vec<AppEvent> {
        let mut events = Vec::new();

        if !vm.is_alert_visible {
            return events;
        }

        if let Some(route) = vm.selected_route {
            let mut is_open = vm.is_alert_visible;
            Window::new("Route Details")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .open(&mut is_open) // This makes the window closeable
                .show(ctx, |ui| {
                    crate::gui::components::common::render_copyable_heading(
                        ui,
                        &format!("{} to {}", route.departure.ICAO, route.destination.ICAO),
                        &format!("{} to {}", route.departure.ICAO, route.destination.ICAO),
                        "Click to copy route",
                    );
                    ui.separator();
                    ui.label(format!("Distance: {} nm", route.route_length));
                    ui.label(format!(
                        "Aircraft: {} {}",
                        route.aircraft.manufacturer, route.aircraft.variant
                    ));
                    ui.label(format!(
                        "Departure Elevation: {} ft",
                        route.departure.Elevation
                    ));
                    ui.label(format!(
                        "Destination Elevation: {} ft",
                        route.destination.Elevation
                    ));
                    ui.separator();

                    ui.heading("Weather (METAR)");
                    if let Some(metar) = vm.departure_metar {
                        ui.label(format!("Departure ({}):", route.departure.ICAO));
                        Self::render_flight_rules_ui(ui, metar);
                        Self::render_metar_raw_ui(ui, metar);
                    } else if let Some(error) = vm.departure_weather_error {
                        ui.label(
                            egui::RichText::new(format!(
                                "⚠️ Departure ({}): {}",
                                route.departure.ICAO, error
                            ))
                            .color(ui.visuals().error_fg_color),
                        );
                    } else {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label(format!("Fetching weather for {}...", route.departure.ICAO));
                        });
                    }
                    ui.add_space(4.0);
                    if let Some(metar) = vm.destination_metar {
                        ui.label(format!("Destination ({}):", route.destination.ICAO));
                        Self::render_flight_rules_ui(ui, metar);
                        Self::render_metar_raw_ui(ui, metar);
                    } else if let Some(error) = vm.destination_weather_error {
                        ui.label(
                            egui::RichText::new(format!(
                                "⚠️ Destination ({}): {}",
                                route.destination.ICAO, error
                            ))
                            .color(ui.visuals().error_fg_color),
                        );
                    } else {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.label(format!(
                                "Fetching weather for {}...",
                                route.destination.ICAO
                            ));
                        });
                    }

                    ui.separator();

                    ui.horizontal(|ui| {
                        let routes_from_not_flown =
                            matches!(vm.display_mode, DisplayMode::NotFlownRoutes);
                        if routes_from_not_flown
                            && ui
                                .button("✅ Mark as Flown")
                                .on_hover_text(
                                    "Add this flight to your history and mark it as flown",
                                )
                                .clicked()
                        {
                            events.push(AppEvent::Data(DataEvent::MarkRouteAsFlown(route.clone())));
                            events.push(AppEvent::Ui(UiEvent::ClosePopup));
                        }
                        if ui
                            .button("❌ Close")
                            .on_hover_text("Close this window (Esc)")
                            .clicked()
                        {
                            events.push(AppEvent::Ui(UiEvent::ClosePopup));
                        }
                    });
                });
            if !is_open || ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                events.push(AppEvent::Ui(UiEvent::ClosePopup));
            }
        }

        events
    }

    fn render_flight_rules_ui(ui: &mut egui::Ui, metar: &Metar) {
        let rules = FlightRules::from(metar.flight_rules.as_deref().unwrap_or(""));
        let color = crate::gui::styles::get_flight_rules_color(&rules, ui.visuals());
        let tooltip = rules.description();
        ui.horizontal(|ui| {
            ui.label("Rules:");
            ui.label(egui::RichText::new(rules.as_str()).color(color))
                .on_hover_text(tooltip);
        });
    }

    fn render_metar_raw_ui(ui: &mut egui::Ui, metar: &Metar) {
        let raw_metar = metar.raw.as_deref().unwrap_or("Raw METAR unavailable");
        crate::gui::components::common::render_copyable_label(
            ui,
            raw_metar,
            raw_metar,
            "Click to copy METAR",
            true,
        );
    }
}
