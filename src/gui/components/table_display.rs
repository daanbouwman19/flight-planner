use crate::gui::data::list_items::ListItem;
use crate::gui::data::TableItem;
use crate::gui::events::Event;
use crate::gui::state::DisplayMode;
use crate::modules::data_operations::FlightStatistics;
use egui::Ui;
use egui_extras::{Column, TableBuilder};
use std::sync::Arc;

// --- View Model ---

pub struct TableDisplayViewModel<'a> {
    pub items: &'a [Arc<TableItem>],
    pub display_mode: &'a DisplayMode,
    pub is_loading_more_routes: bool,
    pub statistics: &'a Option<Result<FlightStatistics, Box<dyn std::error::Error + Send + Sync>>>,
}

// --- Component ---

pub struct TableDisplay;

impl TableDisplay {
    pub fn render(vm: &TableDisplayViewModel, ui: &mut Ui) -> Vec<Event> {
        let mut events = Vec::new();

        match vm.display_mode {
            DisplayMode::Statistics => {
                Self::render_statistics(vm.statistics, ui);
            }
            _ => {
                Self::render_data_table(vm, ui, &mut events);
            }
        }

        events
    }

    fn render_data_table(vm: &TableDisplayViewModel, ui: &mut Ui, events: &mut Vec<Event>) {
        let table = TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::auto())
            .min_scrolled_height(0.0);

        table
            .header(20.0, |mut header| {
                if let Some(item) = vm.items.first() {
                    let headers = match &**item {
                        TableItem::Aircraft(item) => item.get_headers(),
                        TableItem::Airport(item) => item.get_headers(),
                        TableItem::Route(item) => item.get_headers(),
                        TableItem::History(item) => item.get_headers(),
                    };
                    for title in headers {
                        header.col(|ui| {
                            ui.strong(title);
                        });
                    }
                }
            })
            .body(|mut body| {
                for (index, item) in vm.items.iter().enumerate() {
                    body.row(30.0, |mut row| {
                        let values = match &**item {
                            TableItem::Aircraft(item) => item.get_values(),
                            TableItem::Airport(item) => item.get_values(),
                            TableItem::Route(item) => item.get_values(),
                            TableItem::History(item) => item.get_values(),
                        };
                        for (i, value) in values.iter().enumerate() {
                            row.col(|ui| {
                                if i == 0 {
                                    if ui.link(value).clicked() {
                                        if let TableItem::Route(route) = &**item {
                                            events.push(Event::RouteSelectedForPopup(route.clone()));
                                        }
                                    }
                                } else {
                                    ui.label(value);
                                }
                            });
                        }
                    });
                    if index == vm.items.len() - 1 && vm.is_loading_more_routes {
                        events.push(Event::LoadMoreRoutes);
                    }
                }
            });
    }

    fn render_statistics(
        statistics: &Option<Result<FlightStatistics, Box<dyn std::error::Error + Send + Sync>>>,
        ui: &mut Ui,
    ) {
        ui.heading("Flight Statistics");
        match statistics {
            Some(Ok(stats)) => {
                ui.label(format!("Total flights: {}", stats.total_flights));
                ui.label(format!("Total distance: {:.2} nm", stats.total_distance));
                ui.label(format!(
                    "Average flight distance: {:.2} nm",
                    stats.average_flight_distance
                ));
            }
            Some(Err(e)) => {
                ui.colored_label(egui::Color32::RED, format!("Error: {}", e));
            }
            None => {
                ui.label("Loading statistics...");
            }
        }
    }
}
