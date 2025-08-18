use crate::gui::data::TableItem;
use crate::gui::state::popup_state::DisplayMode;
use crate::gui::ui::Gui;
use egui::Ui;
use std::sync::Arc;

pub struct ActionButtons;

impl ActionButtons {
    /// Renders action buttons in the original vertical layout with sections.
    pub fn render(gui: &mut Gui, ui: &mut Ui) {
        ui.add_space(10.0);
        ui.separator();
        ui.add_space(5.0);

        // Action buttons section label (matching original)
        ui.label("Actions");
        ui.separator();

        // Vertical layout of buttons (matching original)
        ui.vertical(|ui| {
            Self::render_random_buttons(gui, ui);
            Self::render_list_buttons(gui, ui);
            Self::render_route_buttons(gui, ui);
        });
    }

    /// Renders random selection buttons.
    fn render_random_buttons(gui: &mut Gui, ui: &mut Ui) {
        if ui.button("Get random airports").clicked() {
            gui.set_display_mode(DisplayMode::RandomAirports);
            let random_airports = gui.get_random_airports(50);
            let airport_items: Vec<_> = random_airports
                .iter()
                .map(|airport| {
                    // Get runway data for this airport
                    let runways = gui.get_runways_for_airport(airport);
                    let runway_length = if runways.is_empty() {
                        "No runways".to_string()
                    } else {
                        let longest_runway = runways.iter().max_by_key(|r| r.Length);
                        match longest_runway {
                            Some(runway) => format!("{}ft", runway.Length),
                            None => "No runways".to_string(),
                        }
                    };

                    crate::gui::data::ListItemAirport::new(
                        airport.Name.clone(),
                        airport.ICAO.clone(),
                        runway_length,
                    )
                })
                .collect();
            let table_items: Vec<Arc<TableItem>> = airport_items
                .into_iter()
                .map(|item| Arc::new(TableItem::Airport(item)))
                .collect();
            gui.set_items_with_display_mode(table_items, DisplayMode::RandomAirports);
        }
    }

    /// Renders list display buttons.
    fn render_list_buttons(gui: &mut Gui, ui: &mut Ui) {
        if ui.button("List all airports").clicked() {
            gui.set_display_mode(DisplayMode::Airports);
            gui.update_displayed_items();
        }

        if ui.button("List all aircraft").clicked() {
            gui.set_display_mode(DisplayMode::Other);
            // Generate aircraft items properly using the constructor
            let aircraft_items: Vec<_> = gui
                .get_all_aircraft()
                .iter()
                .map(crate::gui::data::ListItemAircraft::new)
                .collect();
            let table_items: Vec<Arc<TableItem>> = aircraft_items
                .into_iter()
                .map(|item| Arc::new(TableItem::Aircraft(item)))
                .collect();
            gui.set_items_with_display_mode(table_items, DisplayMode::Other);
        }

        if ui.button("List history").clicked() {
            gui.set_display_mode(DisplayMode::History);
            gui.update_displayed_items();
        }

        if ui.button("Statistics").clicked() {
            gui.set_display_mode(DisplayMode::Statistics);
            gui.update_displayed_items();
        }
    }

    /// Renders route generation buttons.
    fn render_route_buttons(gui: &mut Gui, ui: &mut Ui) {
        // Check if departure airport is valid (empty means random)
        let departure_airport_valid = gui.get_departure_airport_icao().is_empty()
            || gui.get_departure_airport_validation().unwrap_or(true);

        let button_tooltip = if departure_airport_valid {
            ""
        } else {
            "Please enter a valid departure airport ICAO code or leave empty for random"
        };

        if ui
            .add_enabled(departure_airport_valid, egui::Button::new("Random route"))
            .on_hover_text(button_tooltip)
            .clicked()
        {
            gui.set_display_mode(DisplayMode::RandomRoutes);
            gui.regenerate_routes_for_selection_change();
        }

        if ui
            .add_enabled(
                departure_airport_valid,
                egui::Button::new("Random route from not flown"),
            )
            .on_hover_text(button_tooltip)
            .clicked()
        {
            gui.set_display_mode(DisplayMode::NotFlownRoutes);
            gui.regenerate_routes_for_selection_change();
        }
    }
}
