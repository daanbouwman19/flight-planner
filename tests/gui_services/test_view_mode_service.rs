use flight_planner::gui::services::view_mode_service::{DisplayMode, ViewModeService};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let view_mode_service = ViewModeService::new();
        assert_eq!(*view_mode_service.display_mode(), DisplayMode::RandomRoutes);
    }

    #[test]
    fn test_display_mode_management() {
        let mut view_mode_service = ViewModeService::new();
        view_mode_service.set_display_mode(DisplayMode::History);
        assert_eq!(*view_mode_service.display_mode(), DisplayMode::History);
    }

    #[test]
    fn test_is_route_mode() {
        let mut view_mode_service = ViewModeService::new();
        view_mode_service.set_display_mode(DisplayMode::RandomRoutes);
        assert!(view_mode_service.is_route_mode());
        view_mode_service.set_display_mode(DisplayMode::NotFlownRoutes);
        assert!(view_mode_service.is_route_mode());
        view_mode_service.set_display_mode(DisplayMode::SpecificAircraftRoutes);
        assert!(view_mode_service.is_route_mode());
        view_mode_service.set_display_mode(DisplayMode::History);
        assert!(!view_mode_service.is_route_mode());
        view_mode_service.set_display_mode(DisplayMode::Airports);
        assert!(!view_mode_service.is_route_mode());
    }

    #[test]
    fn test_get_appropriate_route_mode() {
        let view_mode_service = ViewModeService::new();
        assert_eq!(
            view_mode_service.get_appropriate_route_mode(true),
            DisplayMode::SpecificAircraftRoutes
        );
        assert_eq!(
            view_mode_service.get_appropriate_route_mode(false),
            DisplayMode::RandomRoutes
        );
    }
}