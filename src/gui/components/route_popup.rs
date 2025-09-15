use crate::gui::events::Event;
use crate::gui::state::ViewState;
use eframe::egui::{Context, Window};

/// The route popup component.
pub struct RoutePopup;

impl RoutePopup {
    /// Renders the route details popup.
    pub fn render(view_state: &mut ViewState, ctx: &Context, event_handler: &mut dyn FnMut(Event)) {
        if !view_state.show_route_popup {
            return;
        }

        if let Some(route) = &view_state.selected_route_for_popup {
            let mut is_open = view_state.show_route_popup;
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
                        if ui.button("✅ Mark as Flown").clicked() {
                            // This now sends an event instead of directly calling a method
                            event_handler(Event::SetShowPopup(false));
                            // You would also add a new event here, like:
                            // event_handler(Event::MarkRouteFlown(route.clone()));
                        }
                        if ui.button("Close").clicked() {
                            event_handler(Event::SetShowPopup(false));
                        }
                    });
                });
            if !is_open {
                event_handler(Event::SetShowPopup(false));
            }
        }
    }
}
