//! Provides a suite of services that encapsulate the application's business logic for the GUI.
//!
//! This module follows a service-oriented approach, where each service is
//! responsible for a specific domain of functionality, such as managing popups,
//! handling search, or providing core application data. The `Services` struct
//! acts as a container to conveniently pass all services throughout the UI.

pub mod aircraft_service;
pub mod airport_service;
#[cfg(not(target_arch = "wasm32"))]
pub mod app_service;
pub mod history_service;
pub mod popup_service;
pub mod route_service;
pub mod search_service;
#[cfg(not(target_arch = "wasm32"))]
pub mod validation_service;
#[cfg(not(target_arch = "wasm32"))]
pub mod weather_service;

#[cfg(not(target_arch = "wasm32"))]
pub use app_service::AppService;
pub use popup_service::PopupService;
pub use search_service::SearchService;
#[cfg(not(target_arch = "wasm32"))]
pub use weather_service::WeatherService;

/// A container for all GUI-related services (native desktop build).
#[cfg(not(target_arch = "wasm32"))]
pub struct Services {
    /// The main application service, handling core business logic and data operations.
    pub app: AppService,
    /// The service responsible for search functionality, including debouncing.
    pub search: SearchService,
    /// The service that manages the state of popups and display modes.
    pub popup: PopupService,
    /// The service responsible for fetching weather data.
    pub weather: WeatherService,
}

#[cfg(not(target_arch = "wasm32"))]
impl Services {
    /// Creates a new `Services` container.
    ///
    /// # Arguments
    ///
    /// * `app_service` - An instance of the core `AppService`.
    /// * `api_key` - The AVWX API key.
    pub fn new(app_service: AppService, api_key: String) -> Self {
        Self {
            app: app_service.clone(),
            search: SearchService::new(),
            popup: PopupService::new(),
            weather: WeatherService::new(api_key, app_service.clone_pool()),
        }
    }
}
