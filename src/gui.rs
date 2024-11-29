use egui_extras::{Column, TableBuilder};

use crate::{
    format_aircraft, format_airport, models::Airport, AircraftOperations, AirportOperations, DatabaseConnections
};

pub struct Gui<'a> {
    database_connections: &'a mut DatabaseConnections,
    results: Vec<String>,
}

impl<'a> Gui<'a> {
    pub fn new(
        _cc: &eframe::CreationContext,
        database_connections: &'a mut DatabaseConnections,
    ) -> Self {
        Gui {
            database_connections,
            results: Vec::new(),
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

            let width = ctx.screen_rect().width();
            let panel_width = width * 0.5;

            ui.with_layout(
                egui::Layout::left_to_right(egui::Align::Min).with_cross_justify(true),
                |ui| {
                    ui.vertical(|ui| {
                        ui.set_min_width(panel_width);

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

                    ui.vertical(|ui| {
                        ui.set_min_width(panel_width);

                        let columns: Vec<(&str, f32, Box<dyn Fn(&Airport) -> String>)> = vec![
                            ("ID", 100.0, Box::new(|airport: &Airport| airport.ID.to_string())),
                            ("Name", 200.0, Box::new(|airport: &Airport| airport.Name.clone())),
                            ("ICAO", 100.0, Box::new(|airport: &Airport| airport.ICAO.clone())),
                        ];

                        let mut table = TableBuilder::new(ui);
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
                            .body(|mut body| {
                                let airports = self.database_connections.get_airports().unwrap();
                                
                                for airport in airports[..100].iter() {
                                    body.row(30.0, |mut row| {
                                        for (_, _, get_data) in &columns {
                                            row.col(|ui| {
                                                ui.label(get_data(&airport));
                                            });
                                        }
                                    });
                                }
                            });
                    });
                },
            );
        });
    }
}
