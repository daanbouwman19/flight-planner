use diesel::QueryableByName;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::Integer;
use diesel::sqlite::SqliteConnection;
use flight_planner::database::apply_database_optimizations;

mod common;
use common::TempDir;
use diesel::connection::SimpleConnection;
use flight_planner::database::DatabasePool;

#[derive(QueryableByName, Debug)]
struct IndexCount {
    #[diesel(sql_type = Integer)]
    count: i32,
}

fn check_index_exists(conn: &mut SqliteConnection, index_name: &str) {
    let results: Vec<IndexCount> =
        sql_query("SELECT count(*) as count FROM sqlite_master WHERE type='index' AND name=?")
            .bind::<diesel::sql_types::Text, _>(index_name)
            .load(conn)
            .unwrap_or_else(|e| panic!("Failed to query for index {}: {}", index_name, e));
    assert_eq!(results[0].count, 1, "Index {} should exist", index_name);
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
    let pool = DatabasePool::new(Some(aircraft_db_str), Some(airport_db_str))
        .expect("Failed to create pool");

    // Setup schema (create tables) in the airport connection
    let mut conn = pool
        .airport_pool
        .get()
        .expect("Failed to get airport connection");

    // Create tables manually
    conn.batch_execute(
        "
        CREATE TABLE Airports (
            ID INTEGER PRIMARY KEY,
            ICAO TEXT NOT NULL
        );
        CREATE TABLE Runways (
            ID INTEGER PRIMARY KEY,
            AirportID INTEGER NOT NULL
        );
    ",
    )
    .expect("Failed to create tables");

    // Apply optimizations
    apply_database_optimizations(&pool).expect("Failed to apply optimizations");

    // Verify indexes exist using helper
    check_index_exists(&mut conn, "idx_airports_icao");
    check_index_exists(&mut conn, "idx_runways_airport_id");
}
