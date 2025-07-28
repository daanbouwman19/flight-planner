use crate::database::DatabasePool;
use crate::models::{Aircraft, Airport, Runway};
use crate::modules::routes::RouteGenerator;
use crate::traits::{AircraftOperations, AirportOperations, HistoryOperations};
use eframe::egui::{self};
use egui::Id;
use log;
use rstar::{RTree, RTreeObject, AABB};
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

/// A trait for items that can be displayed in the table.
pub trait TableEntry {
    /// Returns the column headers for the table.
    fn get_columns(&self) -> Vec<&'static str>;
    /// Returns the data for the table row.
    fn get_data(&self) -> Vec<Cow<'_, str>>;
    /// Checks if the item matches the search query.
    fn matches_query(&self, query: &str) -> bool;
}

/// An enum representing the items that can be displayed in the table.
pub enum TableItem {
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
pub struct ListItemHistory {
    /// The ID of the history item.
    pub id: String,
    /// The departure ICAO code.
    pub departure_icao: String,
    /// The arrival ICAO code.
    pub arrival_icao: String,
    /// The aircraft ID.
    pub aircraft_name: String,
    /// The date of the flight.
    pub date: String,
}

#[derive(Clone)]
pub struct ListItemAircraft {
    /// The ID of the aircraft.
    pub id: String,
    /// The variant of the aircraft.
    pub variant: String,
    /// The manufacturer of the aircraft.
    pub manufacturer: String,
    /// The number of times the aircraft has been flown.
    pub flown: String,
}

impl ListItemAircraft {
    /// Creates a new `ListItemAircraft` from an Aircraft.
    ///
    /// # Arguments
    ///
    /// * `aircraft` - The aircraft to convert.
    pub fn from_aircraft(aircraft: &Aircraft) -> Self {
        Self {
            id: aircraft.id.to_string(),
            variant: aircraft.variant.clone(),
            manufacturer: aircraft.manufacturer.clone(),
            flown: if aircraft.flown > 0 { "true".to_string() } else { "false".to_string() },
        }
    }
}

#[derive(Clone)]
pub struct ListItemAirport {
    /// The ID of the airport.
    pub id: String,
    /// The name of the airport.
    pub name: String,
    /// The ICAO code of the airport.
    pub icao: String,
}

impl TableItem {
    /// Returns the column headers for the table item.
    pub fn get_columns(&self) -> Vec<&'static str> {
        match self {
            Self::Airport(item) => item.get_columns(),
            Self::Aircraft(item) => item.get_columns(),
            Self::Route(item) => item.get_columns(),
            Self::History(item) => item.get_columns(),
        }
    }

    /// Returns the data for the table item.
    pub fn get_data(&self) -> Vec<Cow<'_, str>> {
        match self {
            Self::Airport(item) => item.get_data(),
            Self::Aircraft(item) => item.get_data(),
            Self::Route(item) => item.get_data(),
            Self::History(item) => item.get_data(),
        }
    }

    /// Checks if the item matches the search query.
    ///
    /// # Arguments
    ///
    /// * `query` - The search query string.
    pub fn matches_query(&self, query: &str) -> bool {
        match self {
            Self::Airport(item) => item.matches_query(query),
            Self::Aircraft(item) => item.matches_query(query),
            Self::Route(item) => item.matches_query(query),
            Self::History(item) => item.matches_query(query),
        }
    }
}

impl TableEntry for ListItemAirport {
    fn get_columns(&self) -> Vec<&'static str> {
        vec!["ID", "Name", "ICAO"]
    }

    fn get_data(&self) -> Vec<Cow<'_, str>> {
        vec![
            Cow::Borrowed(&self.id),
            Cow::Borrowed(&self.name),
            Cow::Borrowed(&self.icao),
        ]
    }

    fn matches_query(&self, query: &str) -> bool {
        let query = query.to_lowercase();
        self.name.to_lowercase().contains(&query)
            || self.icao.to_lowercase().contains(&query)
            || self.id.to_string().contains(&query)
    }
}

