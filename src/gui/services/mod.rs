//! Provides a suite of services that encapsulate the application's business logic for the GUI.
//!
//! This module follows a service-oriented approach, where each service is
//! responsible for a specific domain of functionality, such as managing popups,
//! handling search, or providing core application data. The `Services` struct
//! acts as a container to conveniently pass all services throughout the UI.

pub mod aircraft_service;
pub mod airport_service;
pub mod app_service;
pub mod history_service;
pub mod route_popup_service;
pub mod route_service;
pub mod search_service;
pub mod validation_service;
pub mod view_mode_service;

pub use app_service::AppService;
pub use route_popup_service::RoutePopupService;
pub use search_service::SearchService;
pub use view_mode_service::ViewModeService;

/// A container for all GUI-related services.
///
/// This struct aggregates the various services used by the application, making
/// it easy to pass them around as a single unit.
pub struct Services {
    /// The main application service, handling core business logic and data operations.
    pub app: AppService,
    /// The service responsible for search functionality, including debouncing.
    pub search: SearchService,
    /// The service that manages the main display mode.
    pub view_mode: ViewModeService,
    /// The service that manages the state of the route details popup.
    pub route_popup: RoutePopupService,
}

impl Services {
    /// Creates a new `Services` container.
    ///
    /// # Arguments
    ///
    /// * `app_service` - An instance of the core `AppService`.
    pub fn new(app_service: AppService) -> Self {
        Self {
            app: app_service,
            search: SearchService::new(),
            view_mode: ViewModeService::new(),
            route_popup: RoutePopupService::new(),
        }
    }
}
