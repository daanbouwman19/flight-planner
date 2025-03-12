use crate::database::DatabasePool;
use crate::models::{Aircraft, Airport, History, Runway};
use crate::modules::routes::RouteGenerator;
use crate::traits::{AircraftOperations, AirportOperations, HistoryOperations};
use eframe::egui::{self, TextEdit};
use egui::Id;
use egui_extras::{Column, TableBuilder};
use rand::prelude::*;
use rstar::{RTree, RTreeObject, AABB};
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;

/// An enum representing the items that can be displayed in the table.
enum TableItem {
    /// Represents an airport item.
    Airport(ListItemAirport),
    /// Represents an aircraft item.
    Aircraft(ListItemAircraft),
    /// Represents a route item.
    Route(ListItemRoute),
    // Represents a history item.
    History(ListItemHistory),
}

/// A structure representing a flight route.
#[derive(Clone)]
pub struct ListItemRoute {
    /// The departure airport.
    pub departure: Arc<Airport>,
    /// The destination airport.
    pub destination: Arc<Airport>,
    /// The aircraft used for the route.
    pub aircraft: Arc<Aircraft>,
    /// The departure runways.
    pub departure_runway_length: String,
    /// The destination runways.
    pub destination_runway_length: String,
    /// route length
    pub route_length: String,
}

#[derive(Clone)]
struct ListItemHistory {
    /// The ID of the history item.
    id: String,
    /// The departure ICAO code.
    departure_icao: String,
    /// The arrival ICAO code.
    arrival_icao: String,
    /// The aircraft ID.
    aircraft_name: String,
    /// The date of the flight.
    date: String,
}

#[derive(Clone)]
struct ListItemAircraft {
    /// The ID of the aircraft.
    id: String,
    /// The variant of the aircraft.
    variant: String,
    /// The manufacturer of the aircraft.
    manufacturer: String,
    /// The number of times the aircraft has been flown.
    flown: String,
}

#[derive(Clone)]
struct ListItemAirport {
    /// The ID of the airport.
    id: String,
    /// The name of the airport.
    name: String,
    /// The ICAO code of the airport.
    icao: String,
}

impl TableItem {
    /// Returns the column headers for the table item.
    fn get_columns(&self) -> Vec<&'static str> {
        match self {
            Self::Airport(_) => vec!["ID", "Name", "ICAO"],
            Self::Aircraft(_) => vec!["ID", "Model", "Registration", "Flown"],
            Self::Route(_) => vec![
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
            Self::History(_) => vec!["ID", "Departure", "Arrival", "Aircraft", "Date"],
        }
    }

