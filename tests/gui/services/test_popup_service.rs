use flight_planner::gui::services::popup_service::{DisplayMode, PopupService};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let popup_service = PopupService::new();
        assert!(!popup_service.is_alert_visible());
        assert!(popup_service.selected_route().is_none());
        assert_eq!(*popup_service.display_mode(), DisplayMode::RandomRoutes);
    }

    #[test]
    fn test_alert_management() {
        let mut popup_service = PopupService::new();
        popup_service.show_alert();
        assert!(popup_service.is_alert_visible());
        popup_service.hide_alert();
        assert!(!popup_service.is_alert_visible());
        popup_service.set_alert_visibility(true);
        assert!(popup_service.is_alert_visible());
    }

    #[test]
    fn test_route_selection() {
        use flight_planner::gui::data::ListItemRoute;
        use flight_planner::models::{Aircraft, Airport};
        use std::sync::Arc;

        let mut popup_service = PopupService::new();
        assert!(popup_service.selected_route().is_none());

        let dummy_route = ListItemRoute {
            departure: Arc::new(Airport::default()),
            destination: Arc::new(Airport::default()),
            aircraft: Arc::new(Aircraft {
                id: 1,
                manufacturer: "Test".to_string(),
                variant: "Test".to_string(),
                icao_code: "TEST".to_string(),
                flown: 0,
                aircraft_range: 1000,
                category: "A".to_string(),
                cruise_speed: 100,
                date_flown: None,
                takeoff_distance: None,
            }),
            departure_runway_length: 0,
            destination_runway_length: 0,
            route_length: 0.0,
            aircraft_info: "Test Test".to_string().into(),
            departure_info: "Default (DEFAULT)".to_string().into(),
            destination_info: "Default (DEFAULT)".to_string().into(),
            distance_str: "0.0 NM".to_string(),
            created_at: std::time::Instant::now(),
        };

        popup_service.select_route(dummy_route.clone());
        assert!(popup_service.selected_route().is_some());
        // Note: We can't directly compare ListItemRoute because it doesn't derive PartialEq.
        // We can check a field if needed, but for this test, Some/None is enough.

        popup_service.clear_route_selection();
        assert!(popup_service.selected_route().is_none());

        popup_service.set_selected_route(Some(dummy_route));
        assert!(popup_service.selected_route().is_some());
    }

    #[test]
    fn test_display_mode_management() {
        let mut popup_service = PopupService::new();
        popup_service.set_display_mode(DisplayMode::History);
        assert_eq!(*popup_service.display_mode(), DisplayMode::History);
    }

    #[test]
    fn test_is_route_mode() {
        let mut popup_service = PopupService::new();
        popup_service.set_display_mode(DisplayMode::RandomRoutes);
        assert!(popup_service.is_route_mode());
        popup_service.set_display_mode(DisplayMode::NotFlownRoutes);
        assert!(popup_service.is_route_mode());
        popup_service.set_display_mode(DisplayMode::SpecificAircraftRoutes);
        assert!(popup_service.is_route_mode());
        popup_service.set_display_mode(DisplayMode::History);
        assert!(!popup_service.is_route_mode());
        popup_service.set_display_mode(DisplayMode::Airports);
        assert!(!popup_service.is_route_mode());
    }

    #[test]
    fn test_get_appropriate_route_mode() {
        let popup_service = PopupService::new();
        assert_eq!(
            popup_service.get_appropriate_route_mode(true),
            DisplayMode::SpecificAircraftRoutes
        );
        assert_eq!(
            popup_service.get_appropriate_route_mode(false),
            DisplayMode::RandomRoutes
        );
    }

    #[test]
    fn test_should_switch_to_route_mode() {
        let mut popup_service = PopupService::new();

        // Currently not in route mode, selection is being made -> should switch
        popup_service.set_display_mode(DisplayMode::History);
        assert!(popup_service.should_switch_to_route_mode(true));

        // Currently not in route mode, selection is not being made -> should not switch
        assert!(!popup_service.should_switch_to_route_mode(false));

        // Currently in route mode, selection is being made -> should not switch
        popup_service.set_display_mode(DisplayMode::RandomRoutes);
        assert!(!popup_service.should_switch_to_route_mode(true));
    }
}
