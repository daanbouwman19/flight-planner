use crate::{AircraftOperations, DatabaseConnections};

pub struct Gui<'a> {
    database_connections: &'a mut DatabaseConnections,
}

impl<'a> Gui<'a> {
    pub fn new(
        cc: &eframe::CreationContext,
        database_connections: &'a mut DatabaseConnections,
    ) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals {
            window_fill: egui::Color32::WHITE,
            panel_fill: egui::Color32::WHITE,
            dark_mode: false,
            ..Default::default()
        });
        Gui {
            database_connections,
        }
    }
}

impl eframe::App for Gui<'_> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello, World!");

            ui.label(format!(
                "Unflown aircraft: {}",
                self.database_connections
                    .get_unflown_aircraft_count()
                    .unwrap()
            ));

            if ui.button("Quit").clicked() {
                let ctx_clone = ctx.clone();
                std::thread::spawn(move || {
                    ctx_clone.send_viewport_cmd(egui::ViewportCommand::Close);
                });
            }
        });
    }
}
