use crate::gui::data::{
    ListItemAircraft, ListItemAirport, ListItemHistory, ListItemRoute, TableItem,
};
use crate::gui::events::Event;
use crate::gui::services::popup_service::DisplayMode;
use crate::models::weather::FlightRules;
use crate::modules::data_operations::FlightStatistics;
use egui::{CursorIcon, Sense, Ui};
use egui_extras::{Column, TableBuilder, TableRow};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use std::time::{Duration, Instant};

// UI Constants
const DISTANCE_FROM_BOTTOM_TO_LOAD_MORE: f32 = 200.0;
const MIN_ITEMS_FOR_LAZY_LOAD: usize = 10;
const ROW_HEIGHT: f32 = 30.0;

// Column Width Constants
const RULES_COL_WIDTH: f32 = 80.0;
const ACTIONS_COL_WIDTH: f32 = 100.0;
const AIRCRAFT_ACTIONS_COL_WIDTH: f32 = 150.0;
const DISTANCE_COL_WIDTH: f32 = 80.0;
const ICAO_COL_WIDTH: f32 = 60.0;
const DATE_COL_WIDTH: f32 = 120.0;
const RUNWAY_COL_WIDTH: f32 = 120.0;
const RANGE_COL_WIDTH: f32 = 80.0;
const CATEGORY_COL_WIDTH: f32 = 100.0;

