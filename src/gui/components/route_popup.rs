use crate::gui::ui::Gui;
use egui::Context;

pub struct RoutePopup;

impl RoutePopup {
    /// Renders a popup dialog for route details.
    pub fn render(gui: &mut Gui, ctx: &Context) {
        if !gui.show_alert() {
            return;
        }

        // Clone the route to avoid borrowing issues
        let route_clone = gui.get_selected_route().cloned();

        egui::Window::new("Route Details")
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO) // Center the popup
            .fixed_size(egui::Vec2::new(400.0, 200.0)) // Fixed size for consistent appearance
            .show(ctx, |ui| {
                if let Some(route) = &route_clone {
                    ui.label(format!(
                        "Aircraft: {} {}",
                        route.aircraft.manufacturer, route.aircraft.variant
                    ));
                    ui.label(format!(
                        "From: {} ({})",
                        route.departure.Name, route.departure.ICAO
                    ));
                    ui.label(format!(
                        "To: {} ({})",
                        route.destination.Name, route.destination.ICAO
                    ));
                    ui.label(format!("Distance: {}", route.route_length));

                    ui.horizontal(|ui| {
                        // Only show "Mark as Flown" button for "not flown" routes
                        if gui.routes_from_not_flown() && ui.button("✅ Mark as Flown").clicked() {
                            if let Err(e) = gui.mark_route_as_flown(route) {
                                log::error!("Failed to mark route as flown: {e}");
                            } else {
                                gui.update_displayed_items();
                            }
                            gui.set_show_alert(false);
                        }

                        if ui.button("❌ Close").clicked() {
                            gui.set_show_alert(false);
                        }
                    });
                } else {
                    ui.label("No route selected");
                    if ui.button("Close").clicked() {
                        gui.set_show_alert(false);
                    }
                }
            });
    }
}
