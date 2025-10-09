use crate::gui::data::ListItemRoute;
use crate::gui::events::Event;
use crate::gui::services::popup_service::DisplayMode;
use eframe::egui::{Context, Window};

/// A view model that provides data for the `RoutePopup` component.
pub struct RoutePopupViewModel<'a> {
    /// A flag indicating whether the popup should be visible.
    pub is_alert_visible: bool,
    /// The route to be displayed in the popup.
    pub selected_route: Option<&'a ListItemRoute>,
    /// The current display mode, used to determine which actions are available.
    pub display_mode: &'a DisplayMode,
}

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

                    ui.heading("Weather");
                    if let Some(metar) = &route.departure_metar {
                        ui.label(format!("Departure ({}): {}", route.departure.ICAO, metar.raw));
                    } else {
                        ui.label(format!(
                            "Departure ({}): METAR not available",
                            route.departure.ICAO
                        ));
                    }
                    if let Some(metar) = &route.destination_metar {
                        ui.label(format!(
                            "Destination ({}): {}",
                            route.destination.ICAO, metar.raw
                        ));
                    } else {
                        ui.label(format!(
                            "Destination ({}): METAR not available",
                            route.destination.ICAO
                        ));
                    }
                    ui.separator();

                    ui.horizontal(|ui| {
                        let routes_from_not_flown =
                            matches!(vm.display_mode, DisplayMode::NotFlownRoutes);
                        if routes_from_not_flown && ui.button("✅ Mark as Flown").clicked() {
                            events.push(Event::MarkRouteAsFlown(route.clone()));
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
}
