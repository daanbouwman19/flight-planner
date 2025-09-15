use crate::gui::data::ListItemRoute;
use crate::gui::events::Event;
use crate::gui::state::{ApplicationState, ViewState};
use eframe::egui::{self, Context, Window};

/// The route popup component.
pub struct RoutePopup;

impl RoutePopup {
    /// Renders the route details popup.
    pub fn render<F>(
        view_state: &mut ViewState,
        _app_state: &ApplicationState,
        ctx: &Context,
        mut event_handler: F,
    ) where
        F: FnMut(Event),
    {
        if let Some(route) = &view_state.selected_route_for_popup {
            Window::new("Route Details")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.heading(format!(
                        "{} to {}",
                        route.departure.ICAO, route.destination.ICAO
                    ));
                    ui.separator();
                    ui.label(format!("Distance: {} nm", route.route_length));
                    ui.label(format!("Aircraft: {}", route.aircraft.variant));
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Fly this route").clicked() {
                            event_handler(Event::SetShowPopup(false));
                        }
                        if ui.button("Close").clicked() {
                            event_handler(Event::SetShowPopup(false));
                        }
                    });
                });
        }
    }
}
