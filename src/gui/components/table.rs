use egui::Ui;
use egui_extras::{Column, TableBuilder};

use crate::gui::data::TableItem;
use crate::gui::ui::Gui;

impl Gui<'_> {
    /// Updates the table UI component with improved encapsulation.
    ///
    /// # Arguments
    ///
    /// * `ui` - The UI context.
    pub fn update_table(&mut self, ui: &mut Ui) {
        // Use all available space for the table
        let filtered_items = self.get_filtered_items();
        if let Some(first_item) = filtered_items.first() {
            let table = Self::build_table(ui, first_item);
            self.populate_table(table);
        } else {
            ui.label("No items to display");
        }
    }

    /// Builds the table UI component.
    ///
    /// # Arguments
    ///
    /// * `ui` - The UI context.
    /// * `first_item` - The first item to determine the table structure.
    fn build_table<'t>(ui: &'t mut Ui, first_item: &TableItem) -> TableBuilder<'t> {
        let mut columns = first_item.get_columns();
        if let TableItem::Route(_) = first_item {
            columns.push("Select");
        }

        let available_height = ui.available_height();
        let mut table = TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .min_scrolled_height(200.0)
            .max_scroll_height(available_height.max(400.0));

        for _ in &columns {
            table = table.column(Column::auto().resizable(true));
        }

        table
    }

    /// Populates the table with data using improved encapsulation.
    ///
    /// # Arguments
    ///
    /// * `table` - The table builder instance.
    fn populate_table(&mut self, table: TableBuilder) {
        let row_height = 30.0;
        let mut create_more_routes = false;

        // Get items before any borrowing to avoid conflicts
        let filtered_items = self.get_filtered_items().to_vec();

        table
            .header(20.0, |mut header| {
                if let Some(first_item) = filtered_items.first() {
                    for name in first_item.get_columns() {
                        header.col(|ui| {
                            ui.label(name.to_string());
                        });
                    }
                    if let TableItem::Route(_) = first_item.as_ref() {
                        header.col(|ui| {
                            ui.label("Actions");
                        });
                    }
                }
            })
            .body(|body| {
                let mut selected_route = None;

                body.rows(row_height, filtered_items.len(), |mut row| {
                    let item = &filtered_items[row.index()];

                    // Display regular columns
                    for name in item.get_data() {
                        row.col(|ui| {
                            ui.label(name.to_string());
                        });
                    }

                    // Handle route-specific columns
                    if let TableItem::Route(route) = item.as_ref() {
                        // Trigger loading more routes when we reach the last visible row
                        if row.index() == filtered_items.len() - 1 {
                            create_more_routes = true;
                        }

                        row.col(|ui| {
                            if ui.button("Select").clicked() {
                                selected_route = Some(route.clone());
                            }
                        });
                    }
                });

                // Handle route selection after the closure
                if let Some(route) = selected_route {
                    self.set_show_alert(true);
                    self.set_selected_route(Some(route));
                }
            });

        // Load more routes if we've reached the end and it's a route table using encapsulated state
        if create_more_routes && self.get_search_query().is_empty() {
            self.load_more_routes_if_needed();
        }
    }
}
