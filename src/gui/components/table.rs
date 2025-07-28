use egui::Ui;
use egui_extras::{Column, TableBuilder};

use crate::gui::ui::{Gui, TableItem};

impl Gui<'_> {
    /// Updates the table UI component.
    ///
    /// # Arguments
    ///
    /// * `ui` - The UI context.
    pub fn update_table(&mut self, ui: &mut Ui) {
        // Use all available space for the table
        if let Some(first_item) = self.search_state.filtered_items.first() {
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

    /// Populates the table with data.
    ///
    /// # Arguments
    ///
    /// * `table` - The table builder instance.
    fn populate_table(&mut self, table: TableBuilder) {
        let row_height = 30.0;
        let mut create_more_routes = false;
        let filtered_items = &self.search_state.filtered_items;

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
                                self.popup_state.show_alert = true;
                                self.popup_state.selected_route = Some(route.clone());
                            }
                        });
                    }
                });
            });

        // Load more routes if we've reached the end and it's a route table
        if create_more_routes && self.search_state.query.is_empty() {
            self.load_more_routes_if_needed();
        }
    }
}
