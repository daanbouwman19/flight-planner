use diesel::QueryableByName;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::Integer;
use diesel::sqlite::SqliteConnection;
use flight_planner::database::apply_database_optimizations;

mod common;
use common::setup_test_pool_db;

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
    // Initialize DatabasePool with shared setup
    let pool = setup_test_pool_db();

    // Get connection to verify
    let mut conn = pool
        .airport_pool
        .get()
        .expect("Failed to get airport connection");

    // Apply optimizations
    apply_database_optimizations(&pool).expect("Failed to apply optimizations");

    // Verify indexes exist using helper
    check_index_exists(&mut conn, "idx_airports_icao");
    check_index_exists(&mut conn, "idx_runways_airport_id");
}