impl TableEntry for ListItemAircraft {
    fn get_columns(&self) -> Vec<&'static str> {
        vec!["ID", "Model", "Registration", "Flown"]
    }

    fn get_data(&self) -> Vec<Cow<'_, str>> {
        vec![
            Cow::Borrowed(&self.id),
            Cow::Borrowed(&self.manufacturer),
            Cow::Borrowed(&self.variant),
            Cow::Borrowed(&self.flown),
        ]
    }

    fn matches_query(&self, query: &str) -> bool {
        let query = query.to_lowercase();
        self.variant.to_lowercase().contains(&query)
            || self.manufacturer.to_lowercase().contains(&query)
            || self.id.to_string().contains(&query)
    }
}

impl TableEntry for ListItemRoute {
    fn get_columns(&self) -> Vec<&'static str> {
        vec![
            "Departure",
            "ICAO",
            "Runway length",
            "Destination",
            "ICAO",
            "Runway length",
            "Manufacturer",
            "Aircraft",
            "Distance",
        ]
    }

    fn get_data(&self) -> Vec<Cow<'_, str>> {
        vec![
            Cow::Borrowed(&self.departure.Name),
            Cow::Borrowed(&self.departure.ICAO),
            Cow::Borrowed(&self.departure_runway_length),
            Cow::Borrowed(&self.destination.Name),
            Cow::Borrowed(&self.destination.ICAO),
            Cow::Borrowed(&self.destination_runway_length),
            Cow::Borrowed(&self.aircraft.manufacturer),
            Cow::Borrowed(&self.aircraft.variant),
            Cow::Borrowed(&self.route_length),
        ]
    }

    fn matches_query(&self, query: &str) -> bool {
        let query = query.to_lowercase();
        self.departure.Name.to_lowercase().contains(&query)
            || self.departure.ICAO.to_lowercase().contains(&query)
            || self.destination.Name.to_lowercase().contains(&query)
            || self.destination.ICAO.to_lowercase().contains(&query)
            || self.aircraft.manufacturer.to_lowercase().contains(&query)
            || self.aircraft.variant.to_lowercase().contains(&query)
    }
}

impl TableEntry for ListItemHistory {
    fn get_columns(&self) -> Vec<&'static str> {
        vec!["ID", "Departure", "Arrival", "Aircraft", "Date"]
    }

    fn get_data(&self) -> Vec<Cow<'_, str>> {
        vec![
            Cow::Borrowed(&self.id),
            Cow::Borrowed(&self.departure_icao),
            Cow::Borrowed(&self.arrival_icao),
            Cow::Borrowed(&self.aircraft_name),
            Cow::Borrowed(&self.date),
        ]
    }

    fn matches_query(&self, _query: &str) -> bool {
        false
    }
}

#[derive(Default)]
pub struct SearchState {
    /// The current search query.
    pub query: String,
    /// The items filtered based on the search query.
    pub filtered_items: Vec<Arc<TableItem>>,
    /// The last time a search was requested (for debouncing).
    pub last_search_request: Option<Instant>,
    /// Whether a search is pending (for debouncing).
    pub search_pending: bool,
}

/// The main GUI application.
pub struct Gui<'a> {
    /// The database pool for accessing data.
    pub database_pool: &'a mut DatabasePool,
    /// The state of the GUI.
    pub state: GuiState,
}

#[derive(Default)]
pub struct GuiState {
    /// The items currently displayed in the GUI.
    pub displayed_items: Vec<Arc<TableItem>>,
    /// All available aircraft.
    pub all_aircraft: Vec<Arc<Aircraft>>,
    /// State for handling popups.
    pub popup_state: PopupState,
    /// State for handling search.
    pub search_state: SearchState,
    pub route_generator: Option<RouteGenerator>,
    /// Currently selected aircraft for route generation.
    pub selected_aircraft: Option<Arc<Aircraft>>,
    /// Search text for aircraft selection.
    pub aircraft_search: String,
    /// Whether the aircraft dropdown is open.
    pub aircraft_dropdown_open: bool,
}

