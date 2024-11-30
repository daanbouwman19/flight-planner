use rayon::prelude::*;
use std::collections::HashMap;

use crate::{
    get_destination_airport_with_suitable_runway_fast, haversine_distance_nm,
    models::{Aircraft, Airport, Runway},
    AircraftOperations, AirportOperations, DatabasePool,
};
use eframe::egui::{self, TextEdit};
use egui_extras::{Column, TableBuilder};
use rand::seq::SliceRandom;

enum TableItem {
    Airport(Airport),
    Aircraft(Aircraft),
    Route(Box<Route>),
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

pub struct Gui {
    database_pool: DatabasePool,
    displayed_items: Vec<TableItem>,
    search_query: String,
}

impl Gui {
    pub fn new(_cc: &eframe::CreationContext, database_pool: DatabasePool) -> Self {
        Gui {
            database_pool,
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

    fn generate_random_routes(&mut self) -> Result<Vec<Route>, String> {
        const AMOUNT: usize = 1000;
        const INITIAL_SAMPLE_SIZE: usize = 3000;
        const GRID_SIZE: f64 = 1.0;
        const MAX_RETRIES: usize = 10;
        // Use pre-allocated vectors to minimize dynamic resizing.
        let mut routes = Vec::with_capacity(AMOUNT);

        // Retrieve all the necessary data once to avoid redundant database queries.
        let aircrafts = self
            .database_pool
            .get_all_aircraft()
            .map_err(|err| format!("Failed to get all aircraft: {}", err))?;

        let airports = self
            .database_pool
            .get_airports()
            .map_err(|err| format!("Failed to get all airports: {}", err))?;

        let runways = self
            .database_pool
            .get_runways()
            .map_err(|err| format!("Failed to get all runways: {}", err))?;

        // Create hashmaps for quick lookup.
        let runways_by_airport: HashMap<_, Vec<_>> =
            runways
                .iter()
                .cloned()
                .fold(HashMap::new(), |mut acc, runway| {
                    acc.entry(runway.AirportID)
                        .or_insert_with(Vec::new)
                        .push(runway);
                    acc
                });

        let mut airports_by_grid: HashMap<(i32, i32), Vec<&Airport>> = HashMap::new();

        for airport in &airports {
            let lat_bin = (airport.Latitude / GRID_SIZE).floor() as i32;
            let lon_bin = (airport.Longtitude / GRID_SIZE).floor() as i32;
            airports_by_grid
                .entry((lat_bin, lon_bin))
                .or_insert_with(Vec::new)
                .push(airport);
        }

        let mut rng = rand::thread_rng();
        let mut result_aircrafts: Vec<_> = aircrafts
            .choose_multiple(&mut rng, INITIAL_SAMPLE_SIZE)
            .cloned()
            .collect();
        let mut result_departures: Vec<_> = airports
            .choose_multiple(&mut rng, INITIAL_SAMPLE_SIZE)
            .cloned()
            .collect();

        let mut attempts = 0;

        while routes.len() < AMOUNT && attempts < MAX_RETRIES {
            // Shuffle aircrafts and departures for variety in each retry.
            result_aircrafts.shuffle(&mut rng);
            result_departures.shuffle(&mut rng);

            // Iterate through aircrafts and departures together to generate routes in parallel.
            let new_routes: Vec<_> = result_aircrafts
                .par_iter()
                .zip(result_departures.par_iter())
                .filter_map(|(aircraft, departure)| {
                    if let Ok(destination) = get_destination_airport_with_suitable_runway_fast(
                        aircraft,
                        departure,
                        &airports_by_grid,
                        &runways_by_airport,
                        GRID_SIZE,
                    ) {
                        // Lookup departure and destination runways using precomputed hashmaps.
                        let departure_runway = runways_by_airport
                            .get(&departure.ID)
                            .cloned()
                            .unwrap_or_default();
                        let destination_runway = runways_by_airport
                            .get(&destination.ID)
                            .cloned()
                            .unwrap_or_default();

                        Some(Route {
                            departure: departure.clone(),
                            destination: destination.clone(),
                            aircraft: aircraft.clone(),
                            departure_runway,
                            destination_runway,
                        })
                    } else {
                        None
                    }
                })
                .collect();

            routes.extend(new_routes);
            attempts += 1;

            // Dynamically increase grid size if routes are not enough
            if routes.len() < AMOUNT {
                result_aircrafts = aircrafts
                    .choose_multiple(&mut rng, INITIAL_SAMPLE_SIZE)
                    .cloned()
                    .collect();
                result_departures = airports
                    .choose_multiple(&mut rng, INITIAL_SAMPLE_SIZE)
                    .cloned()
                    .collect();
            }
        }

        routes.truncate(AMOUNT);

        println!("Generated {} routes in {} attempts", routes.len(), attempts);
        Ok(routes)
    }

    fn update_buttons(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            if ui
                .button("Select random aircraft")
                .on_hover_text("Select a random aircraft from the database")
                .clicked()
            {
                if let Ok(aircraft) = self.database_pool.random_aircraft() {
                    self.displayed_items = vec![TableItem::Aircraft(aircraft)];
                    self.search_query.clear();
                }
            }

            if ui.button("Get random airport").clicked() {
                if let Ok(airport) = self.database_pool.get_random_airport() {
                    self.displayed_items = vec![TableItem::Airport(airport)];
                    self.search_query.clear();
                }
            }

            if ui.button("List all airports").clicked() {
                if let Ok(airports) = self.database_pool.get_airports() {
                    self.displayed_items = airports.into_iter().map(TableItem::Airport).collect();
                    self.search_query.clear();
                }
            }

            if ui.button("Random route").clicked() {
                self.displayed_items.clear();

                if let Ok(routes) = self.generate_random_routes() {
                    self.displayed_items = routes
                        .into_iter()
                        .map(|route| TableItem::Route(Box::new(route)))
                        .collect();
                    self.search_query.clear();
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

impl eframe::App for Gui {
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
