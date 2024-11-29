use crate::{
    haversine_distance_nm,
    models::{Aircraft, Airport, Runway},
    AircraftOperations, AirportOperations, DatabaseConnections,
};
use eframe::egui::{self, TextEdit};
use egui_extras::{Column, TableBuilder};

enum TableItem {
    Airport(Airport),
    Aircraft(Aircraft),
    Route(Route),
}

struct Route {
    departure: Airport,
    destination: Airport,
    aircraft: Aircraft,
    departure_runway: Vec<Runway>,
    destination_runway: Vec<Runway>,
}

impl TableItem {
    fn get_columns(&self) -> Vec<&'static str> {
        match self {
            TableItem::Airport(_) => vec!["ID", "Name", "ICAO"],
            TableItem::Aircraft(_) => vec!["ID", "Model", "Registration", "Flown"],
            TableItem::Route(_) => vec![
                "Departure",
                "ICAO",
                "Runway length",
                "Destination",
                "ICAO",
                "Runway length",
                "Manufacturer",
                "Aircraft",
                "Distance",
            ],
        }
    }

    fn get_data(&self) -> Vec<String> {
        match self {
            TableItem::Airport(airport) => vec![
                airport.ID.to_string(),
                airport.Name.clone(),
                airport.ICAO.clone(),
            ],
            TableItem::Aircraft(aircraft) => vec![
                aircraft.id.to_string(),
                aircraft.variant.clone(),
                aircraft.manufacturer.clone(),
                aircraft.flown.to_string(),
            ],
            TableItem::Route(route) => vec![
                route.departure.Name.clone(),
                route.departure.ICAO.clone(),
                route
                    .departure_runway
                    .iter()
                    .max_by(|a, b| a.Length.cmp(&b.Length))
                    .unwrap()
                    .Length
                    .to_string(),
                route.destination.Name.clone(),
                route.destination.ICAO.clone(),
                route
                    .destination_runway
                    .iter()
                    .max_by(|a, b| a.Length.cmp(&b.Length))
                    .unwrap()
                    .Length
                    .to_string(),
                route.aircraft.manufacturer.clone(),
                route.aircraft.variant.clone(),
                haversine_distance_nm(&route.departure, &route.destination).to_string(),
            ],
        }
    }

    fn matches_query(&self, query: &str) -> bool {
        let query = query.to_lowercase();
        match self {
            TableItem::Airport(airport) => {
                airport.Name.to_lowercase().contains(&query)
                    || airport.ICAO.to_lowercase().contains(&query)
                    || airport.ID.to_string().contains(&query)
            }
            TableItem::Aircraft(aircraft) => {
                aircraft.variant.to_lowercase().contains(&query)
                    || aircraft.manufacturer.to_lowercase().contains(&query)
                    || aircraft.id.to_string().contains(&query)
            }
            TableItem::Route(_) => false,
        }
    }
}

pub struct Gui<'a> {
    database_connections: &'a mut DatabaseConnections,
    displayed_items: Vec<TableItem>,
    search_query: String,
}

impl<'a> Gui<'a> {
    pub fn new(
        _cc: &eframe::CreationContext,
        database_connections: &'a mut DatabaseConnections,
    ) -> Self {
        Gui {
            database_connections,
            displayed_items: Vec::new(),
            search_query: String::new(),
        }
    }

    fn update_menu(&self, ui: &mut egui::Ui) {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Exit").clicked() {
                    std::process::exit(0);
                }
            });
        });
    }

    fn generate_random_route(&mut self) -> Result<Route, String> {
        let aircraft = self
            .database_connections
            .random_aircraft()
            .map_err(|err| format!("Failed to get random aircraft: {}", err))?;

        let departure = self
            .database_connections
            .get_random_airport_for_aircraft(&aircraft)
            .map_err(|err| format!("Failed to get random airport for aircraft: {}", err))?;

        let destination = self
            .database_connections
            .get_destination_airport(&aircraft, &departure)
            .map_err(|err| format!("Failed to get destination airport: {}", err))?;

        let departure_runway = self
            .database_connections
            .get_runways_for_airport(&departure)
            .map_err(|err| format!("Failed to get runways for departure airport: {}", err))?;

        let destination_runway = self
            .database_connections
            .get_runways_for_airport(&destination)
            .map_err(|err| format!("Failed to get runways for destination airport: {}", err))?;

        Ok(Route {
            departure,
            destination,
            aircraft,
            departure_runway,
            destination_runway,
        })
    }

    fn update_buttons(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            if ui
                .button("Select random aircraft")
                .on_hover_text("Select a random aircraft from the database")
                .clicked()
            {
                if let Ok(aircraft) = self.database_connections.random_aircraft() {
                    self.displayed_items = vec![TableItem::Aircraft(aircraft)];
                    self.search_query.clear();
                }
            }

            if ui.button("Get random airport").clicked() {
                if let Ok(airport) = self.database_connections.get_random_airport() {
                    self.displayed_items = vec![TableItem::Airport(airport)];
                    self.search_query.clear();
                }
            }

            if ui.button("List all airports").clicked() {
                if let Ok(airports) = self.database_connections.get_airports() {
                    self.displayed_items = airports.into_iter().map(TableItem::Airport).collect();
                    self.search_query.clear();
                }
            }

            if ui.button("Random route").clicked() {
                self.displayed_items.clear();

                for _ in 0..10 {
                    match self.generate_random_route() {
                        Ok(route) => {
                            self.displayed_items.push(TableItem::Route(route));
                            self.search_query.clear();
                        }
                        Err(err) => {
                            log::error!("{}", err);
                            break;
                        }
                    }
                }
            }
        });
    }

    fn update_search_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.add(TextEdit::singleline(&mut self.search_query).hint_text("Type to search..."));
        });
    }

    fn filter_items(&self) -> Vec<&TableItem> {
        if self.search_query.is_empty() {
            self.displayed_items.iter().collect()
        } else {
            self.displayed_items
                .iter()
                .filter(|item| item.matches_query(&self.search_query))
                .collect()
        }
    }

    fn update_table(&self, ui: &mut egui::Ui, filtered_items: &[&TableItem]) {
        let row_height = 30.0;

        if let Some(first_item) = filtered_items.first() {
            let columns = first_item.get_columns();

            let mut table = TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .min_scrolled_height(0.0);

            for _ in &columns {
                table = table.column(Column::auto().resizable(true));
            }

            table
                .header(20.0, |mut header| {
                    for name in &columns {
                        header.col(|ui| {
                            ui.label(*name);
                        });
                    }
                })
                .body(|body| {
                    body.rows(row_height, filtered_items.len(), |mut row| {
                        if let Some(item) = filtered_items.get(row.index()) {
                            for data in item.get_data() {
                                row.col(|ui| {
                                    ui.label(data);
                                });
                            }
                        }
                    });
                });
        }
    }
}

impl eframe::App for Gui<'_> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.update_menu(ui);

            ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                self.update_buttons(ui);
                ui.add_space(50.0);

                ui.vertical(|ui| {
                    self.update_search_bar(ui);
                    let filtered_items = self.filter_items();
                    self.update_table(ui, &filtered_items);
                });
            });
        });
    }
}
