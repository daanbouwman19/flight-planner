use crate::gui::data::ListItemRoute;
use crate::gui::events::{AppEvent, DataEvent, UiEvent};
use crate::gui::icons;
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

impl<'a> RoutePopupViewModel<'a> {
    /// Checks if the current display mode is a route mode.
    pub fn is_route_mode(&self) -> bool {
        self.display_mode.is_route_mode()
    }
}

/// A UI component that displays the details of a selected route in a popup window.
pub struct RoutePopup;

const SKYVECTOR_URL_FORMAT: &str = "https://skyvector.com/?fpl={} {}";
const GOOGLE_MAPS_URL_FORMAT: &str = "https://www.google.com/maps/search/?api=1&query={},{}";

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
                    ui.horizontal(|ui| {
                        crate::gui::components::common::render_copyable_heading(
                            ui,
                            &format!("{} to {}", route.departure.ICAO, route.destination.ICAO),
                            &format!("{} to {}", route.departure.ICAO, route.destination.ICAO),
                            "Click to copy route",
                        );
                        ui.add_space(8.0);
                        let skyvector_url = SKYVECTOR_URL_FORMAT
                            .replacen("{}", &route.departure.ICAO, 1)
                            .replacen("{}", &route.destination.ICAO, 1);
                        ui.hyperlink_to(
                            format!("View on SkyVector {}", icons::ICON_EXTERNAL_LINK),
                            skyvector_url,
                        )
                        .on_hover_text("Open route in SkyVector");
                    });

                    ui.separator();
                    ui.label(format!("Distance: {} nm", route.route_length));
                    ui.label(format!(
                        "Aircraft: {} {}",
                        route.aircraft.manufacturer, route.aircraft.variant
                    ));

                    Self::render_airport_elevation_with_map_link(ui, &route.departure, "Departure");
                    Self::render_airport_elevation_with_map_link(
                        ui,
                        &route.destination,
                        "Destination",
                    );
                    ui.separator();

                    ui.heading("Weather (METAR)");
                    if let Some(metar) = vm.departure_metar {
                        ui.label(format!("Departure ({}):", route.departure.ICAO));
                        Self::render_flight_rules_ui(ui, metar);
                        Self::render_metar_raw_ui(ui, metar);
                    } else if let Some(error) = vm.departure_weather_error {
                        ui.label(
                            egui::RichText::new(format!(
                                "{} Departure ({}): {}",
                                icons::ICON_WARNING,
                                route.departure.ICAO,
                                error
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
                                "{} Destination ({}): {}",
                                icons::ICON_WARNING,
                                route.destination.ICAO,
                                error
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
                        if vm.is_route_mode()
                            && ui
                                .button(format!("{} Mark as Flown", icons::ICON_CHECK_CIRCLE))
                                .on_hover_text(
                                    "Add this flight to your history and mark it as flown",
                                )
                                .clicked()
                        {
                            events.push(AppEvent::Data(DataEvent::MarkRouteAsFlown(route.clone())));
                            events.push(AppEvent::Ui(UiEvent::ClosePopup));
                        }
                        if ui
                            .button(format!("{} Close", icons::ICON_CLOSE))
                            .on_hover_text("Close this window (Esc)")
                            .clicked()
                            || ui.input(|i| i.key_pressed(egui::Key::Escape))
                        {
                            events.push(AppEvent::Ui(UiEvent::ClosePopup));
                        }
                    });
                });
            if !is_open {
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

    fn render_airport_elevation_with_map_link(
        ui: &mut egui::Ui,
        airport: &std::sync::Arc<crate::models::Airport>,
        prefix: &str,
    ) {
        ui.horizontal(|ui| {
            ui.label(format!("{} Elevation: {} ft", prefix, airport.Elevation));
            ui.add_space(4.0);
            let url = GOOGLE_MAPS_URL_FORMAT
                .replacen("{}", &airport.Latitude.to_string(), 1)
                .replacen("{}", &airport.Longtitude.to_string(), 1);
            ui.hyperlink_to(format!("{} Map", icons::ICON_MAP), url)
                .on_hover_text(format!(
                    "View {} airport on Google Maps",
                    prefix.to_lowercase()
                ));
        });
    }
}
