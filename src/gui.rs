use diesel::SqliteConnection;

use crate::get_unflown_aircraft_count;

pub struct GUI {
    aircraft_connection: SqliteConnection,
    airport_connection: SqliteConnection,
}

impl GUI {
    pub fn new(
        _cc: &eframe::CreationContext,
        aircraft_connection: SqliteConnection,
        airport_connection: SqliteConnection,
    ) -> Self {
        GUI {
            aircraft_connection,
            airport_connection,
        }
    }
}

impl eframe::App for GUI {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello, World!");

            ui.label(format!(
                "Unflown aircraft: {}",
                get_unflown_aircraft_count(&mut self.aircraft_connection).unwrap()
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
