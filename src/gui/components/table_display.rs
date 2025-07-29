use crate::gui::data::TableItem;
use crate::gui::state::popup_state::DisplayMode;
use crate::gui::ui::Gui;
use egui::Ui;

pub struct TableDisplay;

impl TableDisplay {
    /// Renders the main display area with items.
    pub fn render(gui: &mut Gui, ui: &mut Ui) {
        let items_to_display = gui.get_displayed_items().to_vec();

        if items_to_display.is_empty() {
            ui.label("No items to display");
            return;
        }

        // Table display with infinite scrolling detection
        let scroll_area = egui::ScrollArea::vertical().auto_shrink([false, true]);

        let scroll_response = scroll_area.show(ui, |ui| {
            egui::Grid::new("items_grid").striped(true).show(ui, |ui| {
                Self::render_headers(gui, ui);
                Self::render_data_rows(gui, ui, &items_to_display);

                // Show loading indicator if loading
                if gui.is_loading_more_routes() {
                    ui.horizontal(|ui| {
                        ui.label("Loading more routes...");
                        ui.spinner();
                    });
                    ui.end_row();
                }
            });
        });

        // Check for infinite scrolling after rendering
        Self::handle_infinite_scrolling(gui, &scroll_response, &items_to_display);
    }

    /// Handles infinite scrolling detection and loading more items.
    fn handle_infinite_scrolling(
        gui: &mut Gui,
        scroll_response: &egui::scroll_area::ScrollAreaOutput<()>,
        items: &[TableItem],
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

    /// Renders table headers based on display mode.
    fn render_headers(gui: &Gui, ui: &mut Ui) {
        match gui.get_display_mode() {
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

    /// Renders data rows for the table.
    fn render_data_rows(gui: &mut Gui, ui: &mut Ui, items: &[TableItem]) {
        for item in items {
            match item {
                TableItem::Route(route) => {
                    Self::render_route_row(gui, ui, route);
                }
                TableItem::History(history) => {
                    Self::render_history_row(ui, history);
                }
                TableItem::Airport(airport) => {
                    Self::render_airport_row(ui, airport);
                }
                TableItem::Aircraft(aircraft) => {
                    Self::render_aircraft_row(gui, ui, aircraft);
                }
            }
        }
    }

    /// Renders a route row.
    fn render_route_row(gui: &mut Gui, ui: &mut Ui, route: &crate::gui::data::ListItemRoute) {
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
            let display_mode = gui.get_display_mode();
            if matches!(
                display_mode,
                DisplayMode::RandomRoutes
                    | DisplayMode::NotFlownRoutes
                    | DisplayMode::SpecificAircraftRoutes
            ) {
                // Clone the route to avoid borrowing issues
                let route_clone = route.clone();

                // Only show "Select" button - "Mark as flown" is in the popup
                if ui.button("Select").clicked() {
                    gui.set_selected_route(Some(route_clone));
                    gui.set_show_alert(true);
                }
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

    /// Renders an aircraft row.
    fn render_aircraft_row(
        gui: &mut Gui,
        ui: &mut Ui,
        aircraft: &crate::gui::data::ListItemAircraft,
    ) {
        ui.label(&aircraft.manufacturer);
        ui.label(&aircraft.variant);
        ui.label(&aircraft.icao_code);
        ui.label(&aircraft.range);
        ui.label(&aircraft.category);
        ui.label(&aircraft.date_flown);

        // Add toggle button for flown status with fixed width
        ui.horizontal(|ui| {
            let button_text = if aircraft.flown > 0 {
                "Mark Not Flown"
            } else {
                " Mark Flown  "
            };

            if ui
                .add_sized([120.0, 20.0], egui::Button::new(button_text))
                .clicked()
            {
                if let Err(e) = gui.toggle_aircraft_flown_status(aircraft.id) {
                    log::error!("Failed to toggle aircraft flown status: {e}");
                }
            }
        });
        ui.end_row();
    }
}