/// Type alias for a function that looks up flight rules for a given ICAO code
pub type FlightRulesLookup<'a> = &'a dyn Fn(&str) -> Option<(FlightRules, Instant)>;

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
    /// Stores the relative column widths (as ratios 0.0-1.0) for each display mode.
    pub column_widths: &'a HashMap<DisplayMode, Vec<f32>>,
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
    /// * `events` - A mutable reference to the event buffer.
    pub fn render(vm: &TableDisplayViewModel, ui: &mut Ui, events: &mut Vec<Event>) {
        if *vm.display_mode == DisplayMode::Statistics {
            Self::render_statistics(vm, ui);
            return;
        }

        let items_to_display = vm.items;

        if items_to_display.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                if vm.is_loading_more_routes {
                    ui.spinner();
                    ui.label("Generating routes...");
                    return;
                }
                match vm.display_mode {
                    DisplayMode::RandomRoutes
                    | DisplayMode::NotFlownRoutes
                    | DisplayMode::SpecificAircraftRoutes => {
                        ui.heading("✈️ No routes generated yet");
                        ui.label("Use the 'Actions' panel on the left to generate routes.");
                    }
                    DisplayMode::History => {
                        ui.heading("📜 No flight history found");
                        if ui
                            .button("➕ Add flight manually")
                            .on_hover_text("Open the manual flight entry form")
                            .clicked()
                        {
                            events.push(Event::ShowAddHistoryPopup);
                        }
                    }
                    DisplayMode::Airports | DisplayMode::RandomAirports | DisplayMode::Other => {
                        ui.heading("🔍 No items found");
                        ui.label("Try adjusting your search criteria.");
                    }
                    _ => {
                        ui.label("No items to display");
                    }
                }
            });
            return;
        }

        let available_width = ui.available_width();
        let ctx = ui.ctx().clone();

        let mut builder = TableBuilder::new(ui)
            .striped(true)
            .resizable(false) // We handle resizing manually for relative widths
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center));

        // Get or calculate column widths
        let mut width_buffer = [0.0; 8];
        let widths: &[f32] = if let Some(ratios) = vm.column_widths.get(vm.display_mode) {
            let count = ratios.len().min(width_buffer.len());
            for (i, r) in ratios.iter().take(count).enumerate() {
                width_buffer[i] = r * available_width;
            }
            &width_buffer[..count]
        } else {
            let count = Self::calculate_default_widths_into(
                vm.display_mode,
                available_width,
                &mut width_buffer,
            );
            &width_buffer[..count]
        };

        let headers = Self::get_headers(vm.display_mode);
        let num_columns = headers.len();

        for width in widths {
            builder = builder.column(Column::exact(*width).resizable(false));
        }

        let scroll_area = builder
            .header(20.0, |mut header| {
                for (i, title) in headers.iter().enumerate() {
                    header.col(|ui| {
                        // Title
                        let tooltip = Self::get_header_tooltip(title);
                        let response = ui.strong(*title);

                        if !tooltip.is_empty() {
                            response.on_hover_text(tooltip);
                        }

                        // Resize Handle (skip for last column)
                        if i < num_columns - 1 {
                            let rect = ui.max_rect();
                            let handle_rect = egui::Rect::from_min_size(
                                egui::pos2(rect.right() - 4.0, rect.top()),
                                egui::vec2(8.0, rect.height()),
                            );

                            let response = ui.allocate_rect(handle_rect, Sense::drag());
                            if response.hovered() || response.dragged() {
                                ui.output_mut(|o| o.cursor_icon = CursorIcon::ResizeColumn);
                            }

                            if response.dragged() {
                                events.push(Event::ColumnResized {
                                    mode: vm.display_mode.clone(),
                                    index: i,
                                    delta: response.drag_delta().x,
                                    total_width: available_width,
                                });
                            }
                        }
                    });
                }
            })
            .body(|body| {
                let count = items_to_display.len() + if vm.is_loading_more_routes { 1 } else { 0 };

                body.rows(ROW_HEIGHT, count, |mut row| {
                    let index = row.index();
                    if index < items_to_display.len() {
                        let item = &items_to_display[index];
                        match item.as_ref() {
                            TableItem::Route(route) => {
                                Self::render_route_row(vm, &mut row, route, events, &ctx)
                            }
                            TableItem::History(history) => {
                                Self::render_history_row(&mut row, history)
                            }
                            TableItem::Airport(airport) => {
                                Self::render_airport_row(&mut row, airport)
                            }
                            TableItem::Aircraft(aircraft) => {
                                Self::render_aircraft_row(&mut row, aircraft, events)
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
                        // Fill empty columns to avoid layout shift, using the pre-calculated num_columns
                        for _ in 1..num_columns {
                            row.col(|_ui| {});
                        }
                    }
                });
            });

        if let Some(event) = Self::handle_infinite_scrolling(vm, &scroll_area, items_to_display) {
            events.push(event);
        }
    }

    fn calculate_default_widths(mode: &DisplayMode, available_width: f32) -> Vec<f32> {
        let mut buffer = [0.0; 8];
        let count = Self::calculate_default_widths_into(mode, available_width, &mut buffer);
        buffer[..count].to_vec()
    }

    fn calculate_default_widths_into(
        mode: &DisplayMode,
        available_width: f32,
        out: &mut [f32],
    ) -> usize {
        match mode {
            DisplayMode::RandomRoutes
            | DisplayMode::NotFlownRoutes
            | DisplayMode::SpecificAircraftRoutes => {
                // Fixed columns: Dep Rules (80), Dest Rules (80), Distance (80), Actions (100) -> Total 340
                let fixed_width = RULES_COL_WIDTH * 2.0 + DISTANCE_COL_WIDTH + ACTIONS_COL_WIDTH;
                let flex_width = (available_width - fixed_width).max(0.0);
                let col_width = flex_width / 3.0;
                let widths = [
                    col_width,          // Aircraft
                    col_width,          // From
                    RULES_COL_WIDTH,    // Dep Rules
                    col_width,          // To
                    RULES_COL_WIDTH,    // Dest Rules
                    DISTANCE_COL_WIDTH, // Distance
                    ACTIONS_COL_WIDTH,  // Actions
                ];
                let count = widths.len();
                if count <= out.len() {
                    out[..count].copy_from_slice(&widths);
                    count
                } else {
                    0
                }
            }
            DisplayMode::History => {
                // Fixed columns: Date Flown (120) -> Total 120
                let fixed_width = DATE_COL_WIDTH;
                let flex_width = (available_width - fixed_width).max(0.0);
                let col_width = flex_width / 3.0;
                let widths = [
                    col_width,      // Aircraft
                    col_width,      // From
                    col_width,      // To
                    DATE_COL_WIDTH, // Date Flown
                ];
                let count = widths.len();
                if count <= out.len() {
                    out[..count].copy_from_slice(&widths);
                    count
                } else {
                    0
                }
            }
            DisplayMode::Airports | DisplayMode::RandomAirports => {
                // Fixed: ICAO (60), Runway Length (120) -> Total 180
                let fixed_width = ICAO_COL_WIDTH + RUNWAY_COL_WIDTH;
                let flex_width = (available_width - fixed_width).max(0.0);
                let widths = [
                    ICAO_COL_WIDTH,   // ICAO
                    flex_width,       // Name
                    RUNWAY_COL_WIDTH, // Runway Length
                ];
                let count = widths.len();
                if count <= out.len() {
                    out[..count].copy_from_slice(&widths);
                    count
                } else {
                    0
                }
            }
            DisplayMode::Other => {
                // Fixed: ICAO (60), Range (80), Category (100), Date Flown (120), Action (150) -> Total 510
                let fixed_width = ICAO_COL_WIDTH
                    + RANGE_COL_WIDTH
                    + CATEGORY_COL_WIDTH
                    + DATE_COL_WIDTH
                    + AIRCRAFT_ACTIONS_COL_WIDTH;
                let flex_width = (available_width - fixed_width).max(0.0);
                let col_width = flex_width / 2.0;

                let widths = [
                    col_width,                  // Manufacturer
                    col_width,                  // Variant
                    ICAO_COL_WIDTH,             // ICAO Code
                    RANGE_COL_WIDTH,            // Range
                    CATEGORY_COL_WIDTH,         // Category
                    DATE_COL_WIDTH,             // Date Flown
                    AIRCRAFT_ACTIONS_COL_WIDTH, // Action
                ];
                let count = widths.len();
                if count <= out.len() {
                    out[..count].copy_from_slice(&widths);
                    count
                } else {
                    0
                }
            }
            DisplayMode::Statistics => 0,
        }
    }

    fn get_headers(mode: &DisplayMode) -> &'static [&'static str] {
        match mode {
            DisplayMode::RandomRoutes
            | DisplayMode::NotFlownRoutes
            | DisplayMode::SpecificAircraftRoutes => &[
                "Aircraft",
                "From",
                "Dep Rules",
                "To",
                "Dest Rules",
                "Distance",
                "Actions",
            ],
            DisplayMode::History => &["Aircraft", "From", "To", "Date Flown"],
            DisplayMode::Airports | DisplayMode::RandomAirports => {
                &["ICAO", "Name", "Runway Length"]
            }
            DisplayMode::Other => &[
                "Manufacturer",
                "Variant",
                "ICAO Code",
                "Range",
                "Category",
                "Date Flown",
                "Action",
            ],
            DisplayMode::Statistics => &[],
        }
    }

    fn get_header_tooltip(header: &str) -> &str {
        match header {
            "Aircraft" => "The aircraft model used for the route",
            "From" => "Departure airport",
            "Dep Rules" => "Departure flight rules (based on current METAR)",
            "To" => "Destination airport",
            "Dest Rules" => "Destination flight rules (based on current METAR)",
            "Distance" => "Direct distance between airports (nautical miles)",
            "Actions" => "Available actions for this item",
            "Date Flown" => "The date this flight was recorded in history",
            "ICAO" => "International Civil Aviation Organization airport code",
            "Name" => "Airport name",
            "Runway Length" => "Length of the longest runway (ft)",
            "Manufacturer" => "Aircraft manufacturer",
            "Variant" => "Aircraft model variant",
            "ICAO Code" => "Aircraft type designator",
            "Range" => "Maximum flight range (nautical miles)",
            "Category" => "Aircraft category (e.g., Landplane, Seaplane)",
            "Action" => "Manage fleet status",
            _ => "",
        }
    }

    // Exposing this helper so existing logic or state initialization can reuse it
    pub fn get_default_widths(mode: &DisplayMode, available_width: f32) -> Vec<f32> {
        Self::calculate_default_widths(mode, available_width)
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

    fn render_flight_rules_cell_with_opacity(
        ui: &mut egui::Ui,
        icao: &str,
        lookup: Option<FlightRulesLookup>,
        row_opacity: f32,
    ) {
        if let Some(lookup) = lookup
            && let Some((rules, fetched_at)) = lookup(icao)
        {
            let mut color = crate::gui::styles::get_flight_rules_color(&rules, ui.visuals());

            // Fade-in animation based on fetch time
            let elapsed = Instant::now().duration_since(fetched_at);
            let fade_duration = Duration::from_millis(500);
            let mut fetch_opacity = 1.0;

            if elapsed < fade_duration {
                let t = elapsed.as_secs_f32() / fade_duration.as_secs_f32();
                fetch_opacity = t.clamp(0.0, 1.0);
                // Request repaint to animate the fade-in
                ui.ctx().request_repaint();
            }

            // Combine row opacity (creation time) with fetch opacity (network time)
            let final_opacity = row_opacity * fetch_opacity;
            color = color.linear_multiply(final_opacity);

            let tooltip = rules.description();

            ui.label(egui::RichText::new(rules.as_str()).color(color))
                .on_hover_text(tooltip);
        }
    }

    fn render_route_row(
        vm: &TableDisplayViewModel,
        row: &mut TableRow,
        route: &ListItemRoute,
        events: &mut Vec<Event>,
        ctx: &egui::Context,
    ) {
        // Calculate opacity for fade-in animation
        let now = Instant::now();
        let (elapsed, is_future) = if route.created_at > now {
            (Duration::ZERO, true)
        } else {
            (now.duration_since(route.created_at), false)
        };

        let fade_duration = Duration::from_millis(500);
        let mut opacity_multiplier = 1.0;

        if is_future {
            opacity_multiplier = 0.0;
            ctx.request_repaint();
        } else if elapsed < fade_duration {
            let t = elapsed.as_secs_f32() / fade_duration.as_secs_f32();
            opacity_multiplier = t.clamp(0.0, 1.0);
            ctx.request_repaint();
        }

        // Helper to apply opacity to standard text labels
        let label_with_opacity = |ui: &mut Ui, text: &str| {
            let color = ui.visuals().text_color();
            let faded_color = color.linear_multiply(opacity_multiplier);
            ui.label(egui::RichText::new(text).color(faded_color));
        };

        row.col(|ui| {
            label_with_opacity(ui, route.aircraft_info.as_str());
        });
        row.col(|ui| {
            label_with_opacity(ui, route.departure_info.as_str());
        });
        row.col(|ui| {
            Self::render_flight_rules_cell_with_opacity(
                ui,
                &route.departure.ICAO,
                vm.flight_rules_lookup,
                opacity_multiplier,
            );
        });
        row.col(|ui| {
            label_with_opacity(ui, route.destination_info.as_str());
        });
        row.col(|ui| {
            Self::render_flight_rules_cell_with_opacity(
                ui,
                &route.destination.ICAO,
                vm.flight_rules_lookup,
                opacity_multiplier,
            );
        });
        row.col(|ui| {
            label_with_opacity(ui, &route.distance_str);
        });

        row.col(|ui| {
            if matches!(
                vm.display_mode,
                DisplayMode::RandomRoutes
                    | DisplayMode::NotFlownRoutes
                    | DisplayMode::SpecificAircraftRoutes
            ) && ui
                .button("Select")
                .on_hover_text("View details and options for this route")
                .clicked()
            {
                events.push(Event::RouteSelectedForPopup(route.clone()));
                events.push(Event::SetShowPopup(true));
            }
        });
    }

    fn render_history_row(row: &mut TableRow, history: &ListItemHistory) {
        row.col(|ui| {
            ui.label(&history.aircraft_name);
        });
        row.col(|ui| {
            ui.label(&history.departure_info);
        });
        row.col(|ui| {
            ui.label(&history.arrival_info);
        });
        row.col(|ui| {
            ui.label(&history.date);
        });
    }

    fn render_airport_row(row: &mut TableRow, airport: &ListItemAirport) {
        row.col(|ui| {
            if ui
                .add(
                    egui::Label::new(egui::RichText::new(&airport.icao).monospace())
                        .sense(Sense::click()),
                )
                .on_hover_cursor(CursorIcon::PointingHand)
                .on_hover_text("Click to copy ICAO code")
                .clicked()
            {
                ui.output_mut(|o| {
                    o.commands
                        .push(egui::OutputCommand::CopyText(airport.icao.clone()))
                });
            }
        });
        row.col(|ui| {
            ui.label(&airport.name);
        });
        row.col(|ui| {
            ui.label(&airport.longest_runway_length);
        });
    }

    fn render_aircraft_row(
        row: &mut TableRow,
        aircraft: &ListItemAircraft,
        events: &mut Vec<Event>,
    ) {
        row.col(|ui| {
            ui.label(&aircraft.manufacturer);
        });
        row.col(|ui| {
            ui.label(&aircraft.variant);
        });
        row.col(|ui| {
            if ui
                .add(
                    egui::Label::new(egui::RichText::new(&aircraft.icao_code).monospace())
                        .sense(Sense::click()),
                )
                .on_hover_cursor(CursorIcon::PointingHand)
                .on_hover_text("Click to copy ICAO code")
                .clicked()
            {
                ui.output_mut(|o| {
                    o.commands
                        .push(egui::OutputCommand::CopyText(aircraft.icao_code.clone()))
                });
            }
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
            let (button_text, tooltip) = if aircraft.flown > 0 {
                ("Mark Not Flown", "Reset flown status for this aircraft")
            } else {
                (
                    " Mark Flown  ",
                    "Mark this aircraft model as flown in your personal fleet",
                )
            };

            if ui
                .add_sized(AIRCRAFT_ACTION_BUTTON_SIZE, egui::Button::new(button_text))
                .on_hover_text(tooltip)
                .clicked()
            {
                events.push(Event::ToggleAircraftFlownStatus(aircraft.id));
            }
        });
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_tooltips_content() {
        // Verify that critical headers have non-empty tooltips
        let critical_headers = ["ICAO", "Dep Rules", "Category", "Distance"];
        for header in critical_headers {
            let tooltip = TableDisplay::get_header_tooltip(header);
            assert!(
                !tooltip.is_empty(),
                "Tooltip for '{}' should not be empty",
                header
            );
        }
    }
}
