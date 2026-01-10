use diesel::SqliteConnection;
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use flight_planner::database::DatabasePool;
use flight_planner::models::{Aircraft, Airport, History};
use flight_planner::modules::data_operations::{DataOperations, FlightStatistics};
use rand::Rng;
use std::sync::Arc;

fn setup_test_db() -> DatabasePool {
    // Generate a unique database name to ensure isolation between parallel tests
    let mut rng = rand::rng();
    let db_name: u64 = rng.random();
    let db_url = format!("file:test_db_{}?mode=memory&cache=shared", db_name);

    let manager = ConnectionManager::<SqliteConnection>::new(db_url);
    let pool = Pool::builder()
        .max_size(2) // Allow at least two connections to avoid deadlocks in tests
        .build(manager)
        .unwrap();

    let mut conn = pool.get().unwrap();

    // Run migrations/setup tables
    conn.batch_execute(
        "
        CREATE TABLE aircraft (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            manufacturer TEXT NOT NULL,
            variant TEXT NOT NULL,
            icao_code TEXT NOT NULL,
            flown INTEGER NOT NULL DEFAULT 0,
            aircraft_range INTEGER NOT NULL,
            category TEXT NOT NULL,
            cruise_speed INTEGER NOT NULL,
            date_flown TEXT,
            takeoff_distance INTEGER
        );
    ",
    )
    .expect("Failed to create tables");

    DatabasePool {
        aircraft_pool: pool.clone(),
        airport_pool: pool,
    }
}

#[test]
fn test_mark_all_aircraft_as_not_flown() {
    let mut pool = setup_test_db();

    {
        let mut conn = pool.aircraft_pool.get().unwrap();
        conn.batch_execute("
            INSERT INTO aircraft (manufacturer, variant, icao_code, flown, aircraft_range, category, cruise_speed)
            VALUES ('Boeing', '737', 'B737', 1, 3000, 'A', 450);
        ").unwrap();
    }

    DataOperations::mark_all_aircraft_as_not_flown(&mut pool).unwrap();

    // Verify
    {
        let mut conn = pool.aircraft_pool.get().unwrap();
        use flight_planner::schema::aircraft::dsl::*;
        let aircraft_list = aircraft.load::<Aircraft>(&mut conn).unwrap();
        assert_eq!(aircraft_list[0].flown, 0);
    }
}

#[test]
fn test_toggle_aircraft_flown_status() {
    let mut pool = setup_test_db();
    {
        let mut conn = pool.aircraft_pool.get().unwrap();
        conn.batch_execute("
            INSERT INTO aircraft (manufacturer, variant, icao_code, flown, aircraft_range, category, cruise_speed)
            VALUES ('Boeing', '737', 'B737', 0, 3000, 'A', 450);
        ").unwrap();
    }

    // Toggle to flown
    DataOperations::toggle_aircraft_flown_status(&mut pool, 1).unwrap();

    {
        let mut conn = pool.aircraft_pool.get().unwrap();
        use flight_planner::schema::aircraft::dsl::*;
        let aircraft_item = aircraft.find(1).first::<Aircraft>(&mut conn).unwrap();
        assert_eq!(aircraft_item.flown, 1);
        assert!(aircraft_item.date_flown.is_some());
    }

    // Toggle back to not flown
    DataOperations::toggle_aircraft_flown_status(&mut pool, 1).unwrap();

    {
        let mut conn = pool.aircraft_pool.get().unwrap();
        use flight_planner::schema::aircraft::dsl::*;
        let aircraft_item = aircraft.find(1).first::<Aircraft>(&mut conn).unwrap();
        assert_eq!(aircraft_item.flown, 0);
        assert!(aircraft_item.date_flown.is_none());
    }
}

#[test]
fn test_calculate_statistics_from_history_empty() {
    let history: Vec<History> = vec![];
    let aircraft: Vec<Arc<Aircraft>> = vec![];

    let stats = DataOperations::calculate_statistics_from_history(&history, &aircraft);

    assert_eq!(stats.total_flights, 0);
    assert_eq!(stats.total_distance, 0);
    assert_eq!(stats.average_flight_distance, 0.0);
    assert!(stats.most_flown_aircraft.is_none());
}

#[test]
fn test_calculate_statistics_complex() {
    let aircraft1 = Arc::new(Aircraft {
        id: 1,
        manufacturer: "Boeing".to_string(),
        variant: "737".to_string(),
        icao_code: "B737".to_string(),
        flown: 0,
        aircraft_range: 3000,
        category: "A".to_string(),
        cruise_speed: 450,
        date_flown: None,
        takeoff_distance: None,
    });

    let aircraft2 = Arc::new(Aircraft {
        id: 2,
        manufacturer: "Airbus".to_string(),
        variant: "A320".to_string(),
        icao_code: "A320".to_string(),
        flown: 0,
        aircraft_range: 3000,
        category: "A".to_string(),
        cruise_speed: 450,
        date_flown: None,
        takeoff_distance: None,
    });

    let history = vec![
        History {
            id: 1,
            date: "2024-01-01".to_string(),
            aircraft: 1,
            departure_icao: "A".to_string(),
            arrival_icao: "B".to_string(),
            distance: Some(100),
        },
        History {
            id: 2,
            date: "2024-01-02".to_string(),
            aircraft: 1,
            departure_icao: "B".to_string(),
            arrival_icao: "A".to_string(),
            distance: Some(100),
        },
        History {
            id: 3,
            date: "2024-01-03".to_string(),
            aircraft: 2,
            departure_icao: "A".to_string(),
            arrival_icao: "C".to_string(),
            distance: Some(200),
        },
    ];

    let aircraft_list = vec![aircraft1, aircraft2];

    let stats = DataOperations::calculate_statistics_from_history(&history, &aircraft_list);

    assert_eq!(stats.total_flights, 3);
    assert_eq!(stats.total_distance, 400);
}

#[test]
fn test_generate_random_airports() {
    let airport1 = Arc::new(Airport {
        ID: 1,
        Name: "A".to_string(),
        ICAO: "AAAA".to_string(),
        PrimaryID: None,
        Latitude: 0.0,
        Longtitude: 0.0,
        Elevation: 0,
        TransitionAltitude: None,
        TransitionLevel: None,
        SpeedLimit: None,
        SpeedLimitAltitude: None,
    });
    let airport2 = Arc::new(Airport {
        ID: 2,
        Name: "B".to_string(),
        ICAO: "BBBB".to_string(),
        PrimaryID: None,
        Latitude: 0.0,
        Longtitude: 0.0,
        Elevation: 0,
        TransitionAltitude: None,
        TransitionLevel: None,
        SpeedLimit: None,
        SpeedLimitAltitude: None,
    });

    let list = vec![airport1, airport2];

    // Request more than available - implementation samples with replacement
    let random_selection = DataOperations::generate_random_airports(&list, 5);
    assert_eq!(random_selection.len(), 5);

    // Request fewer
    let random_selection = DataOperations::generate_random_airports(&list, 1);
    assert_eq!(random_selection.len(), 1);
}

#[test]
fn test_flight_statistics_debug() {
    let stats = FlightStatistics {
        total_flights: 0,
        total_distance: 0,
        most_flown_aircraft: None,
        most_visited_airport: None,
        average_flight_distance: 0.0,
        longest_flight: None,
        shortest_flight: None,
        favorite_departure_airport: None,
        favorite_arrival_airport: None,
    };
    let debug_str = format!("{:?}", stats);
    assert!(debug_str.contains("FlightStatistics"));
    assert!(debug_str.contains("total_flights"));
}
