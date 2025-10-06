//! This module manages the state and logic for the route details popup.

use crate::gui::data::ListItemRoute;
use crate::modules::weather::Metar;
use std::sync::Arc;

/// Represents the state of a weather data fetch operation.
#[derive(Clone, Debug, Default)]
pub enum WeatherState {
    #[default]
    Idle,
    Loading,
    Success(Metar),
    Error(String),
}

/// Holds the state for the route details popup.
#[derive(Clone, Default)]
pub struct RoutePopupState {
    /// The route currently selected for display in the popup.
    pub selected_route: Option<Arc<ListItemRoute>>,
    /// The weather data for the departure airport.
    pub departure_weather: WeatherState,
    /// The weather data for the destination airport.
    pub destination_weather: WeatherState,
}

/// The service responsible for managing the state of the route details popup.
#[derive(Default)]
pub struct RoutePopupService {
    /// The state of the popup.
    state: RoutePopupState,
    /// A flag indicating whether the popup is visible.
    is_visible: bool,
}

impl RoutePopupService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_visibility(&mut self, visible: bool) {
        self.is_visible = visible;
        if !visible {
            // Reset state when hiding
            self.state = RoutePopupState::default();
        }
    }

    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    pub fn set_selected_route(&mut self, route: Option<Arc<ListItemRoute>>) {
        self.state.selected_route = route;
        // When a new route is selected, reset the weather state
        self.state.departure_weather = WeatherState::Loading;
        self.state.destination_weather = WeatherState::Loading;
    }

    pub fn selected_route(&self) -> &Option<Arc<ListItemRoute>> {
        &self.state.selected_route
    }

    pub fn state(&self) -> &RoutePopupState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut RoutePopupState {
        &mut self.state
    }
}