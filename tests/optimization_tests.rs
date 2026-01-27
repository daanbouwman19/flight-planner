use flight_planner::database::apply_database_optimizations;
use diesel::prelude::*;
use diesel::sql_types::Integer;
use diesel::sql_query;
use diesel::QueryableByName;

mod common;
use common::TempDir;
use flight_planner::database::DatabasePool;
use diesel::connection::SimpleConnection;

#[derive(QueryableByName, Debug)]
struct IndexCount {
    #[diesel(sql_type = Integer)]
    count: i32,
}

#[test]
fn test_apply_database_optimizations() {
    // Use TempDir to create temporary database files
    let tmp_dir = TempDir::new("test_optimization");
    let airport_db_path = tmp_dir.path.join("airports.db3");
    let aircraft_db_path = tmp_dir.path.join("data.db");

    let airport_db_str = airport_db_path.to_str().unwrap();
    let aircraft_db_str = aircraft_db_path.to_str().unwrap();

    // Initialize DatabasePool with file paths
    let pool = DatabasePool::new(Some(aircraft_db_str), Some(airport_db_str)).expect("Failed to create pool");

    // Setup schema (create tables) in the airport connection
    let mut conn = pool.airport_pool.get().expect("Failed to get airport connection");

    // Create tables manually
    conn.batch_execute("
        CREATE TABLE airports (
            id INTEGER PRIMARY KEY,
            icao TEXT NOT NULL
        );
        CREATE TABLE runways (
            id INTEGER PRIMARY KEY,
            airportid INTEGER NOT NULL
        );
    ").expect("Failed to create tables");

    // Apply optimizations
    apply_database_optimizations(&pool).expect("Failed to apply optimizations");

    // Verify indexes exist
    // Check idx_airports_icao
    let results: Vec<IndexCount> = sql_query("SELECT count(*) as count FROM sqlite_master WHERE type='index' AND name='idx_airports_icao'")
        .load(&mut conn)
        .expect("Failed to query sqlite_master for airports index");
    assert_eq!(results[0].count, 1, "Index idx_airports_icao should exist");

    // Check idx_runways_airport_id
    let results: Vec<IndexCount> = sql_query("SELECT count(*) as count FROM sqlite_master WHERE type='index' AND name='idx_runways_airport_id'")
        .load(&mut conn)
        .expect("Failed to query sqlite_master for runways index");
    assert_eq!(results[0].count, 1, "Index idx_runways_airport_id should exist");
}