    /// Returns the data for the table item.
    fn get_data(&self) -> Vec<Cow<'_, str>> {
        match self {
            Self::Airport(airport) => vec![
                Cow::Borrowed(&airport.id),
                Cow::Borrowed(&airport.name),
                Cow::Borrowed(&airport.icao),
            ],
            Self::Aircraft(aircraft) => vec![
                Cow::Borrowed(&aircraft.id),
                Cow::Borrowed(&aircraft.manufacturer),
                Cow::Borrowed(&aircraft.variant),
                Cow::Borrowed(&aircraft.flown),
            ],
            Self::Route(route) => {
                vec![
                    Cow::Borrowed(&route.departure.Name),
                    Cow::Borrowed(&route.departure.ICAO),
                    Cow::Borrowed(&route.departure_runway_length),
                    Cow::Borrowed(&route.destination.Name),
                    Cow::Borrowed(&route.destination.ICAO),
                    Cow::Borrowed(&route.destination_runway_length),
                    Cow::Borrowed(&route.aircraft.manufacturer),
                    Cow::Borrowed(&route.aircraft.variant),
                    Cow::Borrowed(&route.route_length),
                ]
            }
            Self::History(history) => {
                vec![
                    Cow::Borrowed(&history.id),
                    Cow::Borrowed(&history.departure_icao),
                    Cow::Borrowed(&history.arrival_icao),
                    Cow::Borrowed(&history.aircraft_name),
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
            Self::Airport(airport) => {
                airport.name.to_lowercase().contains(&query)
                    || airport.icao.to_lowercase().contains(&query)
                    || airport.id.to_string().contains(&query)
            }
            Self::Aircraft(aircraft) => {
                aircraft.variant.to_lowercase().contains(&query)
                    || aircraft.manufacturer.to_lowercase().contains(&query)
                    || aircraft.id.to_string().contains(&query)
            }
            Self::Route(route) => {
                route.departure.Name.to_lowercase().contains(&query)
                    || route.departure.ICAO.to_lowercase().contains(&query)
                    || route.destination.Name.to_lowercase().contains(&query)
                    || route.destination.ICAO.to_lowercase().contains(&query)
                    || route.aircraft.manufacturer.to_lowercase().contains(&query)
                    || route.aircraft.variant.to_lowercase().contains(&query)
            }
            Self::History(_) => false,
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
    /// State for handling popups.
    popup_state: PopupState,
    /// State for handling search.
    search_state: SearchState,
    route_generator: RouteGenerator,
}

#[derive(Default)]
struct PopupState {
    /// Whether to show the alert popup.
    show_alert: bool,
    /// The currently selected route.
    selected_route: Option<ListItemRoute>,
    /// Whether the routes are generated from not flown aircraft list.
    routes_from_not_flown: bool,
}

/// A spatial index object for airports.
pub struct SpatialAirport {
    pub airport: Arc<Airport>,
}

/// Implement the `RTreeObject` trait for `SpatialAirport` for the `RTree` index.
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

        let route_generator = RouteGenerator {
            all_airports,
            all_runways,
            spatial_airports,
        };

        Gui {
            database_pool,
            displayed_items: Vec::new(),
            all_aircraft,
            popup_state: PopupState::default(),
            search_state: SearchState::default(),
            route_generator,
        }
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
                if let Some(aircraft) = self.all_aircraft.choose(&mut rand::rng()) {
                    let list_item_aircraft = ListItemAircraft {
                        id: aircraft.id.to_string(),
                        variant: aircraft.variant.clone(),
                        manufacturer: aircraft.manufacturer.clone(),
                        flown: aircraft.flown.to_string(),
                    };
                    self.displayed_items = vec![Arc::new(TableItem::Aircraft(list_item_aircraft))];
                    self.search_state.query.clear();
                }
            }

            if ui.button("Get random airport").clicked() {
                if let Some(airport) = self.route_generator.all_airports.choose(&mut rand::rng()) {
                    let list_item_airport = ListItemAirport {
                        id: airport.ID.to_string(),
                        name: airport.Name.clone(),
                        icao: airport.ICAO.clone(),
                    };

                    self.displayed_items = vec![Arc::new(TableItem::Airport(list_item_airport))];
                    self.search_state.query.clear();
                }
            }

            if ui.button("List all airports").clicked() {
                self.displayed_items = self
                    .route_generator
                    .all_airports
                    .iter()
                    .map(|airport| {
                        Arc::new(TableItem::Airport(ListItemAirport {
                            id: airport.ID.to_string(),
                            name: airport.Name.clone(),
                            icao: airport.ICAO.clone(),
                        }))
                    })
                    .collect();
                self.search_state.query.clear();
            }

            if ui.button("List history").clicked() {
                let history: Vec<History> = self
                    .database_pool
                    .get_history()
                    .expect("Failed to load history");

                self.displayed_items = history
                    .into_iter()
                    .map(|history| {
                        Arc::new(TableItem::History(ListItemHistory {
                            id: history.id.to_string(),
                            departure_icao: history.departure_icao,
                            arrival_icao: history.arrival_icao,
                            aircraft_name: history.aircraft.to_string(),
                            date: history.date,
                        }))
                    })
                    .collect();

                self.search_state.query.clear();
            }

            if ui.button("Random route").clicked() {
                self.displayed_items.clear();
                self.popup_state.routes_from_not_flown = false;

                let routes = self
                    .route_generator
                    .generate_random_routes(&self.all_aircraft);

                self.displayed_items.extend(
                    routes
                        .into_iter()
                        .map(|route| Arc::new(TableItem::Route(route))),
                );
            }

            if ui.button("Random not flown aircraft routes").clicked() {
                self.displayed_items.clear();
                self.popup_state.routes_from_not_flown = true;

                let routes = self
                    .route_generator
                    .generate_random_not_flown_aircraft_routes(&self.all_aircraft);

                self.displayed_items.extend(
                    routes
                        .into_iter()
                        .map(|route| Arc::new(TableItem::Route(route))),
                );
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
            let table = Self::build_table(ui, first_item);
            self.populate_table(table);
        }
    }

    /// Builds the table UI component.
    ///
    /// # Arguments
    ///
    /// * `ui` - The UI context.
    /// * `first_item` - The first item to determine the table structure.
    fn build_table<'t>(ui: &'t mut egui::Ui, first_item: &TableItem) -> TableBuilder<'t> {
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
                    for name in item.get_data() {
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
                                self.popup_state.selected_route = Some(route.clone());
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
            let route_clone = Arc::new(route.clone());

            ui.label(format!(
                "Departure: {} ({})",
                route.departure.Name, route.departure.ICAO
            ));
            ui.label(format!(
                "Destination: {} ({})",
                route.destination.Name, route.destination.ICAO
            ));
            ui.label(format!("Distance: {} nm", route.route_length));
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
    fn handle_mark_flown_button(&mut self, route: &ListItemRoute) {
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

        let routes = if self.popup_state.routes_from_not_flown {
            self.route_generator
                .generate_random_not_flown_aircraft_routes(&self.all_aircraft)
        } else {
            self.route_generator
                .generate_random_routes(&self.all_aircraft)
        };

        self.displayed_items.extend(
            routes
                .into_iter()
                .map(|route| Arc::new(TableItem::Route(route))),
        );
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
