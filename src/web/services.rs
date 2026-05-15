use crate::gui::services::{PopupService, SearchService};
use crate::web::app_service::WebAppService;
use crate::web::weather_service::WebWeatherService;

/// WASM-compatible services container — mirrors the native `Services` struct.
///
/// Field names are identical so `Gui::ui()` rendering code works unchanged
/// across both native and WASM compilation targets.
pub struct WebServices {
    pub app: WebAppService,
    pub search: SearchService,
    pub popup: PopupService,
    pub weather: WebWeatherService,
}

impl WebServices {
    pub fn new(app: WebAppService, weather: WebWeatherService) -> Self {
        Self {
            app,
            search: SearchService::new(),
            popup: PopupService::new(),
            weather,
        }
    }
}
