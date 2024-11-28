use crate::{AircraftOperations, DatabaseConnections};

pub struct Gui {
    database_connections: Box<DatabaseConnections>,
}

impl Gui {
    pub fn new(
        _cc: &eframe::CreationContext,
        database_connections: Box<DatabaseConnections>,
    ) -> Self {
        Gui {
            database_connections,
        }
    }
}

impl eframe::App for Gui {
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
