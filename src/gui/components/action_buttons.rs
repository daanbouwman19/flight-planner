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
    /// A flag indicating whether route generation is in progress.
    pub is_loading: bool,
    /// The current display mode of the application.
    pub current_mode: DisplayMode,
}

impl ActionButtonsViewModel {
    /// Creates a new `ActionButtonsViewModel`.
    pub fn new(departure_airport_valid: bool, is_loading: bool, current_mode: DisplayMode) -> Self {
        Self {
            departure_airport_valid,
            is_loading,
            current_mode,
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
        self.departure_airport_valid && !self.is_loading
    }
}

// --- Component ---

/// A UI component that renders the main action buttons for the application.
///
/// This component is responsible for creating buttons that trigger various
/// application-wide actions, such as changing the display mode or generating routes.
pub struct ActionButtons;

#[cfg(not(tarpaulin_include))]
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
            Self::render_random_buttons(vm, ui, events);
            Self::render_list_buttons(vm, ui, events);
            Self::render_route_buttons(vm, ui, events);
        });
    }

    /// Renders random selection buttons.
    fn render_random_buttons(
        vm: &ActionButtonsViewModel,
        ui: &mut Ui,
        events: &mut Vec<AppEvent>,
    ) {
        if ui
            .add(
                egui::Button::new("🎲 Get random airports")
                    .selected(vm.current_mode == DisplayMode::RandomAirports),
            )
            .on_hover_text("Show a random selection of 50 airports")
            .clicked()
        {
            events.push(AppEvent::Ui(UiEvent::SetDisplayMode(
                DisplayMode::RandomAirports,
            )));
        }
    }

    /// Renders list display buttons.
    fn render_list_buttons(vm: &ActionButtonsViewModel, ui: &mut Ui, events: &mut Vec<AppEvent>) {
        if ui
            .add(
                egui::Button::new("🌍 List all airports")
                    .selected(vm.current_mode == DisplayMode::Airports),
            )
            .on_hover_text("Browse the complete database of airports")
            .clicked()
        {
            events.push(AppEvent::Ui(UiEvent::SetDisplayMode(DisplayMode::Airports)));
        }

        if ui
            .add(
                egui::Button::new("✈ List all aircraft")
                    .selected(vm.current_mode == DisplayMode::Other),
            )
            .on_hover_text("View and manage your aircraft fleet")
            .clicked()
        {
            events.push(AppEvent::Ui(UiEvent::SetDisplayMode(DisplayMode::Other)));
        }

        if ui
            .add(
                egui::Button::new("📜 List history")
                    .selected(vm.current_mode == DisplayMode::History),
            )
            .on_hover_text("View your flight history log")
            .clicked()
        {
            events.push(AppEvent::Ui(UiEvent::SetDisplayMode(DisplayMode::History)));
        }

        if ui
            .add(
                egui::Button::new("📊 Statistics")
                    .selected(vm.current_mode == DisplayMode::Statistics),
            )
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
        let is_loading = vm.is_loading;

        let disabled_tooltip = if !departure_airport_valid {
            "Please enter a valid departure airport ICAO code or leave empty for random"
        } else {
            "Route generation in progress..."
        };

        let random_route_text = if is_loading {
            "⏳ Generating..."
        } else {
            "🔀 Random route"
        };

        let is_random_route_selected = matches!(
            vm.current_mode,
            DisplayMode::RandomRoutes | DisplayMode::SpecificAircraftRoutes
        );

        if ui
            .add_enabled(
                departure_airport_valid && !is_loading,
                egui::Button::new(random_route_text).selected(is_random_route_selected),
            )
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

        let not_flown_text = if is_loading {
            "⏳ Generating..."
        } else {
            "🆕 Random route from not flown"
        };

        let is_not_flown_selected = vm.current_mode == DisplayMode::NotFlownRoutes;

        if ui
            .add_enabled(
                departure_airport_valid && !is_loading,
                egui::Button::new(not_flown_text).selected(is_not_flown_selected),
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
