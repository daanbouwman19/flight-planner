use crate::database::DatabasePool;
use crate::models::{Aircraft, Airport, Runway};
use crate::modules::routes::RouteGenerator;
use crate::traits::{AircraftOperations, AirportOperations, HistoryOperations};
use eframe::egui::{self};
use log;
use rstar::{RTree, RTreeObject, AABB};
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

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
    id: String,
    /// The variant of the aircraft.
    variant: String,
    /// The manufacturer of the aircraft.
    manufacturer: String,
    /// The number of times the aircraft has been flown.
    flown: String,
}

impl ListItemAircraft {
    /// Creates a new `ListItemAircraft` from an `Aircraft`.
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
    pub fn get_data(&self) -> Vec<Cow<'_, str>> {
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
    pub fn matches_query(&self, query: &str) -> bool {
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
    /// The items currently displayed in the GUI.
    pub displayed_items: Vec<Arc<TableItem>>,
    /// All available aircraft.
    pub all_aircraft: Vec<Arc<Aircraft>>,
    /// State for handling popups.
    pub popup_state: PopupState,
    /// State for handling search.
    pub search_state: SearchState,
    pub route_generator: RouteGenerator,
    /// Currently selected aircraft for route generation.
    pub selected_aircraft: Option<Arc<Aircraft>>,
    /// Search text for aircraft selection.
    pub aircraft_search: String,
    /// Whether the aircraft dropdown is open.
    pub aircraft_dropdown_open: bool,
    /// The ICAO code of the departure airport.
    pub departure_airport_icao: String,
    /// Cached validation result for departure airport
    pub departure_airport_valid: Option<bool>,
    /// Last validated departure airport ICAO to detect changes
    pub last_validated_departure_icao: String,
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
            displayed_items: Vec::new(),
            all_aircraft,
            popup_state: PopupState::default(),
            search_state: SearchState::default(),
            route_generator,
            selected_aircraft: None,
            aircraft_search: String::new(),
            aircraft_dropdown_open: false,
            departure_airport_icao: String::new(),
            departure_airport_valid: None,
            last_validated_departure_icao: String::new(),
        }
    }

    // Getter methods for better encapsulation
    /// Gets the current departure airport ICAO code.
    pub fn get_departure_airport_icao(&self) -> &str {
        &self.departure_airport_icao
    }

    /// Gets the departure airport validation result if available.
    pub fn get_departure_airport_validation(&self) -> Option<bool> {
        self.departure_airport_valid
    }

    /// Gets a reference to the available airports.
    pub fn get_available_airports(&self) -> &[Arc<Airport>] {
        &self.route_generator.all_airports
    }

    // Helper methods for state management
    /// Clears the departure airport validation cache.
    pub fn clear_departure_validation_cache(&mut self) {
        self.departure_airport_valid = None;
        self.last_validated_departure_icao.clear();
    }

    /// Sets the departure airport validation result and updates the cache.
    pub fn set_departure_validation(&mut self, icao: &str, is_valid: bool) {
        self.departure_airport_valid = Some(is_valid);
        self.last_validated_departure_icao = icao.to_string();
    }

    /// Checks if departure airport validation needs to be refreshed.
    pub fn needs_departure_validation_refresh(&self) -> bool {
        self.last_validated_departure_icao != self.departure_airport_icao
    }


    /// Handles the action when the "Mark as flown" button is pressed.
    ///
    /// # Arguments
    ///
    /// * `route` - The route to mark as flown.
    pub fn handle_mark_flown_button(&mut self, route: &ListItemRoute) {
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
    pub fn handle_input(&mut self, ctx: &egui::Context) {
        if self.popup_state.show_alert {
            self.show_modal_popup(ctx);
        }
    }

    /// Filters the displayed items based on the search query.
    pub fn handle_search(&mut self) {
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

    /// Resets the UI state to a clean state and triggers search refresh.
    /// This is commonly needed after changing the displayed items.
    /// 
    /// # Arguments
    /// 
    /// * `clear_search_query` - Whether to clear the search query or keep it
    pub fn reset_ui_state_and_refresh(&mut self, clear_search_query: bool) {
        if clear_search_query {
            self.search_state.query.clear();
        }
        self.selected_aircraft = None;
        self.aircraft_search.clear();
        self.aircraft_dropdown_open = false;
        self.handle_search();
    }

    /// Loads more routes if needed.
    pub fn load_more_routes_if_needed(&mut self) {
        // Only load more routes if we're not searching and we have route items
        if !self.search_state.query.is_empty() {
            return;
        }

        // Check if we have route items (infinite scrolling only applies to routes)
        if let Some(first_item) = self.displayed_items.first() {
            if !matches!(first_item.as_ref(), TableItem::Route(_)) {
                return;
            }
        } else {
            return;
        }

        let departure_icao = if self.departure_airport_icao.is_empty() {
            None
        } else {
            Some(self.departure_airport_icao.as_str())
        };
        let routes = if self.popup_state.routes_from_not_flown {
            self.route_generator
                .generate_random_not_flown_aircraft_routes(&self.all_aircraft, departure_icao)
        } else if self.popup_state.routes_for_specific_aircraft {
            // If we're generating routes for a specific aircraft, use the selected aircraft
            if let Some(selected_aircraft) = &self.selected_aircraft {
                self.route_generator
                    .generate_routes_for_aircraft(selected_aircraft, departure_icao)
            } else {
                // If somehow there's no selected aircraft, fall back to all aircraft
                log::warn!("No selected aircraft when generating routes for a specific aircraft");
                self.route_generator
                    .generate_random_routes(&self.all_aircraft, departure_icao)
            }
        } else {
            // Otherwise generate random routes for all aircraft
            self.route_generator
                .generate_random_routes(&self.all_aircraft, departure_icao)
        };

        self.displayed_items.extend(
            routes
                .into_iter()
                .map(|route| Arc::new(TableItem::Route(route))),
        );
        
        // Update filtered items after adding more routes
        self.handle_search();
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
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_enabled_ui(!self.popup_state.show_alert, |ui| {
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