#[derive(Default)]
pub struct PopupState {
    /// Whether to show the alert popup.
    pub show_alert: bool,
    /// The currently selected route.
    pub selected_route: Option<ListItemRoute>,
    /// Whether the routes are generated from not flown aircraft list.
    pub routes_from_not_flown: bool,
    /// Whether the current routes are for a specific aircraft.
    pub routes_for_specific_aircraft: bool,
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
            state: GuiState {
                all_aircraft,
                route_generator: Some(route_generator),
                ..Default::default()
            },
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
            let route = self.state.popup_state.selected_route.as_ref().unwrap();
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
                if self.state.popup_state.routes_from_not_flown
                    && ui.button("Mark as flown").clicked()
                {
                    self.handle_mark_flown_button(&route_clone);
                }
                if ui.button("Close").clicked() {
                    self.state.popup_state.show_alert = false;
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
        self.state.popup_state.show_alert = false;
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
        self.state.all_aircraft = all_aircraft.into_iter().map(Arc::new).collect();
    }

    /// Handles user input and updates state.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The egui context.
    fn handle_input(&mut self, ctx: &egui::Context) {
        if self.state.popup_state.show_alert {
            self.show_modal_popup(ctx);
        }
    }

    /// Filters the displayed items based on the search query.
    pub fn handle_search(&mut self) {
        self.state.search_state.filtered_items = if self.state.search_state.query.is_empty() {
            self.state.displayed_items.iter().map(Arc::clone).collect()
        } else {
            self.state
                .displayed_items
                .iter()
                .filter(|item| item.matches_query(&self.state.search_state.query))
                .map(Arc::clone)
                .collect()
        };
    }

    /// Loads more routes if needed.
    pub fn load_more_routes_if_needed(&mut self) {
        // Only load more routes if we're not searching and we have route items
        if !self.state.search_state.query.is_empty() {
            return;
        }

        // Check if we have route items (infinite scrolling only applies to routes)
        if let Some(first_item) = self.state.displayed_items.first() {
            if !matches!(first_item.as_ref(), TableItem::Route(_)) {
                return;
            }
        } else {
            return;
        }

        let routes = if self.state.popup_state.routes_from_not_flown {
            self.state
                .route_generator
                .as_mut()
                .unwrap()
                .generate_random_not_flown_aircraft_routes(&self.state.all_aircraft)
        } else if self.state.popup_state.routes_for_specific_aircraft {
            // If we're generating routes for a specific aircraft, use the selected aircraft
            if let Some(selected_aircraft) = &self.state.selected_aircraft {
                self.state
                    .route_generator
                    .as_mut()
                    .unwrap()
                    .generate_routes_for_aircraft(selected_aircraft)
            } else {
                // If somehow there's no selected aircraft, fall back to all aircraft
                log::warn!("No selected aircraft when generating routes for a specific aircraft");
                self.state
                    .route_generator
                    .as_mut()
                    .unwrap()
                    .generate_random_routes(&self.state.all_aircraft)
            }
        } else {
            // Otherwise generate random routes for all aircraft
            self.state
                .route_generator
                .as_mut()
                .unwrap()
                .generate_random_routes(&self.state.all_aircraft)
        };

        self.state.displayed_items.extend(
            routes
                .into_iter()
                .map(|route| Arc::new(TableItem::Route(route))),
        );

        // Update filtered items after adding more routes
        self.handle_search();
    }

    /// Renders the user interface.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The egui context.
    fn render_ui(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_enabled_ui(!self.state.popup_state.show_alert, |ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Min), |ui| {
                    // Left panel with buttons - fixed width
                    ui.allocate_ui_with_layout(
                        egui::Vec2::new(250.0, ui.available_height()),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            self.update_buttons(ui);
                        },
                    );

                    ui.separator();

                    // Right panel with search and table - takes remaining space
                    ui.vertical(|ui| {
                        self.update_search_bar(ui);
                        ui.add_space(10.0);
                        ui.separator();
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
