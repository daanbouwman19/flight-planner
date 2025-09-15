pub mod aircraft_service;
pub mod airport_service;
pub mod app_service;
pub mod history_service;
pub mod popup_service;
pub mod route_service;
pub mod search_service;
pub mod validation_service;

pub use app_service::AppService;
pub use popup_service::PopupService;
pub use search_service::SearchService;

/// Container for all services (business logic).
/// This makes it easy to pass all services to components without many parameters.
pub struct Services {
    /// Main application service (business logic and data operations).
    pub app: AppService,
    /// Search functionality service.
    pub search: SearchService,
    /// Popup dialog service.
    pub popup: PopupService,
}

impl Services {
    /// Creates a new services container.
    pub fn new(app_service: AppService) -> Self {
        Self {
            app: app_service,
            search: SearchService::new(),
            popup: PopupService::new(),
        }
    }
}
