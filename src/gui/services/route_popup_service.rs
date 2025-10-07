//! This module manages the state and logic for the route details popup.

use crate::gui::data::ListItemRoute;
use crate::modules::weather::Metar;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

const CACHE_DURATION: Duration = Duration::from_secs(60 * 10); // 10 minutes

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
    /// A cache for weather data to reduce API calls.
    weather_cache: HashMap<String, Metar>,
    /// Timestamps for the cached weather data.
    cache_timestamps: HashMap<String, Instant>,
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

    pub fn update_cache(&mut self, icao: &str, metar: &Metar) {
        self.weather_cache.insert(icao.to_string(), metar.clone());
        self.cache_timestamps
            .insert(icao.to_string(), Instant::now());
    }

    pub fn get_cached_weather(&self, icao: &str) -> Option<&Metar> {
        if let Some(timestamp) = self.cache_timestamps.get(icao)
            && timestamp.elapsed() < CACHE_DURATION {
            return self.weather_cache.get(icao);
        }
        None
    }
}