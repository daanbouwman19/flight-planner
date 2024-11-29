use eframe::egui::{self, TextEdit};
use egui_extras::{Column, TableBuilder};

use crate::{
    models::Aircraft, models::Airport, AircraftOperations, AirportOperations, DatabaseConnections,
};

// Define an enum to represent different types of items to display
enum TableItem {
    Airport(Airport),
    Aircraft(Aircraft),
    // You can add more variants here, like Route(Route), etc.
}

pub struct Gui<'a> {
    database_connections: &'a mut DatabaseConnections,
    airports: Vec<Airport>,
    displayed_items: Vec<TableItem>, // Stores the items to be displayed in the table
    search_query: String,
    current_data_type: DataType, // Tracks the type of data currently displayed
}

// Enum to track the current data type displayed
enum DataType {
    Airport,
    Aircraft,
    // Add more types as needed
}

impl<'a> Gui<'a> {
    pub fn new(
        _cc: &eframe::CreationContext,
        database_connections: &'a mut DatabaseConnections,
    ) -> Self {
        let airports = database_connections.get_airports().unwrap_or_default();

        // Initially display all airports
        let displayed_items = airports.iter().cloned().map(TableItem::Airport).collect();

        Gui {
            database_connections,
            airports,
            displayed_items,
            search_query: String::new(), // Initialize with an empty string
            current_data_type: DataType::Airport, // Start with airports
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
                        // Get a random aircraft and update the displayed list
                        if let Ok(aircraft) = self.database_connections.random_aircraft() {
                            self.displayed_items = vec![TableItem::Aircraft(aircraft)];
                            self.current_data_type = DataType::Aircraft;
                            self.search_query.clear(); // Clear search query
                        }
                    }

                    if ui.button("Get random airport").clicked() {
                        // Get a random airport and update the displayed list
                        if let Ok(airport) = self.database_connections.get_random_airport() {
                            self.displayed_items = vec![TableItem::Airport(airport)];
                            self.current_data_type = DataType::Airport;
                            self.search_query.clear(); // Clear search query
                        }
                    }

                    if ui.button("List all airports").clicked() {
                        // List all airports by setting displayed_items to all airports
                        self.displayed_items = self
                            .airports
                            .iter()
                            .cloned()
                            .map(TableItem::Airport)
                            .collect();
                        self.current_data_type = DataType::Airport;
                        self.search_query.clear(); // Clear search query
                    }
                });

                // Adding some spacing between buttons and table
                ui.add_space(50.0);

                // Right side - Table, with search and dynamic resizing
                let available_width = ui.available_width();

                // Create a vertical container for the table with dynamic width adjustment
                ui.vertical(|ui| {
                    ui.set_min_width(available_width); // Set the width to take all available space

                    // Add a search bar for filtering the items
                    ui.horizontal(|ui| {
                        ui.label("Search:");
                        ui.add(
                            TextEdit::singleline(&mut self.search_query)
                                .hint_text("Type to search..."),
                        );
                    });

                    // Filter items based on the search query
                    let filtered_items: Vec<&TableItem> = if self.search_query.is_empty() {
                        self.displayed_items.iter().collect() // No filter if query is empty
                    } else {
                        self.displayed_items
                            .iter()
                            .filter(|item| match item {
                                TableItem::Airport(airport) => {
                                    airport
                                        .Name
                                        .to_lowercase()
                                        .contains(&self.search_query.to_lowercase())
                                        || airport
                                            .ICAO
                                            .to_lowercase()
                                            .contains(&self.search_query.to_lowercase())
                                        || airport.ID.to_string().contains(&self.search_query)
                                }
                                TableItem::Aircraft(aircraft) => {
                                    aircraft
                                        .variant
                                        .to_lowercase()
                                        .contains(&self.search_query.to_lowercase())
                                        || aircraft
                                            .manufacturer
                                            .to_lowercase()
                                            .contains(&self.search_query.to_lowercase())
                                        || aircraft.id.to_string().contains(&self.search_query)
                                }
                            })
                            .collect()
                    };

                    // Define the columns of the table based on the current data type
                    let (columns, get_data_functions): (
                        Vec<&str>,
                        Vec<Box<dyn Fn(&TableItem) -> String>>,
                    ) = match self.current_data_type {
                        DataType::Airport => (
                            vec!["ID", "Name", "ICAO"],
                            vec![
                                Box::new(|item| match item {
                                    TableItem::Airport(airport) => airport.ID.to_string(),
                                    _ => String::new(),
                                }),
                                Box::new(|item| match item {
                                    TableItem::Airport(airport) => airport.Name.clone(),
                                    _ => String::new(),
                                }),
                                Box::new(|item| match item {
                                    TableItem::Airport(airport) => airport.ICAO.clone(),
                                    _ => String::new(),
                                }),
                            ],
                        ),
                        DataType::Aircraft => (
                            vec!["ID", "Model", "Registration"],
                            vec![
                                Box::new(|item| match item {
                                    TableItem::Aircraft(aircraft) => aircraft.id.to_string(),
                                    _ => String::new(),
                                }),
                                Box::new(|item| match item {
                                    TableItem::Aircraft(aircraft) => aircraft.variant.clone(),
                                    _ => String::new(),
                                }),
                                Box::new(|item| match item {
                                    TableItem::Aircraft(aircraft) => aircraft.manufacturer.clone(),
                                    _ => String::new(),
                                }),
                            ],
                        ),
                        // Add more DataType cases as needed
                    };

                    let row_height = 30.0;

                    // Build the table
                    let mut table = TableBuilder::new(ui)
                        .striped(true)
                        .resizable(true)
                        .min_scrolled_height(0.0); // Ensure the table stretches fully vertically

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
                                    for get_data in &get_data_functions {
                                        row.col(|ui| {
                                            ui.label(get_data(item));
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
