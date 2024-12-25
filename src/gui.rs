use crate::models::History;
use crate::traits::*;
use crate::util::calculate_haversine_distance_nm;
use crate::{
    get_destination_airport_with_suitable_runway_fast,
    models::{Aircraft, Airport, Runway},
    DatabasePool,
};
use eframe::egui::{self, TextEdit};
use egui::Id;
use egui_extras::{Column, TableBuilder};
use rand::prelude::SliceRandom;
use rayon::prelude::*;
use rstar::{RTree, RTreeObject, AABB};
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

const GENERATE_AMOUNT: usize = 50;
const M_TO_FT: f64 = 3.28084;

/// An enum representing the items that can be displayed in the table.
enum TableItem {
    /// Represents an airport item.
    Airport(Arc<Airport>),
    /// Represents an aircraft item.
    Aircraft(Arc<Aircraft>),
    /// Represents a route item.
    Route(Arc<Route>),
    // Represents a history item.
    History(Arc<History>),
}

/// A structure representing a flight route.
#[derive(Clone)]
struct Route {
    /// The departure airport.
    departure: Arc<Airport>,
    /// The destination airport.
    destination: Arc<Airport>,
    /// The aircraft used for the route.
    aircraft: Arc<Aircraft>,
    /// The departure runways.
    departure_runway: Arc<Vec<Runway>>,
    /// The destination runways.
    destination_runway: Arc<Vec<Runway>>,
}

impl TableItem {
    /// Returns the column headers for the table item.
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
            TableItem::History(_) => vec!["ID", "Departure", "Arrival", "Aircraft", "Date"],
        }
    }

    /// Returns the data for the table item.
    fn get_data(&self, db: &mut impl AircraftOperations) -> Vec<Cow<'_, str>> {
        match self {
            TableItem::Airport(airport) => vec![
                Cow::Owned(airport.ID.to_string()),
                Cow::Borrowed(&airport.Name),
                Cow::Borrowed(&airport.ICAO),
            ],
            TableItem::Aircraft(aircraft) => vec![
                Cow::Owned(aircraft.id.to_string()),
                Cow::Borrowed(&aircraft.variant),
                Cow::Borrowed(&aircraft.manufacturer),
                Cow::Owned(aircraft.flown.to_string()),
            ],
            TableItem::Route(route) => {
                let max_departure_runway = route
                    .departure_runway
                    .iter()
                    .max_by(|a, b| a.Length.cmp(&b.Length))
                    .map(|r| r.Length.to_string())
                    .unwrap_or_default();

                let max_destination_runway = route
                    .destination_runway
                    .iter()
                    .max_by(|a, b| a.Length.cmp(&b.Length))
                    .map(|r| r.Length.to_string())
                    .unwrap_or_default();

                let distance = calculate_haversine_distance_nm(
                    route.departure.as_ref(),
                    route.destination.as_ref(),
                );

                vec![
                    Cow::Borrowed(&route.departure.Name),
                    Cow::Borrowed(&route.departure.ICAO),
                    Cow::Owned(max_departure_runway),
                    Cow::Borrowed(&route.destination.Name),
                    Cow::Borrowed(&route.destination.ICAO),
                    Cow::Owned(max_destination_runway),
                    Cow::Borrowed(&route.aircraft.manufacturer),
                    Cow::Borrowed(&route.aircraft.variant),
                    Cow::Owned(distance.to_string()),
                ]
            }
            TableItem::History(history) => {
                let aircraft_id = history.aircraft;
                let aircraft = db.get_aircraft_by_id(aircraft_id).unwrap();
                let aircraft_str = format!("{} {}", aircraft.manufacturer, aircraft.variant);

                vec![
                    Cow::Owned(history.id.to_string()),
                    Cow::Borrowed(&history.departure_icao),
                    Cow::Borrowed(&history.arrival_icao),
                    Cow::Owned(aircraft_str),
                    Cow::Borrowed(&history.date),
                ]
            }
        }
    }

    /// Checks if the item matches the search query.
    ///
    /// # Arguments
    ///
    /// * `query` - The search query string.
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
            TableItem::Route(route) => {
                route.departure.Name.to_lowercase().contains(&query)
                    || route.departure.ICAO.to_lowercase().contains(&query)
                    || route.destination.Name.to_lowercase().contains(&query)
                    || route.destination.ICAO.to_lowercase().contains(&query)
                    || route.aircraft.manufacturer.to_lowercase().contains(&query)
                    || route.aircraft.variant.to_lowercase().contains(&query)
            }
            TableItem::History(_) => false,
        }
    }
}

#[derive(Default)]
struct SearchState {
    /// The current search query.
    query: String,
    /// The items filtered based on the search query.
    filtered_items: Vec<Arc<TableItem>>,
}

