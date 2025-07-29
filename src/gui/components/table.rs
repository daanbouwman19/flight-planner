// OLD COMPONENT IMPLEMENTATION - DISABLED FOR CLEAN ARCHITECTURE
/*
impl Gui {
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
        let columns = first_item.get_columns();

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
                    let columns = first_item.get_columns();

                    for name in columns {
                        header.col(|ui| {
                            ui.label(name.to_string());
                        });
                    }
                }
            })
            .body(|body| {
                let mut selected_route = None;
                let mut aircraft_to_toggle = None;

                body.rows(row_height, filtered_items.len(), |mut row| {
                    let item = &filtered_items[row.index()];
                    let data = item.get_data();

                    // Display all data columns except the last one (which is the action column)
                    for name in &data[0..data.len() - 1] {
                        row.col(|ui| {
                            ui.label(name.to_string());
                        });
                    }

                    // Handle the action column based on item type
                    match item.as_ref() {
                        TableItem::Aircraft(aircraft) => {
                            row.col(|ui| {
                                let button_text = if aircraft.flown == 0 {
                                    "Mark Flown"
                                } else {
                                    "Mark Not Flown"
                                };
                                let button = egui::Button::new(button_text)
                                    .min_size(egui::Vec2::new(100.0, 0.0));
                                if ui.add(button).clicked() {
                                    aircraft_to_toggle = Some(aircraft.id);
                                }
                            });
                        }
                        TableItem::Route(route) => {
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
                        TableItem::Airport(_) => {
                            // Trigger loading more airports when we reach the last visible row
                            if row.index() == filtered_items.len() - 1 {
                                create_more_routes = true;
                            }
                            // Airport items don't have action columns, but we need to maintain consistency
                            // This case shouldn't occur since airports don't have action columns in get_columns
                        }
                        TableItem::History(_) => {
                            // History items don't have action columns
                            // This case shouldn't occur since history doesn't have action columns in get_columns
                        }
                    }
                });

                // Handle actions after the closure
                if let Some(route) = selected_route {
                    self.set_show_alert(true);
                    self.set_selected_route(Some(route));
                }

                if let Some(aircraft_id) = aircraft_to_toggle {
                    self.toggle_aircraft_flown_status(aircraft_id);
                }
            });

        // Load more items if we've reached the end and it's not a search using encapsulated state
        if create_more_routes && self.get_search_query().is_empty() {
            self.load_more_routes_if_needed();
        }
    }
}
*/
