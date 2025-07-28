use egui::{Context, Id};

use crate::gui::ui::Gui;

impl Gui<'_> {
    /// Shows the modal popup for route selection.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The egui context.
    pub fn show_modal_popup(&mut self, ctx: &Context) {
        let modal = egui::Modal::new(Id::NULL);
        modal.show(ctx, |ui| {
            let route = self.popup_state.selected_route.as_ref().unwrap();
            let route_clone = route.clone();

            ui.label(format!(
                "Departure: {} ({})",
                route.departure.Name, route.departure.ICAO
            ));
            ui.label(format!(
                "Destination: {} ({})",
                route.destination.Name, route.destination.ICAO
            ));
            ui.label(format!("Distance: {} nm", route.route_length));
            ui.label(format!(
                "Aircraft: {} {}",
                route.aircraft.manufacturer, route.aircraft.variant
            ));

            ui.separator();
            ui.horizontal(|ui| {
                if self.popup_state.routes_from_not_flown && ui.button("Mark as flown").clicked() {
                    self.handle_mark_flown_button(&route_clone);
                }
                if ui.button("Close").clicked() {
                    self.popup_state.show_alert = false;
                }
            });
        });
    }
}