/// The main GUI application.
pub struct Gui<'a> {
    /// The database pool for accessing data.
    database_pool: &'a mut DatabasePool,
    /// The items currently displayed in the GUI.
    displayed_items: Vec<Arc<TableItem>>,
    /// All available aircraft.
    all_aircraft: Vec<Arc<Aircraft>>,
    /// All available airports.
    all_airports: Vec<Arc<Airport>>,
    /// A map of all runways.
    all_runways: HashMap<i32, Arc<Vec<Runway>>>,
    /// State for handling popups.
    popup_state: PopupState,
    /// State for handling search.
    search_state: SearchState,
    /// Spatial index of airports for efficient queries.
    spatial_airports: RTree<SpatialAirport>,
}

#[derive(Default)]
struct PopupState {
    /// Whether to show the alert popup.
    show_alert: bool,
    /// The currently selected route.
    selected_route: Option<Arc<Route>>,
    /// Whether the routes are generated from not flown aircraft list.
    routes_from_not_flown: bool,
}

/// A spatial index object for airports.
pub struct SpatialAirport {
    pub airport: Arc<Airport>,
}

/// Implement the RTreeObject trait for SpatialAirport for the RTree index.
impl RTreeObject for SpatialAirport {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        let point = [self.airport.Latitude, self.airport.Longtitude];
        AABB::from_point(point)
    }
}

impl<'a> Gui<'a> {
    /// Creates a new GUI instance.
    ///
    /// # Arguments
    ///
    /// * `_cc` - The creation context.
    /// * `database_pool` - A mutable reference to the database pool.
    pub fn new(_cc: &eframe::CreationContext, database_pool: &'a mut DatabasePool) -> Self {
        let all_aircraft = database_pool
            .get_all_aircraft()
            .expect("Failed to load aircraft");
        let all_airports = database_pool
            .get_airports()
            .expect("Failed to load airports");
        let runway_data = database_pool.get_runways().expect("Failed to load runways");

        let mut runway_map: HashMap<i32, Vec<Runway>> = HashMap::new();
        for runway in runway_data {
            runway_map.entry(runway.AirportID).or_default().push(runway);
        }

        let all_runways: HashMap<i32, Arc<Vec<Runway>>> = runway_map
            .into_iter()
            .map(|(id, runways)| (id, Arc::new(runways)))
            .collect();

        let all_aircraft: Vec<Arc<Aircraft>> = all_aircraft.into_iter().map(Arc::new).collect();
        let all_airports: Vec<Arc<Airport>> = all_airports.into_iter().map(Arc::new).collect();

        let spatial_airports = RTree::bulk_load(
            all_airports
                .iter()
                .map(|airport| SpatialAirport {
                    airport: Arc::clone(airport),
                })
                .collect(),
        );

        Gui {
            database_pool,
            displayed_items: Vec::new(),
            all_aircraft,
            all_airports,
            all_runways,
            popup_state: PopupState::default(),
            search_state: SearchState::default(),
            spatial_airports,
        }
    }

    /// Generates a list of random routes.
    fn generate_random_routes(&mut self) -> Result<Vec<Route>, String> {
        self.generate_random_routes_generic(&self.all_aircraft, GENERATE_AMOUNT)
    }

    /// Generates random routes for aircraft that have not been flown yet.
    fn generate_random_not_flown_aircraft_routes(&self) -> Result<Vec<Route>, String> {
        let not_flown_aircraft: Vec<_> = self
            .all_aircraft
            .iter()
            .filter(|aircraft| aircraft.flown == 0)
            .cloned()
            .collect();

        self.generate_random_routes_generic(&not_flown_aircraft, GENERATE_AMOUNT)
    }

