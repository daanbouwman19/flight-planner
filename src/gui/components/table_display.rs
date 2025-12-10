use crate::gui::data::{
    ListItemAircraft, ListItemAirport, ListItemHistory, ListItemRoute, TableItem,
};
use crate::gui::events::Event;
use crate::gui::services::popup_service::DisplayMode;
use crate::modules::data_operations::FlightStatistics;
use egui::Ui;
use egui_extras::{Column, TableBuilder, TableRow};
use std::error::Error;
use std::sync::Arc;

// UI Constants
const DISTANCE_FROM_BOTTOM_TO_LOAD_MORE: f32 = 200.0;
const MIN_ITEMS_FOR_LAZY_LOAD: usize = 10;
const ROW_HEIGHT: f32 = 30.0;

/// Type alias for a function that looks up flight rules for a given ICAO code
pub type FlightRulesLookup<'a> = &'a dyn Fn(&str) -> Option<String>;

/// View model for the table display component.
///
/// This struct holds all the data needed to render the table, including
/// the items to display, the current display mode, and any loading states.
pub struct TableDisplayViewModel<'a> {
    /// The items to be displayed in the table.
    pub items: &'a [Arc<TableItem>],
    /// The current display mode, which determines the table's layout and content.
    pub display_mode: &'a DisplayMode,
    /// Indicates whether more routes are currently being loaded.
    pub is_loading_more_routes: bool,
    /// Optional flight statistics to display.
    pub statistics: &'a Option<Result<FlightStatistics, Box<dyn Error + Send + Sync>>>,
    /// Optional function to look up flight rules for an ICAO code.
    pub flight_rules_lookup: Option<FlightRulesLookup<'a>>,
}

// --- Component ---

/// Standard button size for aircraft action buttons
const AIRCRAFT_ACTION_BUTTON_SIZE: [f32; 2] = [120.0, 20.0];

/// A UI component responsible for rendering the main data table.
///
/// This component can display various types of data (routes, airports, history, etc.)
/// based on the current `DisplayMode`. It uses `egui_extras::TableBuilder` for
/// efficient rendering of potentially large datasets.
pub struct TableDisplay;

