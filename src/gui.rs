use crate::{
    get_destination_airport_with_suitable_runway_fast, haversine_distance_nm,
    models::{Aircraft, Airport, Runway},
    AircraftOperations, AirportOperations, DatabasePool,
};
use eframe::egui::{self, TextEdit};
use egui_extras::{Column, TableBuilder};
use std::collections::HashMap;

enum TableItem {
    Airport(Airport),
    Aircraft(Aircraft),
    Route(Box<Route>),
}

#[derive(Clone)]
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
    database_pool: &'a mut DatabasePool,
    displayed_items: Vec<TableItem>,
    search_query: String,
}

impl<'a> Gui<'a> {
    pub fn new(_cc: &eframe::CreationContext, database_pool: &'a mut DatabasePool) -> Self {
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
        use rayon::prelude::*; // Import Rayon for parallel iteration
        use std::sync::{
            atomic::{AtomicUsize, Ordering}, // Atomic counters for thread-safe updates
            Arc, RwLock, // Shared mutable state management
        };
        use std::time::Instant; // Measure execution time
        use rand::seq::SliceRandom; // Randomize collections

        const AMOUNT: usize = 100; // Target number of routes to generate
        const GRID_SIZE: f64 = 1.0; // Grid size for organizing airports
        const MAX_RETRIES: usize = 10; // Maximum number of attempts to generate routes

        // Retrieve aircraft data from the database
        let aircraft_data = self
            .database_pool
            .get_all_aircraft()
            .map_err(|err| format!("Failed to get all aircraft: {}", err))?;

        // Retrieve airport data from the database
        let airport_data = self
            .database_pool
            .get_airports()
            .map_err(|err| format!("Failed to get all airports: {}", err))?;

        // Retrieve runway data from the database
        let runway_data = self
            .database_pool
            .get_runways()
            .map_err(|err| format!("Failed to get all runways: {}", err))?;

        // Group runways by their associated airport ID for quick lookup
        let runways_by_airport: HashMap<_, Vec<_>> =
            runway_data
                .into_iter()
                .fold(HashMap::new(), |mut acc, runway| {
                    acc.entry(runway.AirportID).or_default().push(runway);
                    acc
                });

        // Organize airports into grid cells for efficient spatial lookup
        let mut airports_by_grid: HashMap<(i32, i32), Vec<&Airport>> = HashMap::new();
        for airport in &airport_data {
            let lat_bin = (airport.Latitude / GRID_SIZE).floor() as i32;
            let lon_bin = (airport.Longtitude / GRID_SIZE).floor() as i32;
            airports_by_grid
                .entry((lat_bin, lon_bin))
                .or_default()
                .push(airport);
        }

        let mut rng = rand::thread_rng(); // Random number generator
        let attempt_counter = AtomicUsize::new(0); // Track total attempts made
        let route_counter = Arc::new(AtomicUsize::new(0)); // Shared route counter
        let shared_routes = Arc::new(RwLock::new(Vec::new())); // Thread-safe vector for storing routes

        let mut shuffled_aircraft = aircraft_data.clone(); // Aircraft list shuffled on each retry
        let mut shuffled_airports = airport_data.clone(); // Airport list shuffled on each retry

        const M_TO_FT: f64 = 3.28084; // Conversion factor: meters to feet

        let start_time = Instant::now(); // Start the timer for performance measurement

        // Retry until enough routes are generated or maximum retries are exceeded
        while route_counter.load(Ordering::Relaxed) < AMOUNT
            && attempt_counter.load(Ordering::Relaxed) < MAX_RETRIES
        {
            attempt_counter.fetch_add(1, Ordering::Relaxed); // Increment the attempt counter

            // Shuffle aircraft and airports for each attempt
            shuffled_airports.shuffle(&mut rng);
            shuffled_aircraft.shuffle(&mut rng);

            // Parallel iteration over shuffled airports
            shuffled_airports.par_iter().for_each(|departure| {
                for aircraft in shuffled_aircraft.iter() {
                    // Stop if the target number of routes is reached
                    if route_counter.load(Ordering::Relaxed) >= AMOUNT {
                        break;
                    }

                    // Check runway length at the departure airport
                    if let Some(runways) = runways_by_airport.get(&departure.ID) {
                        let max_runway_length = runways.iter().map(|r| r.Length).max();

                        if let Some(distance) = aircraft.takeoff_distance {
                            if let Some(max_runway_length) = max_runway_length {
                                // Skip if aircraft takeoff distance exceeds runway length
                                if distance as f64 * M_TO_FT > max_runway_length as f64 {
                                    continue;
                                }
                            }
                        }
                    }

                    // Attempt to find a destination airport with a suitable runway
                    if let Ok(destination) = get_destination_airport_with_suitable_runway_fast(
                        aircraft,
                        departure,
                        &airports_by_grid,
                        &runways_by_airport,
                        GRID_SIZE,
                    ) {
                        // Create a new route object
                        let route = Route {
                            departure: departure.clone(),
                            destination: destination.clone(),
                            aircraft: aircraft.clone(),
                            departure_runway: runways_by_airport
                                .get(&departure.ID)
                                .map(|runways| runways.clone())
                                .unwrap_or_default(),
                            destination_runway: runways_by_airport
                                .get(&destination.ID)
                                .map(|runways| runways.clone())
                                .unwrap_or_default(),
                        };

                        // Write the route to the shared vector
                        let mut routes_guard = shared_routes.write().unwrap();
                        if route_counter.load(Ordering::Relaxed) < AMOUNT {
                            routes_guard.push(route);
                            route_counter.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                }
            });
        }

        let duration = start_time.elapsed(); // Stop the timer

        let final_routes = shared_routes.read().unwrap().clone(); // Finalize the generated routes
        println!(
            "Generated {} routes in {} attempts in {:.2?}",
            final_routes.len(),
            attempt_counter.load(Ordering::Relaxed),
            duration
        );
        Ok(final_routes) // Return the generated routes
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
