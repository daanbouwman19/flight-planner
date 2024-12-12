use crate::traits::*;
use crate::{
    get_destination_airport_with_suitable_runway_fast, haversine_distance_nm,
    models::{Aircraft, Airport, Runway},
    DatabasePool,
};
use eframe::egui::{self, TextEdit};
use egui_extras::{Column, TableBuilder};
use rand::prelude::SliceRandom;
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    RwLock,
};
use std::time::Instant;

#[derive(Clone)]
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

pub struct Gui {
    displayed_items: Vec<TableItem>,
    search_query: String,
    all_aircraft: Vec<Aircraft>,
    all_airports: Vec<Airport>,
    all_runways: HashMap<i32, Vec<Runway>>,
    popup_state: PopupState,
}

#[derive(Default)]
struct PopupState {
    show_alert: bool,
    selected_route: Option<Route>,
}

impl Gui {
    pub fn new(_cc: &eframe::CreationContext, database_pool: &mut DatabasePool) -> Self {
        let all_aircraft = database_pool
            .get_all_aircraft()
            .expect("Failed to load aircraft");
        let all_airports = database_pool
            .get_airports()
            .expect("Failed to load airports");
        let runway_data = database_pool.get_runways().expect("Failed to load runways");
        let all_runways: HashMap<i32, Vec<Runway>> =
            runway_data
                .into_iter()
                .fold(HashMap::new(), |mut acc, runway| {
                    acc.entry(runway.AirportID).or_default().push(runway);
                    acc
                });

        Gui {
            displayed_items: Vec::new(),
            search_query: String::new(),
            all_aircraft,
            all_airports,
            all_runways,
            popup_state: PopupState::default(),
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
        const GRID_SIZE: f64 = 1.0;
        const M_TO_FT: f64 = 3.28084;

        let mut rng = rand::thread_rng();
        let attempt_counter = AtomicUsize::new(0);
        let route_counter = AtomicUsize::new(0);
        let shared_routes = RwLock::new(Vec::new());

        let airports_by_grid: HashMap<(i32, i32), Vec<&Airport>> =
            self.all_airports
                .iter()
                .fold(HashMap::new(), |mut acc, airport| {
                    let lat_bin = (airport.Latitude / GRID_SIZE).floor() as i32;
                    let lon_bin = (airport.Longtitude / GRID_SIZE).floor() as i32;
                    acc.entry((lat_bin, lon_bin)).or_default().push(airport);
                    acc
                });

        let mut shuffled_aircraft = self.all_aircraft.clone();
        let mut shuffled_airports = self.all_airports.clone();

        let start_time = Instant::now();

        while route_counter.load(Ordering::Relaxed) < AMOUNT
            && attempt_counter.load(Ordering::Relaxed) < 10
        {
            attempt_counter.fetch_add(1, Ordering::Relaxed);
            shuffled_airports.shuffle(&mut rng);
            shuffled_aircraft.shuffle(&mut rng);

            shuffled_airports.par_iter().for_each(|departure| {
                for aircraft in &shuffled_aircraft {
                    if route_counter.load(Ordering::Relaxed) >= AMOUNT {
                        break;
                    }

                    if let Some(max_runway_length) = self
                        .all_runways
                        .get(&departure.ID)
                        .and_then(|runways| runways.iter().map(|r| r.Length).max())
                    {
                        if let Some(distance) = aircraft.takeoff_distance {
                            if (distance as f64 * M_TO_FT) > (max_runway_length as f64) {
                                continue;
                            }
                        }
                    }

                    if let Ok(destination) = get_destination_airport_with_suitable_runway_fast(
                        aircraft,
                        departure,
                        &airports_by_grid,
                        &self.all_runways,
                        GRID_SIZE,
                    ) {
                        let route = Route {
                            departure: departure.clone(),
                            destination: destination.clone(),
                            aircraft: aircraft.clone(),
                            departure_runway: self
                                .all_runways
                                .get(&departure.ID)
                                .cloned()
                                .unwrap_or_default(),
                            destination_runway: self
                                .all_runways
                                .get(&destination.ID)
                                .cloned()
                                .unwrap_or_default(),
                        };

                        let mut routes_guard = shared_routes.write().unwrap();
                        if route_counter.load(Ordering::Relaxed) < AMOUNT {
                            routes_guard.push(route);
                            route_counter.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                }
            });
        }

        let duration = start_time.elapsed();
        let final_routes = shared_routes.read().unwrap().clone();
        println!(
            "Generated {} routes in {:.2?}",
            final_routes.len(),
            duration
        );
        Ok(final_routes)
    }

    fn update_buttons(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            if ui
                .button("Select random aircraft")
                .on_hover_text("Select a random aircraft from the database")
                .clicked()
            {
                if let Some(aircraft) = self.all_aircraft.choose(&mut rand::thread_rng()) {
                    self.displayed_items = vec![TableItem::Aircraft(aircraft.clone())];
                    self.search_query.clear();
                }
            }

            if ui.button("Get random airport").clicked() {
                if let Some(airport) = self.all_airports.choose(&mut rand::thread_rng()) {
                    self.displayed_items = vec![TableItem::Airport(airport.clone())];
                    self.search_query.clear();
                }
            }

            if ui.button("List all airports").clicked() {
                self.displayed_items = self
                    .all_airports
                    .iter()
                    .map(|airport| TableItem::Airport(airport.clone()))
                    .collect();
                self.search_query.clear();
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

    fn update_table(
        &self,
        ui: &mut egui::Ui,
        filtered_items: &[&TableItem],
    ) -> (bool, Option<Route>) {
        let row_height = 30.0;
        let mut show_alert_flag = false;
        let mut route_to_select: Option<Route> = None;

        if let Some(first_item) = filtered_items.first() {
            let mut columns = first_item.get_columns();
            if let TableItem::Route(_) = first_item {
                columns.push("Select");
            }

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
                        let item = filtered_items[row.index()];
                        let data = item.get_data();
                        for d in data {
                            row.col(|ui| {
                                ui.label(d);
                            });
                        }

                        if let TableItem::Route(route) = item {
                            row.col(|ui| {
                                if ui.button("Select").clicked() {
                                    show_alert_flag = true;
                                    route_to_select = Some((**route).clone());
                                }
                            });
                        }
                    });
                });
        }

        (show_alert_flag, route_to_select)
    }

    fn show_modal_popup(&mut self, ctx: &egui::Context) {
        // Draw a semi-transparent overlay on top of everything
        egui::Area::new(egui::Id::new("modal_overlay"))
            .interactable(false)
            .fixed_pos(egui::pos2(0.0, 0.0))
            .show(ctx, |ui| {
                let size = ctx.input(|i| i.screen_rect().size());
                ui.painter().rect_filled(
                    egui::Rect::from_min_size(egui::Pos2::ZERO, size),
                    0.0,
                    egui::Color32::from_black_alpha(128),
                );
            });

        egui::Window::new("Alert")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                if let Some(route) = &self.popup_state.selected_route {
                    ui.label(format!(
                        "Departure: {} ({})",
                        route.departure.Name, route.departure.ICAO
                    ));
                    ui.label(format!(
                        "Destination: {} ({})",
                        route.destination.Name, route.destination.ICAO
                    ));
                    ui.label(format!(
                        "Distance: {:.2} NM",
                        haversine_distance_nm(&route.departure, &route.destination)
                    ));
                    ui.label(format!(
                        "Aircraft: {} {}",
                        route.aircraft.manufacturer, route.aircraft.variant
                    ));
                } else {
                    ui.label("No route selected.");
                }

                ui.separator();
                if ui.button("OK").clicked() {
                    self.popup_state.show_alert = false;
                    self.popup_state.selected_route = None;
                }
            });
    }
}

impl eframe::App for Gui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.update_menu(ui);

            ui.add_enabled_ui(!self.popup_state.show_alert, |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                    self.update_buttons(ui);
                    ui.add_space(50.0);

                    ui.vertical(|ui| {
                        self.update_search_bar(ui);
                        let filtered_items = self.filter_items();
                        let (show_alert_flag, route_to_select) =
                            self.update_table(ui, &filtered_items);
                        if show_alert_flag {
                            self.popup_state.show_alert = true;
                            self.popup_state.selected_route = route_to_select;
                        }
                    });
                });
            });
        });

        if self.popup_state.show_alert {
            self.show_modal_popup(ctx);
        }
    }
}