    /// Generates random routes using a generic aircraft list.
    ///
    /// # Arguments
    ///
    /// * `aircraft_list` - A slice of aircraft to generate routes for.
    /// * `amount` - The number of routes to generate.
    fn generate_random_routes_generic(
        &self,
        aircraft_list: &[Arc<Aircraft>],
        amount: usize,
    ) -> Result<Vec<Route>, String> {
        let start_time = Instant::now();

        let routes: Vec<Route> = (0..amount)
            .into_par_iter()
            .filter_map(|_| {
                let mut rand = rand::thread_rng();
                let aircraft = aircraft_list.choose(&mut rand)?;

                loop {
                    let departure = self.all_airports.choose(&mut rand)?;
                    let departure_runways = self.all_runways.get(&departure.ID)?;
                    let longest_runway = departure_runways.iter().max_by_key(|r| r.Length)?;

                    if let Some(takeoff_distance) = aircraft.takeoff_distance {
                        if takeoff_distance as f64 * M_TO_FT > longest_runway.Length as f64 {
                            continue;
                        }
                    }

                    if let Ok(destination) = get_destination_airport_with_suitable_runway_fast(
                        aircraft,
                        departure,
                        &self.spatial_airports,
                        &self.all_runways,
                    ) {
                        let destination_arc = Arc::new(destination);
                        let departure_runways = Arc::clone(departure_runways);
                        let destination_runways = self.all_runways.get(&destination_arc.ID)?;
                        let destination_runways = Arc::clone(destination_runways);
                        return Some(Route {
                            departure: Arc::clone(departure),
                            destination: Arc::clone(&destination_arc),
                            aircraft: Arc::clone(aircraft),
                            departure_runway: departure_runways,
                            destination_runway: destination_runways,
                        });
                    } else {
                        continue;
                    }
                }
            })
            .collect();

        let duration = start_time.elapsed();
        log::info!("Generated {} routes in {:?}", routes.len(), duration);

        Ok(routes)
    }

