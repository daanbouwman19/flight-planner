use crate::gui::events::Event;
use crate::gui::services::popup_service::DisplayMode;
use egui::Ui;

// --- View Model ---

/// View-model for the `ActionButtons` component.
#[derive(Debug, Clone)]
pub struct ActionButtonsViewModel {
    pub departure_airport_valid: bool,
}

impl ActionButtonsViewModel {
    /// Creates a new view-model instance.
    pub fn new(departure_airport_valid: bool) -> Self {
        Self {
            departure_airport_valid,
        }
    }

    /// Checks if route generation buttons should be enabled.
    pub fn can_generate_routes(&self) -> bool {
        self.departure_airport_valid
    }
}

// --- Component ---

pub struct ActionButtons;

impl ActionButtons {
    /// Renders action buttons in the original vertical layout with sections.
    pub fn render(vm: &ActionButtonsViewModel, ui: &mut Ui) -> Vec<Event> {
        let mut events = Vec::new();

        // Action buttons section label (matching original)
        ui.label("Actions");
        ui.separator();

        // Vertical layout of buttons (matching original)
        ui.vertical(|ui| {
            events.extend(Self::render_random_buttons(ui));
            events.extend(Self::render_list_buttons(ui));
            events.extend(Self::render_route_buttons(vm, ui));
        });

        events
    }

    /// Renders random selection buttons.
    fn render_random_buttons(ui: &mut Ui) -> Vec<Event> {
        let mut events = Vec::new();
        if ui.button("Get random airports").clicked() {
            events.push(Event::SetDisplayMode(DisplayMode::RandomAirports));
        }
        events
    }

    /// Renders list display buttons.
    fn render_list_buttons(ui: &mut Ui) -> Vec<Event> {
        let mut events = Vec::new();
        if ui.button("List all airports").clicked() {
            events.push(Event::SetDisplayMode(DisplayMode::Airports));
        }

        if ui.button("List all aircraft").clicked() {
            events.push(Event::SetDisplayMode(DisplayMode::Other));
        }

        if ui.button("List history").clicked() {
            events.push(Event::SetDisplayMode(DisplayMode::History));
        }

        if ui.button("Statistics").clicked() {
            events.push(Event::SetDisplayMode(DisplayMode::Statistics));
        }
        events
    }

    /// Renders route generation buttons.
    fn render_route_buttons(vm: &ActionButtonsViewModel, ui: &mut Ui) -> Vec<Event> {
        let mut events = Vec::new();
        // Check if departure airport is valid (empty means random)
        let departure_airport_valid = vm.departure_airport_valid;

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
            events.push(Event::SetDisplayMode(DisplayMode::RandomRoutes));
            events.push(Event::RegenerateRoutesForSelectionChange);
        }

        if ui
            .add_enabled(
                departure_airport_valid,
                egui::Button::new("Random route from not flown"),
            )
            .on_hover_text(button_tooltip)
            .clicked()
        {
            events.push(Event::SetDisplayMode(DisplayMode::NotFlownRoutes));
            events.push(Event::RegenerateRoutesForSelectionChange);
        }
        events
    }
}
