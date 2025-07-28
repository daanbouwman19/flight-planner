use crate::gui::ui::{Gui, TableItem};
use eframe::egui;
use egui_extras::{Column, TableBuilder};

impl Gui<'_> {
    /// Updates the table UI component.
    ///
    /// # Arguments
    ///
    /// * `ui` - The UI context.
    pub(super) fn update_table(&mut self, ui: &mut egui::Ui) {
        // Use all available space for the table
        if let Some(first_item) = self.state.search_state.filtered_items.first() {
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
    fn build_table<'t>(ui: &'t mut egui::Ui, first_item: &TableItem) -> TableBuilder<'t> {
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
        let filtered_items = &self.state.search_state.filtered_items;

        table
            .header(20.0, |mut header| {
                if let Some(first_item) = filtered_items.first() {
                    for name in first_item.get_columns() {
                        header.col(|ui| {
                            ui.label(name);
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
                            ui.label(name);
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
                                self.state.popup_state.show_alert = true;
                                self.state.popup_state.selected_route = Some(route.clone());
                            }
                        });
                    }
                });
            });

        // Load more routes if we've reached the end and it's a route table
        if create_more_routes && self.state.search_state.query.is_empty() {
            self.load_more_routes_if_needed();
        }
    }
}
