use crate::gui::events::{AppEvent, DataEvent, UiEvent};
use crate::gui::services::popup_service::DisplayMode;
use egui::Ui;

// --- View Model ---

/// A view model that provides data and logic for the `ActionButtons` component.
///
/// This struct holds the state necessary to determine whether certain action
/// buttons, particularly those related to route generation, should be enabled.
#[derive(Debug, Clone)]
pub struct ActionButtonsViewModel {
    /// A flag indicating whether the selected departure airport is valid.
    pub departure_airport_valid: bool,
}

impl ActionButtonsViewModel {
    /// Creates a new `ActionButtonsViewModel`.
    pub fn new(departure_airport_valid: bool) -> Self {
        Self {
            departure_airport_valid,
        }
    }

    /// Determines if the route generation buttons should be enabled.
    ///
    /// This is based on the validity of the selected departure airport.
    ///
    /// # Returns
    ///
    /// `true` if route generation is allowed, `false` otherwise.
    pub fn can_generate_routes(&self) -> bool {
        self.departure_airport_valid
    }
}

// --- Component ---

/// A UI component that renders the main action buttons for the application.
///
/// This component is responsible for creating buttons that trigger various
/// application-wide actions, such as changing the display mode or generating routes.
pub struct ActionButtons;

impl ActionButtons {
    /// Renders the action buttons and appends events triggered by user interaction to the provided vector.
    ///
    /// The buttons are grouped into logical sections: random selections, list displays,
    /// and route generation.
    ///
    /// # Arguments
    ///
    /// * `vm` - The `ActionButtonsViewModel` containing the necessary data and logic.
    /// * `ui` - A mutable reference to the `egui::Ui` context for rendering.
    /// * `events` - A mutable reference to the event buffer.
    #[cfg(not(tarpaulin_include))]
    pub fn render(vm: &ActionButtonsViewModel, ui: &mut Ui, events: &mut Vec<AppEvent>) {
        // Action buttons section label (matching original)
        ui.label("Actions");
        ui.separator();

        // Vertical layout of buttons (matching original)
        ui.vertical(|ui| {
            Self::render_random_buttons(ui, events);
            Self::render_list_buttons(ui, events);
            Self::render_route_buttons(vm, ui, events);
        });
    }

    /// Renders random selection buttons.
    fn render_random_buttons(ui: &mut Ui, events: &mut Vec<AppEvent>) {
        if ui
            .button("🎲 Get random airports")
            .on_hover_text("Show a random selection of 50 airports")
            .clicked()
        {
            events.push(AppEvent::Ui(UiEvent::SetDisplayMode(
                DisplayMode::RandomAirports,
            )));
        }
    }

    /// Renders list display buttons.
    fn render_list_buttons(ui: &mut Ui, events: &mut Vec<AppEvent>) {
        if ui
            .button("🌍 List all airports")
            .on_hover_text("Browse the complete database of airports")
            .clicked()
        {
            events.push(AppEvent::Ui(UiEvent::SetDisplayMode(DisplayMode::Airports)));
        }

        if ui
            .button("✈ List all aircraft")
            .on_hover_text("View and manage your aircraft fleet")
            .clicked()
        {
            events.push(AppEvent::Ui(UiEvent::SetDisplayMode(DisplayMode::Other)));
        }

        if ui
            .button("📜 List history")
            .on_hover_text("View your flight history log")
            .clicked()
        {
            events.push(AppEvent::Ui(UiEvent::SetDisplayMode(DisplayMode::History)));
        }

        if ui
            .button("📊 Statistics")
            .on_hover_text("View flight statistics and achievements")
            .clicked()
        {
            events.push(AppEvent::Ui(UiEvent::SetDisplayMode(
                DisplayMode::Statistics,
            )));
        }
    }

    /// Renders route generation buttons.
    fn render_route_buttons(vm: &ActionButtonsViewModel, ui: &mut Ui, events: &mut Vec<AppEvent>) {
        // Check if departure airport is valid (empty means random)
        let departure_airport_valid = vm.departure_airport_valid;

        let disabled_tooltip =
            "Please enter a valid departure airport ICAO code or leave empty for random";

        if ui
            .add_enabled(departure_airport_valid, egui::Button::new("🔀 Random route"))
            .on_hover_text("Generate a random route starting from the selected airport (or a random one if none selected)")
            .on_disabled_hover_text(disabled_tooltip)
            .clicked()
        {
            events.push(AppEvent::Ui(UiEvent::SetDisplayMode(
                DisplayMode::RandomRoutes,
            )));
            events.push(AppEvent::Data(
                DataEvent::RegenerateRoutesForSelectionChange,
            ));
        }

        if ui
            .add_enabled(
                departure_airport_valid,
                egui::Button::new("🆕 Random route from not flown"),
            )
            .on_hover_text("Generate a route to a destination you haven't visited yet")
            .on_disabled_hover_text(disabled_tooltip)
            .clicked()
        {
            events.push(AppEvent::Ui(UiEvent::SetDisplayMode(
                DisplayMode::NotFlownRoutes,
            )));
            events.push(AppEvent::Data(
                DataEvent::RegenerateRoutesForSelectionChange,
            ));
        }
    }
}
