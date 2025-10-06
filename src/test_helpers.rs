//! Utility functions to support integration and unit tests.
//!
//! This module is only compiled for test and debug builds. It provides
//! helper functions to set up a consistent test environment, such as creating
//! in-memory databases and seeding them with test data.

use crate::database::DatabasePool;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

const AIRCRAFT_MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");
const AIRPORT_MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations_airport_database");

/// Sets up an in-memory SQLite database for testing.
///
/// This function creates a new in-memory database with a unique name for each
/// test run to ensure test isolation. It runs all necessary migrations and
/// seeds the database with a standard set of test data, including sample
/// aircraft, airports, and runways.
///
/// # Returns
///
/// A `DatabasePool` connected to the fully initialized in-memory test database.
pub fn setup_database() -> DatabasePool {
    use crate::models::{Airport, NewAircraft, Runway};
    use crate::schema::{
        Airports as airports_schema, Runways as runways_schema, aircraft as aircraft_schema,
    };
    use diesel::RunQueryDsl;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static DB_COUNTER: AtomicUsize = AtomicUsize::new(0);
    let aircraft_db_name = format!(
        "file:aircraft_test_db_{}?mode=memory&cache=shared",
        DB_COUNTER.fetch_add(1, Ordering::SeqCst)
    );
    let airport_db_name = format!(
        "file:airport_test_db_{}?mode=memory&cache=shared",
        DB_COUNTER.fetch_add(1, Ordering::SeqCst)
    );

    // In-memory database for testing
    let db_pool = DatabasePool::new(Some(&aircraft_db_name), Some(&airport_db_name)).unwrap();
    let mut aircraft_conn = db_pool.aircraft_pool.get().unwrap();
    let mut airport_conn = db_pool.airport_pool.get().unwrap();

    aircraft_conn
        .run_pending_migrations(AIRCRAFT_MIGRATIONS)
        .expect("Failed to run aircraft migrations in test");
    airport_conn
        .run_pending_migrations(AIRPORT_MIGRATIONS)
        .expect("Failed to run airport migrations in test");

    // Insert test data
    let test_aircraft = NewAircraft {
        manufacturer: "TestAir".to_string(),
        variant: "T-1".to_string(),
        icao_code: "TEST".to_string(),
        flown: 0,
        aircraft_range: 1000,
        category: "A".to_string(),
        cruise_speed: 400,
        date_flown: None,
        takeoff_distance: Some(4000),
    };
    diesel::insert_into(aircraft_schema::table)
        .values(&test_aircraft)
        .execute(&mut aircraft_conn)
        .unwrap();

    let test_airport1 = Airport {
        ID: 1,
        Name: "Airport A".to_string(),
        ICAO: "AAAA".to_string(),
        PrimaryID: None,
        Latitude: 0.0,
        Longtitude: 0.0,
        Elevation: 0,
        TransitionAltitude: None,
        TransitionLevel: None,
        SpeedLimit: None,
        SpeedLimitAltitude: None,
    };
    let test_airport2 = Airport {
        ID: 2,
        Name: "Airport B".to_string(),
        ICAO: "BBBB".to_string(),
        PrimaryID: None,
        Latitude: 1.0,
        Longtitude: 1.0,
        Elevation: 0,
        TransitionAltitude: None,
        TransitionLevel: None,
        SpeedLimit: None,
        SpeedLimitAltitude: None,
    };
    diesel::insert_into(airports_schema::table)
        .values(&vec![test_airport1, test_airport2])
        .execute(&mut airport_conn)
        .unwrap();

    let test_runway1 = Runway {
        ID: 1,
        AirportID: 1,
        Length: 14000,
        Width: 150,
        Surface: "ASPH".to_string(),
        Latitude: 0.0,
        Longtitude: 0.0,
        Elevation: 0,
        Ident: "09/27".to_string(),
        TrueHeading: 90.0,
    };
    let test_runway2 = Runway {
        ID: 2,
        AirportID: 2,
        Length: 14000,
        Width: 150,
        Surface: "ASPH".to_string(),
        Latitude: 1.0,
        Longtitude: 1.0,
        Elevation: 0,
        Ident: "01/19".to_string(),
        TrueHeading: 10.0,
    };
    diesel::insert_into(runways_schema::table)
        .values(&vec![test_runway1, test_runway2])
        .execute(&mut airport_conn)
        .unwrap();

    db_pool
}
