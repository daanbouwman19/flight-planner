// OLD COMPONENT IMPLEMENTATION - DISABLED FOR CLEAN ARCHITECTURE
/*
impl Gui {
    /// Shows the modal popup for route selection.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The egui context.
    pub fn show_modal_popup(&mut self, ctx: &Context) {
        let modal = egui::Modal::new(Id::NULL);
        modal.show(ctx, |ui| {
            if let Some(route) = self.get_selected_route() {
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
                    if self.routes_from_not_flown() && ui.button("Mark as flown").clicked() {
                        self.handle_mark_flown_button(&route_clone);
                    }
                    if ui.button("Close").clicked() {
                        self.set_show_alert(false);
                    }
                });
            } else {
                // Fallback case - this shouldn't normally happen, but it's safer to handle it
                ui.label("No route selected");
                if ui.button("Close").clicked() {
                    self.set_show_alert(false);
                }
            }
        });
    }
}
*/
