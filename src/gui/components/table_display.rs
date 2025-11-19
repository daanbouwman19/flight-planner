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

// --- View Model ---

/// A view model that provides the necessary data for the `TableDisplay` component.
///
/// This struct aggregates all data required to render the main table view,
/// including the items to display, the current display mode, and loading states.
pub struct TableDisplayViewModel<'a> {
    /// A slice of `TableItem`s to be displayed in the table.
    pub items: &'a [Arc<TableItem>],
    /// The current `DisplayMode`, which determines the table's structure and content.
    pub display_mode: &'a DisplayMode,
    /// A flag indicating whether more routes are currently being loaded for infinite scrolling.
    pub is_loading_more_routes: bool,
    /// The cached flight statistics to be displayed in the statistics view.
    pub statistics: &'a Option<Result<FlightStatistics, Box<dyn Error + Send + Sync>>>,
    /// A lookup function to get flight rules for an airport ICAO code.
    pub flight_rules_lookup: Option<&'a dyn Fn(&str) -> Option<String>>,
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

        let num_columns = match vm.display_mode {
            DisplayMode::RandomRoutes
            | DisplayMode::NotFlownRoutes
            | DisplayMode::SpecificAircraftRoutes => 7,
            DisplayMode::History => 4,
            DisplayMode::Airports | DisplayMode::RandomAirports => 3,
            DisplayMode::Other => 7,
            DisplayMode::Statistics => 0, // Not used
        };

        let table = TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .columns(Column::auto(), num_columns);

        let scroll_area = table
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
                for item in items_to_display {
                    body.row(30.0, |mut row| match item.as_ref() {
                        TableItem::Route(route) => {
                            events.extend(Self::render_route_row(vm, &mut row, route))
                        }
                        TableItem::History(history) => Self::render_history_row(&mut row, history),
                        TableItem::Airport(airport) => Self::render_airport_row(&mut row, airport),
                        TableItem::Aircraft(aircraft) => {
                            events.extend(Self::render_aircraft_row(&mut row, aircraft))
                        }
                    });
                }
                if vm.is_loading_more_routes {
                    body.row(30.0, |mut row| {
                        row.col(|ui| {
                            ui.horizontal(|ui| {
                                ui.spinner();
                                ui.label("Loading more routes...");
                            });
                        });
                        // Fill empty columns to avoid layout shift
                        for _ in 1..num_columns {
                            row.col(|_ui| {});
                        }
                    });
                }
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
            if let Some(lookup) = vm.flight_rules_lookup {
                if let Some(rules) = lookup(&route.departure.ICAO) {
                    ui.label(rules);
                }
            }
        });
        row.col(|ui| {
            ui.label(format!(
                "{} ({})",
                route.destination.Name, route.destination.ICAO
            ));
        });
        row.col(|ui| {
            if let Some(lookup) = vm.flight_rules_lookup {
                if let Some(rules) = lookup(&route.destination.ICAO) {
                    ui.label(rules);
                }
            }
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
