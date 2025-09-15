use eframe::egui::{Context, Window};

/// The route popup component.
pub struct RoutePopup;

impl RoutePopup {
    /// Renders the route details popup.
    pub fn render(gui: &mut crate::gui::ui::Gui, ctx: &Context) {
        if !gui.services.popup.is_alert_visible() {
            return;
        }

        let route_clone = gui.services.popup.selected_route().cloned();

        if let Some(route) = route_clone {
            let mut is_open = gui.services.popup.is_alert_visible();
            Window::new("Route Details")
                .collapsible(false)
                .resizable(false)
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
                        if gui.services.popup.routes_from_not_flown()
                            && ui.button("✅ Mark as Flown").clicked()
                        {
                            if let Err(e) = gui.mark_route_as_flown(&route) {
                                log::error!("Failed to mark route as flown: {e}");
                            } else {
                                gui.services.popup.set_alert_visibility(false);
                            }
                        }
                        if ui.button("Close").clicked() {
                            gui.services.popup.set_alert_visibility(false);
                        }
                    });
                });
            if !is_open {
                gui.services.popup.set_alert_visibility(false);
            }
        }
    }
}