impl TableDisplay {
    /// Renders the main data table or the statistics view.
    ///
    /// # Arguments
    ///
    /// * `vm` - The `TableDisplayViewModel` containing the data and state for rendering.
    /// * `ui` - A mutable reference to the `egui::Ui` context.
    ///
    /// # Returns
    ///
    /// A `Vec<Event>` containing events triggered by user interactions within the table,
    /// such as button clicks or scrolling to load more items.
    pub fn render(vm: &TableDisplayViewModel, ui: &mut Ui) -> Vec<Event> {
        let mut events = Vec::new();

        if *vm.display_mode == DisplayMode::Statistics {
            Self::render_statistics(vm, ui);
            return events;
        }

        let items_to_display = vm.items;

        if items_to_display.is_empty() {
            ui.label("No items to display");
            return events;
        }

        let mut builder = TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center));

        // Configure columns based on display mode
        builder = match vm.display_mode {
            DisplayMode::RandomRoutes
            | DisplayMode::NotFlownRoutes
            | DisplayMode::SpecificAircraftRoutes => builder
                .column(Column::initial(150.0).resizable(true)) // Aircraft
                .column(Column::initial(150.0).resizable(true)) // From
                .column(Column::initial(80.0).resizable(true)) // Dep Rules
                .column(Column::initial(150.0).resizable(true)) // To
                .column(Column::initial(80.0).resizable(true)) // Dest Rules
                .column(Column::initial(80.0).resizable(true)) // Distance
                .column(Column::remainder()), // Actions (fill rest)
            DisplayMode::History => builder
                .column(Column::initial(150.0).resizable(true)) // Aircraft
                .column(Column::initial(150.0).resizable(true)) // From
                .column(Column::initial(150.0).resizable(true)) // To
                .column(Column::remainder()), // Date Flown
            DisplayMode::Airports | DisplayMode::RandomAirports => builder
                .column(Column::initial(60.0).resizable(true)) // ICAO
                .column(Column::initial(200.0).resizable(true)) // Name
                .column(Column::remainder()), // Runway Length
            DisplayMode::Other => builder
                .column(Column::initial(120.0).resizable(true)) // Manufacturer
                .column(Column::initial(100.0).resizable(true)) // Variant
                .column(Column::initial(60.0).resizable(true)) // ICAO Code
                .column(Column::initial(80.0).resizable(true)) // Range
                .column(Column::initial(100.0).resizable(true)) // Category
                .column(Column::initial(100.0).resizable(true)) // Date Flown
                .column(Column::remainder()), // Action
            DisplayMode::Statistics => builder, // Should be handled early return
        };

        let scroll_area = builder
            .header(20.0, |mut header| match vm.display_mode {
                DisplayMode::RandomRoutes
                | DisplayMode::NotFlownRoutes
                | DisplayMode::SpecificAircraftRoutes => {
                    header.col(|ui| {
                        ui.strong("Aircraft");
                    });
                    header.col(|ui| {
                        ui.strong("From");
                    });
                    header.col(|ui| {
                        ui.strong("Dep Rules");
                    });
                    header.col(|ui| {
                        ui.strong("To");
                    });
                    header.col(|ui| {
                        ui.strong("Dest Rules");
                    });
                    header.col(|ui| {
                        ui.strong("Distance");
                    });
                    header.col(|ui| {
                        ui.strong("Actions");
                    });
                }
                DisplayMode::History => {
                    header.col(|ui| {
                        ui.strong("Aircraft");
                    });
                    header.col(|ui| {
                        ui.strong("From");
                    });
                    header.col(|ui| {
                        ui.strong("To");
                    });
                    header.col(|ui| {
                        ui.strong("Date Flown");
                    });
                }
                DisplayMode::Airports | DisplayMode::RandomAirports => {
                    header.col(|ui| {
                        ui.strong("ICAO");
                    });
                    header.col(|ui| {
                        ui.strong("Name");
                    });
                    header.col(|ui| {
                        ui.strong("Runway Length");
                    });
                }
                DisplayMode::Other => {
                    header.col(|ui| {
                        ui.strong("Manufacturer");
                    });
                    header.col(|ui| {
                        ui.strong("Variant");
                    });
                    header.col(|ui| {
                        ui.strong("ICAO Code");
                    });
                    header.col(|ui| {
                        ui.strong("Range");
                    });
                    header.col(|ui| {
                        ui.strong("Category");
                    });
                    header.col(|ui| {
                        ui.strong("Date Flown");
                    });
                    header.col(|ui| {
                        ui.strong("Action");
                    });
                }
                DisplayMode::Statistics => {}
            })
            .body(|mut body| {
                let count = items_to_display.len()
                    + if vm.is_loading_more_routes { 1 } else { 0 };

                body.rows(ROW_HEIGHT, count, |mut row| {
                    let index = row.index();
                    if index < items_to_display.len() {
                        let item = &items_to_display[index];
                        match item.as_ref() {
                            TableItem::Route(route) => {
                                events.extend(Self::render_route_row(vm, &mut row, route))
                            }
                            TableItem::History(history) => {
                                Self::render_history_row(&mut row, history)
                            }
                            TableItem::Airport(airport) => {
                                Self::render_airport_row(&mut row, airport)
                            }
                            TableItem::Aircraft(aircraft) => {
                                events.extend(Self::render_aircraft_row(&mut row, aircraft))
                            }
                        }
                    } else if vm.is_loading_more_routes {
                        // Render loading spinner row
                        row.col(|ui| {
                            ui.horizontal(|ui| {
                                ui.spinner();
                                ui.label("Loading more routes...");
                            });
                        });
                        // Fill empty columns to avoid layout shift
                        let num_columns = match vm.display_mode {
                            DisplayMode::RandomRoutes
                            | DisplayMode::NotFlownRoutes
                            | DisplayMode::SpecificAircraftRoutes => 7,
                            DisplayMode::History => 4,
                            DisplayMode::Airports | DisplayMode::RandomAirports => 3,
                            DisplayMode::Other => 7,
                            DisplayMode::Statistics => 0,
                        };
                        for _ in 1..num_columns {
                            row.col(|_ui| {});
                        }
                    }
                });
            });

        if let Some(event) = Self::handle_infinite_scrolling(vm, &scroll_area, items_to_display) {
            events.push(event);
        }

        events
    }

    fn handle_infinite_scrolling(
        vm: &TableDisplayViewModel,
        scroll_response: &egui::scroll_area::ScrollAreaOutput<()>,
        items: &[Arc<TableItem>],
    ) -> Option<Event> {
        let is_route_mode = matches!(
            vm.display_mode,
            DisplayMode::RandomRoutes
                | DisplayMode::NotFlownRoutes
                | DisplayMode::SpecificAircraftRoutes
        );

        if !is_route_mode || vm.is_loading_more_routes {
            return None;
        }

        let state = &scroll_response.state;
        let content_size = scroll_response.content_size;
        let available_size = scroll_response.inner_rect.size();
        let scroll_position = state.offset.y;
        let max_scroll = (content_size.y - available_size.y).max(0.0);

        if items.len() >= MIN_ITEMS_FOR_LAZY_LOAD
            && max_scroll > 0.0
            && max_scroll - scroll_position < DISTANCE_FROM_BOTTOM_TO_LOAD_MORE
        {
            return Some(Event::LoadMoreRoutes);
        }
        None
    }

    fn render_flight_rules_cell(ui: &mut egui::Ui, icao: &str, lookup: Option<FlightRulesLookup>) {
        if let Some(lookup) = lookup
            && let Some(rules) = lookup(icao)
        {
            ui.label(rules);
        }
    }

    fn render_route_row(
        vm: &TableDisplayViewModel,
        row: &mut TableRow,
        route: &ListItemRoute,
    ) -> Vec<Event> {
        let mut events = Vec::new();
        row.col(|ui| {
            ui.label(format!(
                "{} {}",
                route.aircraft.manufacturer, route.aircraft.variant
            ));
        });
        row.col(|ui| {
            ui.label(format!(
                "{} ({})",
                route.departure.Name, route.departure.ICAO
            ));
        });
        row.col(|ui| {
            Self::render_flight_rules_cell(ui, &route.departure.ICAO, vm.flight_rules_lookup);
        });
        row.col(|ui| {
            ui.label(format!(
                "{} ({})",
                route.destination.Name, route.destination.ICAO
            ));
        });
        row.col(|ui| {
            Self::render_flight_rules_cell(ui, &route.destination.ICAO, vm.flight_rules_lookup);
        });
        row.col(|ui| {
            ui.label(format!("{:.1} NM", route.route_length));
        });

        row.col(|ui| {
            if matches!(
                vm.display_mode,
                DisplayMode::RandomRoutes
                    | DisplayMode::NotFlownRoutes
                    | DisplayMode::SpecificAircraftRoutes
            ) && ui.button("Select").clicked()
            {
                events.push(Event::RouteSelectedForPopup(route.clone()));
                events.push(Event::SetShowPopup(true));
            }
        });
        events
    }

    fn render_history_row(row: &mut TableRow, history: &ListItemHistory) {
        row.col(|ui| {
            ui.label(&history.aircraft_name);
        });
        row.col(|ui| {
            ui.label(format!(
                "{} ({})",
                history.departure_airport_name, history.departure_icao
            ));
        });
        row.col(|ui| {
            ui.label(format!(
                "{} ({})",
                history.arrival_airport_name, history.arrival_icao
            ));
        });
        row.col(|ui| {
            ui.label(&history.date);
        });
    }

    fn render_airport_row(row: &mut TableRow, airport: &ListItemAirport) {
        row.col(|ui| {
            ui.label(&airport.icao);
        });
        row.col(|ui| {
            ui.label(&airport.name);
        });
        row.col(|ui| {
            ui.label(&airport.longest_runway_length);
        });
    }

    fn render_aircraft_row(row: &mut TableRow, aircraft: &ListItemAircraft) -> Vec<Event> {
        let mut events = Vec::new();
        row.col(|ui| {
            ui.label(&aircraft.manufacturer);
        });
        row.col(|ui| {
            ui.label(&aircraft.variant);
        });
        row.col(|ui| {
            ui.label(&aircraft.icao_code);
        });
        row.col(|ui| {
            ui.label(&aircraft.range);
        });
        row.col(|ui| {
            ui.label(&aircraft.category);
        });
        row.col(|ui| {
            ui.label(&aircraft.date_flown);
        });

        row.col(|ui| {
            let button_text = if aircraft.flown > 0 {
                "Mark Not Flown"
            } else {
                " Mark Flown  "
            };
            if ui
                .add_sized(AIRCRAFT_ACTION_BUTTON_SIZE, egui::Button::new(button_text))
                .clicked()
            {
                events.push(Event::ToggleAircraftFlownStatus(aircraft.id));
            }
        });
        events
    }

    fn render_statistics(vm: &TableDisplayViewModel, ui: &mut Ui) {
        ui.heading("Flight Statistics");
        ui.separator();
        ui.add_space(10.0);

        match vm.statistics {
            Some(Ok(stats)) => {
                egui::Grid::new("statistics_grid")
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Total Flights:");
                        ui.label(stats.total_flights.to_string());
                        ui.end_row();

                        ui.label("Total Distance:");
                        ui.label(format!("{} NM", stats.total_distance));
                        ui.end_row();

                        ui.label("Average Flight Distance:");
                        ui.label(format!("{:.2} NM", stats.average_flight_distance));
                        ui.end_row();

                        ui.label("Most Flown Aircraft:");
                        ui.label(stats.most_flown_aircraft.as_deref().unwrap_or("N/A"));
                        ui.end_row();

                        ui.label("Most Visited Airport:");
                        ui.label(stats.most_visited_airport.as_deref().unwrap_or("N/A"));
                        ui.end_row();

                        ui.label("Favorite Departure Airport:");
                        ui.label(stats.favorite_departure_airport.as_deref().unwrap_or("N/A"));
                        ui.end_row();

                        ui.label("Favorite Arrival Airport:");
                        ui.label(stats.favorite_arrival_airport.as_deref().unwrap_or("N/A"));
                        ui.end_row();

                        ui.label("Longest Flight:");
                        ui.label(stats.longest_flight.as_deref().unwrap_or("N/A"));
                        ui.end_row();

                        ui.label("Shortest Flight:");
                        ui.label(stats.shortest_flight.as_deref().unwrap_or("N/A"));
                        ui.end_row();
                    });
            }
            Some(Err(e)) => {
                ui.label(format!("Error loading statistics: {e}"));
            }
            None => {
                ui.label("Loading statistics...");
                ui.spinner();
            }
        }
    }
}
