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
    // Create in-memory SQLite database pools
    let aircraft_manager = ConnectionManager::<SqliteConnection>::new(":memory:");
    let aircraft_pool = Pool::builder().build(aircraft_manager)?;
    let mut aircraft_conn = aircraft_pool.get()?;

    let airport_manager = ConnectionManager::<SqliteConnection>::new(":memory:");
    let airport_pool = Pool::builder().build(airport_manager)?;
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
        assert_eq!(app_service.airport_items().len(), 3);

        assert_eq!(app_service.aircraft()[0].manufacturer, "Test Manufacturer");
        assert_eq!(app_service.airports()[0].Name, "Test Airport 1");
        assert_eq!(
            app_service.airport_items()[0].longest_runway_length,
            "10000ft"
        );
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
            departure: departure_airport,
            destination: destination_airport,
            aircraft: aircraft_to_fly,
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
}
