use egui_extras::{Column, TableBuilder};

use crate::{
    format_aircraft, format_airport, models::Airport, AircraftOperations, AirportOperations,
    DatabaseConnections,
};

pub struct Gui<'a> {
    database_connections: &'a mut DatabaseConnections,
    results: Vec<String>,
    airports: Vec<Airport>, // New field to store airports
}

impl<'a> Gui<'a> {
    pub fn new(
        _cc: &eframe::CreationContext,
        database_connections: &'a mut DatabaseConnections,
    ) -> Self {
        let airports = database_connections.get_airports().unwrap_or_default();

        Gui {
            database_connections,
            results: Vec::new(),
            airports,
        }
    }
}

impl eframe::App for Gui<'_> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Exit").clicked() {
                        std::process::exit(0);
                    }
                });
            });

            // Use a left-to-right layout to place the buttons and the table side by side
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                // Left side - Buttons with fixed width
                ui.vertical(|ui| {
                    if ui
                        .button("Select random aircraft")
                        .on_hover_text("Select a random aircraft from the database")
                        .clicked()
                    {
                        match self.database_connections.random_aircraft() {
                            Ok(aircraft) => {
                                self.results.push(format_aircraft(&aircraft));
                            }
                            Err(_) => {
                                self.results.push("No aircraft found".to_string());
                            }
                        }
                    }

                    if ui.button("Get random airport").clicked() {
                        match self.database_connections.get_random_airport() {
                            Ok(airport) => {
                                self.results.push(format_airport(&airport));
                            }
                            Err(_) => {
                                self.results.push("No airport found".to_string());
                            }
                        }
                    }
                });

                // Adding some spacing between buttons and table
                ui.add_space(50.0);

                // Right side - Table, snapped to the right edge with flexible resizing
                let available_width = ui.available_width();

                // Create a vertical container for the table with dynamic width adjustment
                ui.vertical(|ui| {
                    ui.set_min_width(available_width); // Set the width to take all available space

                    let columns: Vec<(&str, f32, Box<dyn Fn(&Airport) -> String>)> = vec![
                        (
                            "ID",
                            100.0,
                            Box::new(|airport: &Airport| airport.ID.to_string()),
                        ),
                        (
                            "Name",
                            200.0,
                            Box::new(|airport: &Airport| airport.Name.clone()),
                        ),
                        (
                            "ICAO",
                            100.0,
                            Box::new(|airport: &Airport| airport.ICAO.clone()),
                        ),
                    ];

                    let row_height = 30.0;

                    let mut table = TableBuilder::new(ui)
                        .striped(true)
                        .resizable(true)
                        .min_scrolled_height(0.0); // Ensure the table stretches fully vertically

                    for (_, width, _) in &columns {
                        table = table.column(Column::initial(*width).resizable(true));
                    }

                    table
                        .header(20.0, |mut header| {
                            for (name, _, _) in &columns {
                                header.col(|ui| {
                                    ui.label(*name);
                                });
                            }
                        })
                        .body(|body| {
                            body.rows(row_height, self.airports.len(), |mut row| {
                                if let Some(airport) = self.airports.get(row.index()) {
                                    for (_, _, get_data) in &columns {
                                        row.col(|ui| {
                                            ui.label(get_data(airport));
                                        });
                                    }
                                }
                            });
                        });
                });
            });
        });
    }
}
