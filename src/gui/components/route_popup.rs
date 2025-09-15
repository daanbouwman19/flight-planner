use crate::gui::data::ListItemRoute;
use crate::gui::events::Event;
use crate::gui::services::popup_service::DisplayMode;
use eframe::egui::{Context, Window};

/// ViewModel for the RoutePopup component.
pub struct RoutePopupViewModel<'a> {
    pub is_alert_visible: bool,
    pub selected_route: Option<&'a ListItemRoute>,
    pub display_mode: &'a DisplayMode,
}

/// The route popup component.
pub struct RoutePopup;

impl RoutePopup {
    /// Renders the route details popup.
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
