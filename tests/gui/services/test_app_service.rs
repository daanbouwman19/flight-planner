use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use flight_planner::database::DatabasePool;
use flight_planner::gui::services::AppService;
use flight_planner::models::Airport;
use flight_planner::schema::{Airports, Runways, aircraft};
use std::error::Error;

// Embed the migrations for both databases
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");
pub const AIRPORT_MIGRATIONS: EmbeddedMigrations =
    embed_migrations!("./migrations_airport_database");
use flight_planner::traits::AirportOperations;

// Define insertable structs for testing, as they are not public in the main crate
#[derive(Insertable)]
#[diesel(table_name = Airports)]
#[allow(non_snake_case)]
pub struct NewAirport {
    pub ICAO: String,
    pub Name: String,
    pub Latitude: f64,
    pub Longtitude: f64,
    pub Elevation: i32,
}

#[derive(Insertable)]
#[diesel(table_name = Runways)]
#[allow(non_snake_case)]
pub struct NewRunway {
    pub AirportID: i32,
    pub Length: i32,
    pub Width: i32,
    pub Surface: String,
    pub Ident: String,
    pub TrueHeading: f64,
    pub Latitude: f64,
    pub Longtitude: f64,
    pub Elevation: i32,
}

// Helper function to set up a test database
fn setup_test_database() -> Result<DatabasePool, Box<dyn Error + Send + Sync>> {
    // Use shared in-memory databases to ensure connections see the same data
    // Use unique names for each test to avoid collisions if run in parallel threads
    let thread_id = std::thread::current().id();
    let aircraft_db_url = format!("file:aircraft_{:?}.db?mode=memory&cache=shared", thread_id);
    let airport_db_url = format!("file:airport_{:?}.db?mode=memory&cache=shared", thread_id);

    // Create pools
    let aircraft_manager = ConnectionManager::<SqliteConnection>::new(&aircraft_db_url);
    let aircraft_pool = Pool::builder().max_size(1).build(aircraft_manager)?;
    let mut aircraft_conn = aircraft_pool.get()?;

    let airport_manager = ConnectionManager::<SqliteConnection>::new(&airport_db_url);
    let airport_pool = Pool::builder().max_size(1).build(airport_manager)?;
    let mut airport_conn = airport_pool.get()?;

    // Run migrations
    aircraft_conn.run_pending_migrations(MIGRATIONS)?;
    airport_conn.run_pending_migrations(AIRPORT_MIGRATIONS)?;

    // Insert test data
    let new_aircraft = vec![
        flight_planner::models::NewAircraft {
            manufacturer: "Test Manufacturer".to_string(),
            variant: "Test Variant 1".to_string(),
            icao_code: "TEST1".to_string(),
            flown: 0,
            aircraft_range: 3000,
            category: "A".to_string(),
            cruise_speed: 450,
            date_flown: None,
            takeoff_distance: Some(1500),
        },
        flight_planner::models::NewAircraft {
            manufacturer: "Test Manufacturer".to_string(),
            variant: "Test Variant 2".to_string(),
            icao_code: "TEST2".to_string(),
            flown: 1,
            aircraft_range: 6000,
            category: "C".to_string(),
            cruise_speed: 500,
            date_flown: Some("2024-01-01".to_string()),
            takeoff_distance: Some(2400),
        },
        flight_planner::models::NewAircraft {
            manufacturer: "Another Manufacturer".to_string(),
            variant: "Variant 3".to_string(),
            icao_code: "TEST3".to_string(),
            flown: 0,
            aircraft_range: 1000,
            category: "B".to_string(),
            cruise_speed: 300,
            date_flown: None,
            takeoff_distance: Some(900),
        },
    ];
    diesel::insert_into(aircraft::table)
        .values(&new_aircraft)
        .execute(&mut aircraft_conn)?;

    // Insert Airports
    let new_airports = vec![
        NewAirport {
            ICAO: "TA1".to_string(),
            Name: "Test Airport 1".to_string(),
            Latitude: 40.0,
            Longtitude: -74.0,
            Elevation: 100,
        },
        NewAirport {
            ICAO: "TA2".to_string(),
            Name: "Test Airport 2".to_string(),
            Latitude: 34.0,
            Longtitude: -118.0,
            Elevation: 200,
        },
        NewAirport {
            ICAO: "TA3".to_string(),
            Name: "Invalid Runway Airport".to_string(),
            Latitude: 50.0,
            Longtitude: -120.0,
            Elevation: 300,
        },
    ];
    diesel::insert_into(Airports::table)
        .values(&new_airports)
        .execute(&mut airport_conn)?;

    // We need the airport IDs to insert runways
    let test_airports: Vec<Airport> = Airports::table.load(&mut airport_conn)?;

    // Insert Runways
    let new_runways = vec![
        NewRunway {
            AirportID: test_airports[0].ID,
            Length: 10000,
            Width: 150,
            Surface: "Asphalt".to_string(),
            Ident: "04L/22R".to_string(),
            TrueHeading: 45.0,
            Latitude: 40.0,
            Longtitude: -74.0,
            Elevation: 100,
        },
        NewRunway {
            AirportID: test_airports[1].ID,
            Length: 8000,
            Width: 100,
            Surface: "Concrete".to_string(),
            Ident: "09/27".to_string(),
            TrueHeading: 90.0,
            Latitude: 34.0,
            Longtitude: -118.0,
            Elevation: 200,
        },
        NewRunway {
            AirportID: test_airports[2].ID,
            Length: 500,
            Width: 50,
            Surface: "Grass".to_string(),
            Ident: "18/36".to_string(),
            TrueHeading: 180.0,
            Latitude: 50.0,
            Longtitude: -120.0,
            Elevation: 300,
        },
    ];
    diesel::insert_into(Runways::table)
        .values(&new_runways)
        .execute(&mut airport_conn)?;

    Ok(DatabasePool {
        aircraft_pool,
        airport_pool,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_service_new_loads_data() {
        let db_pool = setup_test_database().expect("Failed to set up test database");
        let app_service = AppService::new(db_pool).expect("Failed to create AppService");

        // Check that data was loaded from the test database
        assert_eq!(app_service.aircraft().len(), 3);
        assert_eq!(app_service.airports().len(), 3);
        assert_eq!(app_service.aircraft_items().len(), 3);

        // Verify airport items generation on demand
        let airport_items = app_service.generate_airport_items();
        assert_eq!(airport_items.len(), 3);

        assert_eq!(app_service.aircraft()[0].manufacturer, "Test Manufacturer");
        assert_eq!(app_service.airports()[0].Name, "Test Airport 1");
        assert_eq!(airport_items[0].longest_runway_length, "10000ft");
    }

    #[test]
    fn test_get_random_airports() {
        use std::collections::HashSet;

        let db_pool = setup_test_database().unwrap();
        let app_service = AppService::new(db_pool).unwrap();

        // Request all 3 airports to make the test deterministic
        let random_airports = app_service.get_random_airports(3);
        assert_eq!(random_airports.len(), 3);

        // Check that we got the correct set of airports, regardless of order
        let expected_icaos: HashSet<String> =
            vec!["TA1".to_string(), "TA2".to_string(), "TA3".to_string()]
                .into_iter()
                .collect();
        let received_icaos: HashSet<String> =
            random_airports.iter().map(|a| a.ICAO.clone()).collect();

        assert_eq!(expected_icaos, received_icaos);
    }

    #[test]
    fn test_get_runways_for_airport() {
        let db_pool = setup_test_database().unwrap();
        let app_service = AppService::new(db_pool).unwrap();

        let airport_to_test = app_service.airports()[0].clone();
        let runways = app_service.get_runways_for_airport(&airport_to_test);
        assert_eq!(runways.len(), 1);
        assert_eq!(runways[0].Length, 10000);
        assert_eq!(runways[0].Surface, "Asphalt");

        let airport_to_test_2 = app_service.airports()[1].clone();
        let runways_2 = app_service.get_runways_for_airport(&airport_to_test_2);
        assert_eq!(runways_2.len(), 1);
        assert_eq!(runways_2[0].Length, 8000);
    }

    #[test]
    fn test_mark_route_as_flown_and_statistics() {
        use flight_planner::gui::data::ListItemRoute;
        use flight_planner::util;

        let db_pool = setup_test_database().unwrap();
        let mut app_service = AppService::new(db_pool).unwrap();

        // 1. Check initial state
        // History should be empty, so stats should be zero.
        assert_eq!(app_service.history_items().len(), 0);
        let initial_stats = app_service.get_flight_statistics().unwrap();
        assert_eq!(initial_stats.total_flights, 0);
        assert_eq!(initial_stats.total_distance, 0);

        // 2. Create a route to mark as flown
        let aircraft_to_fly = app_service.aircraft()[0].clone();
        let departure_airport = app_service.airports()[0].clone();
        let destination_airport = app_service.airports()[1].clone();
        let expected_distance =
            util::calculate_haversine_distance_nm(&departure_airport, &destination_airport);

        let route_to_fly = ListItemRoute {
            aircraft_info: format!(
                "{} {}",
                aircraft_to_fly.manufacturer, aircraft_to_fly.variant
            )
            .into(),
            departure_info: format!("{} ({})", departure_airport.Name, departure_airport.ICAO)
                .into(),
            destination_info: format!(
                "{} ({})",
                destination_airport.Name, destination_airport.ICAO
            )
            .into(),
            distance_str: String::from("100.0 NM"),
            created_at: std::time::Instant::now(),
            departure: departure_airport.clone(),
            destination: destination_airport.clone(),
            aircraft: aircraft_to_fly.clone(),
            departure_runway_length: 10000,
            destination_runway_length: 8000,
            route_length: 1000.0,
        };

        // 3. Mark the route as flown
        app_service.mark_route_as_flown(&route_to_fly).unwrap();

        // 4. Verify history items are updated in the service
        assert_eq!(app_service.history_items().len(), 1);
        let history_item = &app_service.history_items()[0];
        assert_eq!(history_item.departure_icao, "TA1");
        assert_eq!(history_item.arrival_icao, "TA2");
        assert_eq!(
            history_item.aircraft_name,
            "Test Manufacturer Test Variant 1"
        );

        // 5. Verify statistics are updated correctly
        let new_stats = app_service.get_flight_statistics().unwrap();
        assert_eq!(new_stats.total_flights, 1);
        assert_eq!(new_stats.total_distance, expected_distance);
    }

    #[test]
    fn test_spawn_route_generation_thread_calls_callback() {
        use flight_planner::gui::services::popup_service::DisplayMode;
        use std::sync::mpsc;
        use std::time::Duration;

        let db_pool = setup_test_database().expect("Failed to set up test database");
        let app_service = AppService::new(db_pool).unwrap();
        let (tx, rx) = mpsc::channel();

        app_service.spawn_route_generation_thread(
            DisplayMode::RandomRoutes,
            None,
            None,
            move |routes| {
                tx.send(routes).unwrap();
            },
        );

        let received_routes = rx.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(!received_routes.is_empty());
    }

    #[test]
    fn test_settings_management() {
        let db_pool = setup_test_database().unwrap();
        let mut app_service = AppService::new(db_pool).unwrap();

        // Test general setting
        app_service.set_setting("test_key", "test_value").unwrap();
        assert_eq!(
            app_service.get_setting("test_key").unwrap(),
            Some("test_value".to_string())
        );

        // Test API key specifically
        app_service.set_api_key("secret_key").unwrap();
        assert_eq!(
            app_service.get_api_key().unwrap(),
            Some("secret_key".to_string())
        );
    }

    #[test]
    fn test_aircraft_status_management() {
        let db_pool = setup_test_database().unwrap();
        let mut app_service = AppService::new(db_pool).unwrap();

        let aircraft_id = app_service.aircraft()[0].id;
        let initial_flown = app_service.aircraft()[0].flown;

        // Toggle status
        app_service
            .toggle_aircraft_flown_status(aircraft_id)
            .unwrap();
        assert_ne!(app_service.aircraft()[0].flown, initial_flown);

        // Mark all as not flown
        app_service.mark_all_aircraft_as_not_flown().unwrap();
        for ac in app_service.aircraft() {
            assert_eq!(ac.flown, 0);
        }
    }

    #[test]
    fn test_filtering_and_sorting() {
        let db_pool = setup_test_database().unwrap();
        let mut app_service = AppService::new(db_pool).unwrap();

        // 1. Aircraft filtering
        let filtered_aircraft = app_service.filter_aircraft_items("Test Manufacturer");
        assert_eq!(filtered_aircraft.len(), 2);

        // 2. Route sorting
        // Since routes are randomized during AppService::new, let's just test that sort doesn't panic
        app_service.sort_route_items("distance", true);
        app_service.sort_route_items("departure", false);

        // 3. Display names
        let ac_name = app_service.get_aircraft_display_name(app_service.aircraft()[0].id);
        assert!(ac_name.contains("Test Manufacturer"));

        let ap_name = app_service.get_airport_display_name("TA1");
        assert!(ap_name.contains("Test Airport 1"));
    }

    #[test]
    fn test_add_history_entry() {
        let db_pool = setup_test_database().unwrap();
        let mut app_service = AppService::new(db_pool).unwrap();

        let initial_history_len = app_service.history_items().len();
        let ac = app_service.aircraft()[0].clone();
        let dep = app_service.airports()[0].clone();
        let dest = app_service.airports()[1].clone();

        app_service.add_history_entry(&ac, &dep, &dest).unwrap();
        assert_eq!(app_service.history_items().len(), initial_history_len + 1);
    }

    #[test]
    fn test_route_generation_variants() {
        let db_pool = setup_test_database().unwrap();
        let mut app_service = AppService::new(db_pool).unwrap();

        // 1. Specific aircraft
        let ac = app_service.aircraft()[0].clone();
        app_service.regenerate_routes_for_aircraft(&ac, None);
        assert!(!app_service.route_items().is_empty());

        // 2. Not flown
        app_service.regenerate_not_flown_routes(None);
        assert!(!app_service.route_items().is_empty());

        // 3. Appending
        let initial_len = app_service.route_items().len();
        app_service.append_random_routes(None);
        assert!(app_service.route_items().len() > initial_len);

        app_service.append_not_flown_routes(None);
        app_service.append_routes_for_aircraft(&ac, None);

        // 4. Manual regenerate
        app_service.regenerate_random_routes(None);
    }

    #[test]
    fn test_create_list_item_for_airport() {
        let db_pool = setup_test_database().unwrap();
        let app_service = AppService::new(db_pool).unwrap();

        let airport = app_service.airports()[0].clone();
        let list_item = app_service.create_list_item_for_airport(&airport);

        assert_eq!(list_item.name, "Test Airport 1");
        assert_eq!(list_item.icao, "TA1");
        // Our test data has a runway of 10000ft for TA1
        assert_eq!(list_item.longest_runway_length, "10000ft");
    }

    #[test]
    fn test_mark_all_aircraft_as_not_flown() {
        let db_pool = setup_test_database().unwrap();
        let mut app_service = AppService::new(db_pool).unwrap();

        // One aircraft starts as flown in our mock data (TEST2)
        assert!(app_service.aircraft().iter().any(|ac| ac.flown == 1));

        app_service.mark_all_aircraft_as_not_flown().unwrap();
        assert!(app_service.aircraft().iter().all(|ac| ac.flown == 0));
    }

    #[test]
    fn test_clear_route_items() {
        let db_pool = setup_test_database().unwrap();
        let mut app_service = AppService::new(db_pool).unwrap();

        assert!(!app_service.route_items().is_empty());
        app_service.clear_route_items();
        assert!(app_service.route_items().is_empty());
    }

    #[test]
    fn test_route_items_management() {
        let db_pool = setup_test_database().unwrap();
        let mut app_service = AppService::new(db_pool).unwrap();

        let initial_routes = app_service.route_items().to_vec();

        // Test set_route_items (includes staggered animation logic)
        app_service.set_route_items(initial_routes.clone());
        assert_eq!(app_service.route_items().len(), initial_routes.len());
        if initial_routes.len() > 1 {
            assert!(
                app_service.route_items()[1].created_at >= app_service.route_items()[0].created_at
            );
        }

        // Test append_route_items
        let initial_count = app_service.route_items().len();
        app_service.append_route_items(initial_routes);
        assert_eq!(app_service.route_items().len(), initial_count * 2);
    }

    #[test]
    fn test_more_filtering_and_sorting() {
        let db_pool = setup_test_database().unwrap();
        let mut app_service = AppService::new(db_pool).unwrap();

        // 1. filter_airport_items
        let airport_items = app_service.generate_airport_items();
        let filtered_airports = AppService::filter_airport_items(&airport_items, "TA1");
        assert_eq!(filtered_airports.len(), 1);

        // 2. filter_route_items
        let filtered_routes = app_service.filter_route_items("TA1");
        assert!(!filtered_routes.is_empty());

        // 3. filter_history_items
        // Load history data first by adding an entry
        let ac = app_service.aircraft()[0].clone();
        let dep = app_service.airports()[0].clone();
        let dest = app_service.airports()[1].clone();
        app_service.add_history_entry(&ac, &dep, &dest).unwrap();

        let filtered_history = app_service.filter_history_items("TA1");
        assert!(!filtered_history.is_empty());

        // 4. sort_history_items
        app_service.sort_history_items("departure", true);
    }

    #[test]
    fn test_selection_helpers() {
        let db_pool = setup_test_database().unwrap();
        let app_service = AppService::new(db_pool).unwrap();

        // get_selected_airport_icao
        let airport = app_service.airports()[0].clone();
        assert_eq!(
            app_service.get_selected_airport_icao(&Some(airport)),
            Some("TA1".to_string())
        );
        assert_eq!(app_service.get_selected_airport_icao(&None), None);

        // get_aircraft_display_name
        assert_eq!(
            app_service.get_aircraft_display_name(1),
            "Test Manufacturer Test Variant 1"
        );

        // get_airport_display_name
        assert_eq!(
            app_service.get_airport_display_name("TA1"),
            "Test Airport 1 (TA1)"
        );
    }

    #[test]
    fn test_app_service_getters() {
        let db_pool = setup_test_database().unwrap();
        let mut app_service = AppService::new(db_pool).unwrap();

        let _ = app_service.database_pool();
        let _ = app_service.route_generator();
        let _ = app_service.clone_pool();
        app_service.invalidate_statistics_cache();
    }

    #[test]
    fn test_gui_get_displayed_items() {
        use flight_planner::gui::data::TableItem;
        use flight_planner::gui::state::ApplicationState;
        use flight_planner::gui::ui::Gui;
        use std::sync::Arc;
        use std::sync::mpsc;

        let (route_sender, _) = mpsc::channel();
        let (_, route_receiver) = mpsc::channel();
        let (search_sender, _) = mpsc::channel();
        let (_, search_receiver) = mpsc::channel();
        let (weather_sender, _) = mpsc::channel();
        let (_, weather_receiver) = mpsc::channel();
        let (airport_items_sender, _) = mpsc::channel();
        let (_, airport_items_receiver) = mpsc::channel();

        let mut gui = Gui {
            state: ApplicationState::new(),
            services: None,
            startup_receiver: None,
            startup_error: None,
            route_sender,
            route_receiver,
            search_sender,
            search_receiver,
            weather_sender,
            weather_receiver,
            airport_items_sender,
            airport_items_receiver,
            route_update_request: None,
            is_loading_airport_items: false,
            current_route_generation_id: 0,
            scroll_to_top: false,
        };

        // Test 1: No services, no items
        assert!(gui.get_displayed_items().is_empty());

        // Test 2: Items in state, no services
        gui.state.all_items.push(Arc::new(TableItem::Airport(
            flight_planner::gui::data::ListItemAirport::new(
                "A".to_string(),
                "ICAO".to_string(),
                "1".to_string(),
            ),
        )));
        assert_eq!(gui.get_displayed_items().len(), 1);
    }

    #[test]
    fn test_database_operations_trait_coverage() {
        let db_pool = setup_test_database().unwrap();
        let mut app_service = AppService::new(db_pool).unwrap();

        // Clone aircraft first to avoid immutable borrow while pool (mutable borrow) is active
        let aircraft = app_service.aircraft()[0].clone();

        let pool = app_service.database_pool();

        // 1. get_random_airport
        let random_airport = pool.get_random_airport();
        assert!(random_airport.is_ok());

        // 2. get_airport_by_icao
        let airport = pool.get_airport_by_icao("TA1");
        assert!(airport.is_ok());
        let airport = airport.unwrap();

        // 3. get_runways_for_airport (Trait version)
        let runways = pool.get_runways_for_airport(&airport);
        assert!(runways.is_ok());
        assert!(!runways.unwrap().is_empty());

        // 4. get_destination_airport
        let dest = pool.get_destination_airport(&aircraft, &airport);
        // It might fail depending on distance, but calling it covers the code
        let _ = dest;

        // 5. get_random_airport_for_aircraft
        let random_for_ac = pool.get_random_airport_for_aircraft(&aircraft);
        let _ = random_for_ac;
    }

    #[test]
    fn test_get_route_from_history() {
        let db_pool = setup_test_database().unwrap();
        let mut app_service = AppService::new(db_pool).unwrap();

        // 1. Create a history entry
        let ac = app_service.aircraft()[0].clone();
        let dep = app_service.airports()[0].clone();
        let dest = app_service.airports()[1].clone();
        app_service.add_history_entry(&ac, &dep, &dest).unwrap();

        // 2. Retrieve it
        let history_item = &app_service.history_items()[0];

        // 3. Reconstruct route
        let route = app_service.get_route_from_history(history_item);

        assert!(route.is_some());
        let route = route.unwrap();

        // 4. Verify fields
        assert_eq!(route.departure.ICAO, dep.ICAO);
        assert_eq!(route.destination.ICAO, dest.ICAO);
        assert_eq!(route.aircraft.id, ac.id);
        // TA1 has 10000ft runway, TA2 has 8000ft
        assert_eq!(route.departure_runway_length, 10000);
        assert_eq!(route.destination_runway_length, 8000);

        // Verify calculated fields
        assert!(route.distance_str.contains("NM"));
        assert!(route.aircraft_info.contains(&ac.manufacturer));
    }
}
