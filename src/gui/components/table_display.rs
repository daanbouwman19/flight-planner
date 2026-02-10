use crate::gui::data::{
    ListItemAircraft, ListItemAirport, ListItemHistory, ListItemRoute, TableItem,
};
use crate::gui::events::{AppEvent, DataEvent, UiEvent};
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
const HISTORY_ACTIONS_COL_WIDTH: f32 = 80.0;
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
    /// { "mode": [0.1, 0.2, ...] }
    pub column_widths: &'a HashMap<DisplayMode, Vec<f32>>,
    /// Indicates if there is an active search query.
    pub has_active_search: bool,
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
    #[cfg(not(tarpaulin_include))]
    pub fn render(
        vm: &TableDisplayViewModel,
        scroll_to_top: &mut bool,
        ui: &mut Ui,
        events: &mut Vec<AppEvent>,
    ) {
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
                        ui.add_space(5.0);
                        if ui
                            .button("Generate Random Route")
                            .on_hover_text("Generate a random route")
                            .clicked()
                        {
                            events.push(AppEvent::Ui(UiEvent::SetDisplayMode(
                                DisplayMode::RandomRoutes,
                            )));
                            events.push(AppEvent::Data(
                                DataEvent::RegenerateRoutesForSelectionChange,
                            ));
                        }
                    }
                    DisplayMode::History => {
                        ui.heading("📜 No flight history found");
                        if ui
                            .button("➕ Add flight manually")
                            .on_hover_text("Open the manual flight entry form")
                            .clicked()
                        {
                            events.push(AppEvent::Ui(UiEvent::ShowAddHistoryPopup));
                        }
                    }
                    DisplayMode::Airports | DisplayMode::RandomAirports => {
                        if vm.has_active_search {
                            Self::render_no_search_results(ui, events);
                        } else {
                            ui.heading("⚠️ No airports found");
                            ui.label(
                                "Your airports database appears to be empty or missing.",
                            );
                            ui.label(
                                "Please ensure 'airports.db3' is present in your data folder.",
                            );
                        }
                    }
                    DisplayMode::Other => {
                        if vm.has_active_search {
                            Self::render_no_search_results(ui, events);
                        } else {
                            ui.heading("✈️ No aircraft found");
                            ui.label("Import an 'aircrafts.csv' file or ensure your database is populated.");
                        }
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
        let now = Instant::now();

        let mut builder = TableBuilder::new(ui)
            .striped(true)
            .resizable(false) // We handle resizing manually for relative widths
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center));

        if *scroll_to_top {
            builder = builder.scroll_to_row(0, Some(egui::Align::TOP));
            *scroll_to_top = false;
        }

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
                                events.push(AppEvent::Ui(UiEvent::ColumnResized {
                                    mode: vm.display_mode.clone(),
                                    index: i,
                                    delta: response.drag_delta().x,
                                    total_width: available_width,
                                }));
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
                                Self::render_route_row(vm, &mut row, route, events, &ctx, now)
                            }
                            TableItem::History(history) => {
                                Self::render_history_row(&mut row, history, events)
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

    /// Handles logic for resizing columns.
    pub fn handle_column_resize(
        column_widths: &mut HashMap<DisplayMode, Vec<f32>>,
        mode: DisplayMode,
        index: usize,
        delta: f32,
        total_width: f32,
    ) {
        let total_width = total_width.max(1.0);
        let ratios = column_widths.entry(mode.clone()).or_insert_with(|| {
            // Initialize with default relative widths if not present
            let defaults = Self::get_default_widths(&mode, total_width);
            defaults.iter().map(|&w| w / total_width).collect()
        });

        let num_columns = ratios.len();
        if index < num_columns {
            let delta_ratio = delta / total_width;

            // The resize handle is not on the last column, so index is always < num_columns - 1.
            if index + 1 < num_columns {
                const MIN_RATIO: f32 = 0.02;

                // Clamp the delta to prevent columns from becoming too small.
                let min_delta = MIN_RATIO - ratios[index];
                let max_delta = ratios[index + 1] - MIN_RATIO;
                let clamped_delta = delta_ratio.clamp(min_delta, max_delta);

                ratios[index] += clamped_delta;
                ratios[index + 1] -= clamped_delta;
            }

            // Re-normalize to correct any floating point drift.
            let sum: f32 = ratios.iter().sum();
            if (sum - 1.0).abs() > 1e-6 {
                for r in ratios.iter_mut() {
                    *r /= sum;
                }
            }
        }
    }

    fn calculate_default_widths(mode: &DisplayMode, available_width: f32) -> Vec<f32> {
        let mut buffer = [0.0; 8];
        let count = Self::calculate_default_widths_into(mode, available_width, &mut buffer);
        buffer[..count].to_vec()
    }

    pub fn calculate_default_widths_into(
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
                // Fixed columns: Date Flown (120), Actions (80) -> Total 200
                let fixed_width = DATE_COL_WIDTH + HISTORY_ACTIONS_COL_WIDTH;
                let flex_width = (available_width - fixed_width).max(0.0);
                let col_width = flex_width / 3.0;
                let widths = [
                    col_width,                 // Aircraft
                    col_width,                 // From
                    col_width,                 // To
                    DATE_COL_WIDTH,            // Date Flown
                    HISTORY_ACTIONS_COL_WIDTH, // Actions
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
            DisplayMode::History => &["Aircraft", "From", "To", "Date Flown", "Actions"],
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

    #[cfg(not(tarpaulin_include))]
    pub fn handle_infinite_scrolling(
        vm: &TableDisplayViewModel,
        scroll_response: &egui::scroll_area::ScrollAreaOutput<()>,
        items: &[Arc<TableItem>],
    ) -> Option<AppEvent> {
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

        if Self::should_load_more_routes(
            items.len(),
            scroll_position,
            content_size.y,
            available_size.y,
        ) {
            return Some(AppEvent::Data(DataEvent::LoadMoreRoutes));
        }
        None
    }

    /// Pure logic helper for infinite scrolling
    pub fn should_load_more_routes(
        item_count: usize,
        scroll_position: f32,
        content_height: f32,
        viewport_height: f32,
    ) -> bool {
        let max_scroll = (content_height - viewport_height).max(0.0);

        item_count >= MIN_ITEMS_FOR_LAZY_LOAD
            && max_scroll > 0.0
            && max_scroll - scroll_position < DISTANCE_FROM_BOTTOM_TO_LOAD_MORE
    }

    #[cfg(not(tarpaulin_include))]
    fn render_no_search_results(ui: &mut Ui, events: &mut Vec<AppEvent>) {
        ui.heading("🔍 No results found");
        ui.label("No items matched your search.");
        ui.add_space(5.0);
        if ui
            .button(egui::RichText::new(format!(
                "{} Clear Search",
                icons::ICON_CLOSE
            )))
            .on_hover_text("Clear the current search filter")
            .clicked()
        {
            events.push(AppEvent::Ui(UiEvent::ClearSearch));
        }
    }

    #[cfg(not(tarpaulin_include))]
    fn render_flight_rules_cell_with_opacity(
        ui: &mut egui::Ui,
        icao: &str,
        lookup: Option<FlightRulesLookup>,
        row_opacity: f32,
        now: Instant,
    ) {
        if let Some(lookup) = lookup
            && let Some((rules, fetched_at)) = lookup(icao)
        {
            let mut color = crate::gui::styles::get_flight_rules_color(&rules, ui.visuals());

            // Fade-in animation based on fetch time
            let elapsed = now.duration_since(fetched_at);
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
        } else {
            let color = ui.visuals().weak_text_color();
            let faded_color = color.linear_multiply(row_opacity);
            ui.label(egui::RichText::new("...").color(faded_color))
                .on_hover_text("Fetching weather data...");
        }
    }

    #[cfg(not(tarpaulin_include))]
    fn render_route_row(
        vm: &TableDisplayViewModel,
        row: &mut TableRow,
        route: &ListItemRoute,
        events: &mut Vec<AppEvent>,
        ctx: &egui::Context,
        now: Instant,
    ) {
        // Calculate opacity for fade-in animation
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
            let color = ui.visuals().text_color();
            let faded_color = color.linear_multiply(opacity_multiplier);
            crate::gui::components::common::render_copyable_label_with_color(
                ui,
                route.departure_info.as_str(),
                &route.departure.ICAO,
                "Click to copy ICAO code",
                faded_color,
            );
        });
        row.col(|ui| {
            Self::render_flight_rules_cell_with_opacity(
                ui,
                &route.departure.ICAO,
                vm.flight_rules_lookup,
                opacity_multiplier,
                now,
            );
        });
        row.col(|ui| {
            let color = ui.visuals().text_color();
            let faded_color = color.linear_multiply(opacity_multiplier);
            crate::gui::components::common::render_copyable_label_with_color(
                ui,
                route.destination_info.as_str(),
                &route.destination.ICAO,
                "Click to copy ICAO code",
                faded_color,
            );
        });
        row.col(|ui| {
            Self::render_flight_rules_cell_with_opacity(
                ui,
                &route.destination.ICAO,
                vm.flight_rules_lookup,
                opacity_multiplier,
                now,
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
                .on_hover_text(format!(
                    "View details for route {} -> {}",
                    route.departure.ICAO, route.destination.ICAO
                ))
                .clicked()
            {
                events.push(AppEvent::Data(DataEvent::RouteSelectedForPopup(
                    route.clone(),
                )));
                events.push(AppEvent::Ui(UiEvent::SetShowPopup(true)));
            }
        });
    }

    #[cfg(not(tarpaulin_include))]
    fn render_history_row(
        row: &mut TableRow,
        history: &ListItemHistory,
        events: &mut Vec<AppEvent>,
    ) {
        row.col(|ui| {
            ui.label(&history.aircraft_name);
        });
        row.col(|ui| {
            crate::gui::components::common::render_copyable_label(
                ui,
                &history.departure_info,
                &history.departure_icao,
                "Click to copy ICAO code",
                false,
            );
        });
        row.col(|ui| {
            crate::gui::components::common::render_copyable_label(
                ui,
                &history.arrival_info,
                &history.arrival_icao,
                "Click to copy ICAO code",
                false,
            );
        });
        row.col(|ui| {
            ui.label(&history.date);
        });
        row.col(|ui| {
            if ui
                .button("Select")
                .on_hover_text(format!(
                    "View details for flight {} -> {} on {}",
                    history.departure_icao, history.arrival_icao, history.date
                ))
                .clicked()
            {
                events.push(AppEvent::Data(DataEvent::HistoryItemSelected(
                    history.clone(),
                )));
            }
        });
    }

    #[cfg(not(tarpaulin_include))]
    fn render_airport_row(row: &mut TableRow, airport: &ListItemAirport) {
        row.col(|ui| {
            crate::gui::components::common::render_copyable_label(
                ui,
                &airport.icao,
                &airport.icao,
                "Click to copy ICAO code",
                true,
            );
        });
        row.col(|ui| {
            ui.label(&airport.name);
        });
        row.col(|ui| {
            ui.label(&airport.longest_runway_length);
        });
    }

    #[cfg(not(tarpaulin_include))]
    fn render_aircraft_row(
        row: &mut TableRow,
        aircraft: &ListItemAircraft,
        events: &mut Vec<AppEvent>,
    ) {
        row.col(|ui| {
            ui.label(&aircraft.manufacturer);
        });
        row.col(|ui| {
            ui.label(&aircraft.variant);
        });
        row.col(|ui| {
            crate::gui::components::common::render_copyable_label(
                ui,
                &aircraft.icao_code,
                &aircraft.icao_code,
                "Click to copy ICAO code",
                true,
            );
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
            let aircraft_name = format!("{} {}", aircraft.manufacturer, aircraft.variant);
            let (button_text, tooltip) = if aircraft.flown > 0 {
                (
                    "Mark Not Flown",
                    format!("Reset flown status for {}", aircraft_name),
                )
            } else {
                (
                    " Mark Flown  ",
                    format!("Mark {} as flown in your personal fleet", aircraft_name),
                )
            };

            if ui
                .add_sized(AIRCRAFT_ACTION_BUTTON_SIZE, egui::Button::new(button_text))
                .on_hover_text(tooltip)
                .clicked()
            {
                events.push(AppEvent::Data(DataEvent::ToggleAircraftFlownStatus(
                    aircraft.id,
                )));
            }
        });
    }

    #[cfg(not(tarpaulin_include))]
    fn render_statistics(vm: &TableDisplayViewModel, ui: &mut Ui) {
        ui.heading("Flight Statistics");
        ui.separator();
        ui.add_space(10.0);

        match vm.statistics {
            Some(Ok(stats)) => {
                let stat_row =
                    |ui: &mut Ui, label: &str, value: String, tooltip: &str, icon: &str| {
                        ui.label(format!("{} {}", icon, label));
                        ui.label(value).on_hover_text(tooltip);
                        ui.end_row();
                    };

                let get_optional_stat =
                    |stat: &Option<String>| stat.as_deref().unwrap_or("N/A").to_string();

                ui.heading("📊 General Activity");
                egui::Grid::new("stats_general")
                    .striped(true)
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .show(ui, |ui| {
                        stat_row(
                            ui,
                            "Total Flights",
                            stats.total_flights.to_string(),
                            "Total number of flights completed",
                            "✈️",
                        );
                        stat_row(
                            ui,
                            "Total Distance",
                            format!("{} NM", stats.total_distance),
                            "Cumulative distance flown across all flights",
                            "🌍",
                        );
                        stat_row(
                            ui,
                            "Average Flight",
                            format!("{:.1} NM", stats.average_flight_distance),
                            "Average distance per flight",
                            "📏",
                        );
                    });

                ui.add_space(20.0);
                ui.heading("🛩️ Fleet & Airports");
                egui::Grid::new("stats_fleet")
                    .striped(true)
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .show(ui, |ui| {
                        stat_row(
                            ui,
                            "Most Flown Aircraft",
                            get_optional_stat(&stats.most_flown_aircraft),
                            "The aircraft you have used most frequently",
                            "👨‍✈️",
                        );
                        stat_row(
                            ui,
                            "Most Visited",
                            get_optional_stat(&stats.most_visited_airport),
                            "Airport with the most takeoffs and landings combined",
                            "📍",
                        );
                        stat_row(
                            ui,
                            "Fav. Departure",
                            get_optional_stat(&stats.favorite_departure_airport),
                            "Airport you depart from most often",
                            "🛫",
                        );
                        stat_row(
                            ui,
                            "Fav. Arrival",
                            get_optional_stat(&stats.favorite_arrival_airport),
                            "Airport you arrive at most often",
                            "🛬",
                        );
                    });

                ui.add_space(20.0);
                ui.heading("🏆 Records");
                egui::Grid::new("stats_records")
                    .striped(true)
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .show(ui, |ui| {
                        stat_row(
                            ui,
                            "Longest Flight",
                            get_optional_stat(&stats.longest_flight),
                            "The single longest flight by distance",
                            "📈",
                        );
                        stat_row(
                            ui,
                            "Shortest Flight",
                            get_optional_stat(&stats.shortest_flight),
                            "The single shortest flight by distance",
                            "📉",
                        );
                    });
            }
            Some(Err(e)) => {
                ui.label(
                    egui::RichText::new(format!("Error loading statistics: {e}"))
                        .color(ui.visuals().error_fg_color),
                );
            }
            None => {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label("Loading statistics...");
                });
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
