#[cfg(feature = "gui")]
use std::path::{Path, PathBuf};

/// Simple GUI to show the airport database warning
#[cfg(feature = "gui")]
pub struct AirportDatabaseWarning {
    app_data_dir: PathBuf,
}

#[cfg(feature = "gui")]
impl AirportDatabaseWarning {
    pub fn new(app_data_dir: &Path) -> Self {
        Self {
            app_data_dir: app_data_dir.to_path_buf(),
        }
    }
}

#[cfg(feature = "gui")]
impl eframe::App for AirportDatabaseWarning {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);

                // Title
                ui.heading("❌ Missing Airports Database");
                ui.add_space(20.0);

                // Error message
                ui.label(
                    "The Flight Planner requires an airports database file (airports.db3) to function.",
                );
                ui.label(
                    "This file is not included with the application and must be provided by the user.",
                );
                ui.add_space(20.0);

                // Application data directory
                ui.label("📁 Application data directory:");
                ui.code(format!("{}", self.app_data_dir.display()));
                ui.add_space(20.0);

                // Instructions
                ui.label("📋 To fix this issue:");
                ui.label("1. Obtain an airports database file (airports.db3)");
                ui.label(format!(
                    "2. Copy it to: {}",
                    self.app_data_dir.display()
                ));
                ui.label("3. Restart the application");
                ui.add_space(20.0);

                ui.label(
                    "💡 Alternative: Run the application from the directory containing airports.db3",
                );
                ui.add_space(20.0);

                // Close button
                if ui.button("Close Application").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
        });
    }
}