    /// Updates the UI buttons.
    ///
    /// # Arguments
    ///
    /// * `ui` - The UI context.
    fn update_buttons(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            if ui
                .button("Select random aircraft")
                .on_hover_text("Select a random aircraft from the database")
                .clicked()
            {
                if let Some(aircraft) = self.all_aircraft.choose(&mut rand::thread_rng()) {
                    self.displayed_items =
                        vec![Arc::new(TableItem::Aircraft(Arc::clone(aircraft)))];
                    self.search_state.query.clear();
                }
            }

            if ui.button("Get random airport").clicked() {
                if let Some(airport) = self.all_airports.choose(&mut rand::thread_rng()) {
                    self.displayed_items = vec![Arc::new(TableItem::Airport(Arc::clone(airport)))];
                    self.search_state.query.clear();
                }
            }

            if ui.button("List all airports").clicked() {
                self.displayed_items = self
                    .all_airports
                    .iter()
                    .map(|airport| Arc::new(TableItem::Airport(Arc::clone(airport))))
                    .collect();
                self.search_state.query.clear();
            }

            if ui.button("List history").clicked() {
                let history = self
                    .database_pool
                    .get_history()
                    .expect("Failed to load history");
                self.displayed_items = history
                    .iter()
                    .map(|history| Arc::new(TableItem::History(Arc::new(history.clone()))))
                    .collect();
                self.search_state.query.clear();
            }

            if ui.button("Random route").clicked() {
                self.displayed_items.clear();
                self.popup_state.routes_from_not_flown = false;

                if let Ok(routes) = self.generate_random_routes() {
                    self.displayed_items.extend(
                        routes
                            .into_iter()
                            .map(|route| Arc::new(TableItem::Route(Arc::new(route)))),
                    );
                }
            }

            if ui.button("Random not flown aircraft routes").clicked() {
                self.displayed_items.clear();
                self.popup_state.routes_from_not_flown = true;

                if let Ok(routes) = self.generate_random_not_flown_aircraft_routes() {
                    self.displayed_items.extend(
                        routes
                            .into_iter()
                            .map(|route| Arc::new(TableItem::Route(Arc::new(route)))),
                    );
                }
            }
        });
    }

    /// Updates the search bar UI component.
    ///
    /// # Arguments
    ///
    /// * `ui` - The UI context.
    fn update_search_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.add(
                TextEdit::singleline(&mut self.search_state.query).hint_text("Type to search..."),
            );
        });
    }

    /// Updates the table UI component.
    ///
    /// # Arguments
    ///
    /// * `ui` - The UI context.
    fn update_table(&mut self, ui: &mut egui::Ui) {
        if let Some(first_item) = self.search_state.filtered_items.first() {
            let table = self.build_table(ui, first_item);
            self.populate_table(table);
        }
    }

    /// Builds the table UI component.
    ///
    /// # Arguments
    ///
    /// * `ui` - The UI context.
    /// * `first_item` - The first item to determine the table structure.
    fn build_table<'t>(&self, ui: &'t mut egui::Ui, first_item: &TableItem) -> TableBuilder<'t> {
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
    }

    /// Populates the table with data.
    ///
    /// # Arguments
    ///
    /// * `table` - The table builder instance.
    fn populate_table(&mut self, table: TableBuilder) {
        let row_height = 30.0;
        let mut create_more_routes = false;
        let filtered_items = &self.search_state.filtered_items;

        table
            .header(20.0, |mut header| {
                if let Some(first_item) = filtered_items.first() {
                    for name in first_item.get_columns() {
                        header.col(|ui| {
                            ui.label(name);
                        });
                    }
                    if let TableItem::Route(_) = first_item.as_ref() {
                        header.col(|ui| {
                            ui.label("Actions");
                        });
                    }
                }
            })
            .body(|body| {
                body.rows(row_height, filtered_items.len(), |mut row| {
                    let item = &filtered_items[row.index()];

                    // Display regular columns
                    for name in item.get_data(self.database_pool) {
                        row.col(|ui| {
                            ui.label(name);
                        });
                    }

                    // Handle route-specific columns
                    if let TableItem::Route(route) = item.as_ref() {
                        if row.index() == filtered_items.len() - 1 {
                            create_more_routes = true;
                        }

                        row.col(|ui| {
                            if ui.button("Select").clicked() {
                                self.popup_state.show_alert = true;
                                self.popup_state.selected_route = Some(Arc::clone(route));
                            }
                        });
                    }
                });
            });

        if create_more_routes {
            self.load_more_routes_if_needed();
        }
    }

    /// Shows the modal popup for route selection.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The egui context.
    fn show_modal_popup(&mut self, ctx: &egui::Context) {
        let modal = egui::Modal::new(Id::NULL);
        modal.show(ctx, |ui| {
            let route = self.popup_state.selected_route.as_ref().unwrap();
            let route_clone = Arc::clone(route);
            let distance = calculate_haversine_distance_nm(
                route.departure.as_ref(),
                route.destination.as_ref(),
            ) as f64;

            ui.label(format!(
                "Departure: {} ({})",
                route.departure.Name, route.departure.ICAO
            ));
            ui.label(format!(
                "Destination: {} ({})",
                route.destination.Name, route.destination.ICAO
            ));
            ui.label(format!("Distance: {:.2} NM", distance));
            ui.label(format!(
                "Aircraft: {} {}",
                route.aircraft.manufacturer, route.aircraft.variant
            ));

            ui.separator();
            ui.horizontal(|ui| {
                if self.popup_state.routes_from_not_flown && ui.button("Mark as flown").clicked() {
                    self.handle_mark_flown_button(&route_clone);
                }
                if ui.button("Close").clicked() {
                    self.popup_state.show_alert = false;
                }
            });
        });
    }

    /// Handles the action when the "Mark as flown" button is pressed.
    ///
    /// # Arguments
    ///
    /// * `route` - The route to mark as flown.
    fn handle_mark_flown_button(&mut self, route: &Route) {
        self.popup_state.show_alert = false;
        self.database_pool
            .add_to_history(
                route.departure.as_ref(),
                route.destination.as_ref(),
                route.aircraft.as_ref(),
            )
            .expect("Failed to add route to history");

        let mut aircraft = (*route.aircraft).clone();
        aircraft.date_flown = Some(chrono::Local::now().format("%Y-%m-%d").to_string());
        aircraft.flown = 1;

        self.database_pool
            .update_aircraft(&aircraft)
            .expect("Failed to update aircraft");

        let all_aircraft = self
            .database_pool
            .get_all_aircraft()
            .expect("Failed to load aircraft");
        self.all_aircraft = all_aircraft.into_iter().map(Arc::new).collect();
    }

    /// Handles user input and updates state.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The egui context.
    fn handle_input(&mut self, ctx: &egui::Context) {
        if self.popup_state.show_alert {
            self.show_modal_popup(ctx);
        }

        self.handle_search();
    }

    /// Filters the displayed items based on the search query.
    fn handle_search(&mut self) {
        self.search_state.filtered_items = if self.search_state.query.is_empty() {
            self.displayed_items.iter().map(Arc::clone).collect()
        } else {
            self.displayed_items
                .iter()
                .filter(|item| item.matches_query(&self.search_state.query))
                .map(Arc::clone)
                .collect()
        };
    }

    /// Loads more routes if needed.
    fn load_more_routes_if_needed(&mut self) {
        if !self.search_state.query.is_empty() {
            return;
        }

        if let Ok(routes) = if self.popup_state.routes_from_not_flown {
            self.generate_random_not_flown_aircraft_routes()
        } else {
            self.generate_random_routes()
        } {
            self.displayed_items.extend(
                routes
                    .into_iter()
                    .map(|route| Arc::new(TableItem::Route(Arc::new(route)))),
            );
        }
    }

    /// Renders the user interface.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The egui context.
    fn render_ui(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_enabled_ui(!self.popup_state.show_alert, |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                    self.update_buttons(ui);
                    ui.add_space(50.0);

                    ui.vertical(|ui| {
                        self.update_search_bar(ui);

                        self.update_table(ui);
                    });
                });
            });
        });
    }
}

impl eframe::App for Gui<'_> {
    /// Updates the application state and the user interface.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The egui context.
    /// * `_frame` - The eframe frame.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_input(ctx);
        self.render_ui(ctx);
    }
}
