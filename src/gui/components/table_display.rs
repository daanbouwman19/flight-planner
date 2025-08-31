use crate::gui::data::TableItem;
use crate::gui::state::popup_state::DisplayMode;
use crate::gui::ui::Gui;
use egui::Ui;
use std::sync::Arc;

pub struct TableDisplay;

impl TableDisplay {
    /// Renders the main display area with items.
    pub fn render(gui: &mut Gui, ui: &mut Ui) {
        // Handle statistics display mode separately
        if *gui.get_display_mode() == DisplayMode::Statistics {
            Self::render_statistics(gui, ui);
            return;
        }

        // Clone items to avoid borrowing issues but implement virtual scrolling
        let items_to_display: Vec<Arc<TableItem>> = gui.get_displayed_items().to_vec();

        if items_to_display.is_empty() {
            ui.label("No items to display");
            return;
        }

        // Store necessary data before the closure
        let is_loading_more_routes = gui.is_loading_more_routes();
        let display_mode = *gui.get_display_mode();

        // Table display with ScrollArea but precise height control
        let row_height = 25.0;
        let items_count = items_to_display.len();
        
        // Calculate exact height needed - be very precise
        let header_height = row_height;
        let loading_height = if is_loading_more_routes { row_height } else { 0.0 };
        
        // For height calculation, limit to a reasonable number of visible rows
        let max_visible_rows = 20; // Show max 20 rows before scrolling
        let visible_data_rows = items_count.min(max_visible_rows);
        let content_height = header_height + (visible_data_rows as f32 * row_height) + loading_height;
        
        // Use exact height - no more, no less
        ui.allocate_ui_with_layout(
            egui::Vec2::new(ui.available_width(), content_height),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                let scroll_area = egui::ScrollArea::vertical()
                    .auto_shrink([false, false]) // Never shrink
                    .max_height(content_height) // Force exact height
                    .id_salt("main_table_scroll");

                let scroll_response = scroll_area.show_rows(ui, row_height, items_to_display.len(), |ui, row_range| {
                    egui::Grid::new("virtual_grid")
                        .striped(true)
                        .min_col_width(0.0)
                        .show(ui, |ui| {
                            // Show header only for the first visible row
                            if row_range.start == 0 {
                                Self::render_headers_for_mode(&display_mode, ui);
                            }
                            
                            // Render only the visible rows
                            for i in row_range.clone() {
                                if i < items_to_display.len() {
                                    Self::render_item_row(&display_mode, ui, &items_to_display[i]);
                                }
                            }
                            
                            // Show loading indicator at the end
                            if is_loading_more_routes && row_range.end >= items_to_display.len() {
                                ui.horizontal(|ui| {
                                    ui.label("Loading more routes...");
                                    ui.spinner();
                                });
                                ui.end_row();
                            }
                        });
                    
                    // Handle infinite scrolling within the virtual scroll
                    if row_range.end >= items_to_display.len().saturating_sub(5) && items_to_display.len() >= 10 {
                        let is_route_mode = matches!(
                            display_mode,
                            DisplayMode::RandomRoutes
                                | DisplayMode::NotFlownRoutes
                                | DisplayMode::SpecificAircraftRoutes
                        );

                        if is_route_mode && !gui.is_loading_more_routes() {
                            gui.load_more_routes_if_needed();
                        }
                    }
                });
                
                scroll_response
            },
        );
        // Check for actions stored by item rendering
        Self::handle_virtual_scroll_actions(gui, ui);
    }

    /// Handles actions stored by item rendering (route selection, aircraft toggle)
    fn handle_virtual_scroll_actions(gui: &mut Gui, ui: &mut Ui) {
        let ctx = ui.ctx();
        
        // Check for route selection
        if let Some(should_show_popup) = ctx.memory(|mem| {
            mem.data.get_temp::<bool>(egui::Id::new("show_route_popup"))
        }) {
            if should_show_popup {
                if let Some(selected_route) = ctx.memory(|mem| {
                    mem.data.get_temp::<crate::gui::data::ListItemRoute>(egui::Id::new("selected_route"))
                }) {
                    gui.set_selected_route(Some(selected_route));
                    gui.set_show_alert(true);
                }
                // Clear the flags
                ctx.memory_mut(|mem| {
                    mem.data.remove::<bool>(egui::Id::new("show_route_popup"));
                    mem.data.remove::<crate::gui::data::ListItemRoute>(egui::Id::new("selected_route"));
                });
            }
        }

        // Check for aircraft toggle
        if let Some(aircraft_id) = ctx.memory(|mem| {
            mem.data.get_temp::<i32>(egui::Id::new("toggle_aircraft_id"))
        }) {
            if let Err(e) = gui.toggle_aircraft_flown_status(aircraft_id) {
                log::error!("Failed to toggle aircraft flown status: {e}");
            }
            // Clear the flag
            ctx.memory_mut(|mem| {
                mem.data.remove::<i32>(egui::Id::new("toggle_aircraft_id"));
            });
        }
    }

    /// Handles infinite scrolling detection and loading more items.
    fn handle_infinite_scrolling(
        gui: &mut Gui,
        scroll_response: &egui::scroll_area::ScrollAreaOutput<()>,
        items: &[Arc<TableItem>],
    ) {
        // Only handle infinite scrolling for route displays
        let display_mode = gui.get_display_mode();
        let is_route_mode = matches!(
            display_mode,
            DisplayMode::RandomRoutes
                | DisplayMode::NotFlownRoutes
                | DisplayMode::SpecificAircraftRoutes
        );

        if !is_route_mode || gui.is_loading_more_routes() {
            return;
        }

        // Check if we've scrolled near the bottom
        let state = &scroll_response.state;
        let content_size = scroll_response.content_size;
        let available_size = scroll_response.inner_rect.size();

        // Calculate how close we are to the bottom
        let scroll_position = state.offset.y;
        let max_scroll = (content_size.y - available_size.y).max(0.0);

        // If we have items and are within 200 pixels of the bottom, load more
        if items.len() >= 10 && max_scroll > 0.0 {
            let scroll_ratio = scroll_position / max_scroll;
            if scroll_ratio > 0.8 {
                // 80% scrolled down
                gui.load_more_routes_if_needed();
            }
        }
    }

    /// Renders table headers based on display mode (static version).
    fn render_headers_for_mode(display_mode: &DisplayMode, ui: &mut Ui) {
        match display_mode {
            DisplayMode::RandomRoutes
            | DisplayMode::NotFlownRoutes
            | DisplayMode::SpecificAircraftRoutes => {
                ui.label("Aircraft");
                ui.label("From");
                ui.label("To");
                ui.label("Distance");
                ui.label("Actions");
                ui.end_row();
            }
            DisplayMode::History => {
                ui.label("Aircraft");
                ui.label("From");
                ui.label("To");
                ui.label("Date Flown");
                ui.end_row();
            }
            DisplayMode::Airports | DisplayMode::RandomAirports => {
                ui.label("ICAO");
                ui.label("Name");
                ui.label("Runway Length");
                ui.end_row();
            }
            DisplayMode::Statistics => {
                // Statistics don't use a traditional table header
            }
            DisplayMode::Other => {
                // Showing aircraft list
                ui.label("Manufacturer");
                ui.label("Variant");
                ui.label("ICAO Code");
                ui.label("Range");
                ui.label("Category");
                ui.label("Date Flown");
                ui.label("Action");
                ui.end_row();
            }
        }
    }

    /// Renders a single item row (static version for virtual scrolling).
    fn render_item_row(display_mode: &DisplayMode, ui: &mut Ui, item: &Arc<TableItem>) {
        match item.as_ref() {
            TableItem::Route(route) => {
                Self::render_route_row_static(display_mode, ui, route);
            }
            TableItem::History(history) => {
                Self::render_history_row(ui, history);
            }
            TableItem::Airport(airport) => {
                Self::render_airport_row(ui, airport);
            }
            TableItem::Aircraft(aircraft) => {
                Self::render_aircraft_row_static(ui, aircraft);
            }
        }
    }

    /// Renders a route row (static version).
    fn render_route_row_static(display_mode: &DisplayMode, ui: &mut Ui, route: &crate::gui::data::ListItemRoute) {
        ui.label(format!(
            "{} {}",
            route.aircraft.manufacturer, route.aircraft.variant
        ));
        ui.label(format!(
            "{} ({})",
            route.departure.Name, route.departure.ICAO
        ));
        ui.label(format!(
            "{} ({})",
            route.destination.Name, route.destination.ICAO
        ));
        ui.label(&route.route_length);

        // Only show actions for route display modes
        ui.horizontal(|ui| {
            if matches!(
                display_mode,
                DisplayMode::RandomRoutes
                    | DisplayMode::NotFlownRoutes
                    | DisplayMode::SpecificAircraftRoutes
            ) {
                if ui.button("Select").clicked() {
                    // Store the route selection for the GUI to pick up later
                    ui.ctx().memory_mut(|mem| {
                        mem.data.insert_temp(egui::Id::new("selected_route"), route.clone());
                        mem.data.insert_temp(egui::Id::new("show_route_popup"), true);
                    });
                }
            }
        });
        ui.end_row();
    }

    /// Renders an aircraft row (static version).
    fn render_aircraft_row_static(
        ui: &mut Ui,
        aircraft: &crate::gui::data::ListItemAircraft,
    ) {
        ui.label(&aircraft.manufacturer);
        ui.label(&aircraft.variant);
        ui.label(&aircraft.icao_code);
        ui.label(&aircraft.range);
        ui.label(&aircraft.category);
        ui.label(&aircraft.date_flown);

        // Add toggle button for flown status
        ui.horizontal(|ui| {
            let button_text = if aircraft.flown > 0 {
                "Mark Not Flown"
            } else {
                " Mark Flown  "
            };

            if ui.button(button_text).clicked() {
                // Store the aircraft toggle for the GUI to pick up later
                ui.ctx().memory_mut(|mem| {
                    mem.data.insert_temp(egui::Id::new("toggle_aircraft_id"), aircraft.id);
                });
            }
        });
        ui.end_row();
    }

    /// Renders a history row.
    fn render_history_row(ui: &mut Ui, history: &crate::gui::data::ListItemHistory) {
        ui.label(&history.aircraft_name);
        ui.label(&history.departure_icao);
        ui.label(&history.arrival_icao);
        ui.label(&history.date);
        ui.end_row();
    }

    /// Renders an airport row.
    fn render_airport_row(ui: &mut Ui, airport: &crate::gui::data::ListItemAirport) {
        ui.label(&airport.icao);
        ui.label(&airport.name);
        ui.label(&airport.longest_runway_length);
        ui.end_row();
    }

    /// Renders the statistics display.
    fn render_statistics(gui: &mut Gui, ui: &mut Ui) {
        ui.heading("Flight Statistics");
        ui.separator();
        ui.add_space(10.0);

        match gui.get_flight_statistics() {
            Ok(stats) => {
                egui::Grid::new("statistics_grid")
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Total Flights:");
                        ui.label(stats.total_flights.to_string());
                        ui.end_row();

                        ui.label("Total Distance:");
                        ui.label(format!("{} NM", stats.total_distance));
                        ui.end_row();

                        ui.label("Most Flown Aircraft:");
                        ui.label(stats.most_flown_aircraft.as_deref().unwrap_or("None"));
                        ui.end_row();

                        ui.label("Most Visited Airport:");
                        ui.label(stats.most_visited_airport.as_deref().unwrap_or("None"));

                        ui.end_row();
                    });
            }
            Err(e) => {
                ui.label(format!("Error loading statistics: {}", e));
            }
        }
    }
}
