use diesel::{prelude::*, r2d2::{ConnectionManager, Pool}};
use diesel::migration::MigrationSource;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use flight_planner::models::{Aircraft, NewAircraft};
use flight_planner::schema::aircraft;

// Embed all migrations
const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

// Helper function to set up an in-memory database and run migrations
fn setup_database() -> Pool<ConnectionManager<SqliteConnection>> {
    let manager = ConnectionManager::<SqliteConnection>::new(":memory:");
    let pool = Pool::builder()
        .build(manager)
        .expect("Failed to create in-memory database pool.");

    let mut conn = pool.get().expect("Failed to get connection from pool.");
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run migrations.");

    pool
}

#[test]
fn test_update_husky_range_migration() {
    // 1. Set up a database and run all migrations to get the schema
    let manager = ConnectionManager::<SqliteConnection>::new(":memory:");
    let pool = Pool::builder().build(manager).unwrap();
    let mut conn = pool.get().unwrap();
    conn.run_pending_migrations(MIGRATIONS).unwrap();

    // 2. Insert the aircraft with the OLD, incorrect range to simulate existing data
    let husky = NewAircraft {
        manufacturer: "Aviat".to_string(),
        variant: "Husky A-1C".to_string(),
        icao_code: "HUSK".to_string(),
        flown: 0,
        aircraft_range: 14400, // The incorrect value
        category: "Utility".to_string(),
        cruise_speed: 91,
        date_flown: None,
        takeoff_distance: Some(229),
    };
    diesel::insert_into(aircraft::table)
        .values(&husky)
        .execute(&mut conn)
        .expect("Failed to insert test aircraft.");

    // 3. Re-run migrations. Diesel is smart enough to only run pending ones.
    // In a real scenario, the user would update, and the new migration would be pending.
    // For this test, we have to manually revert the last one and then run pending.

    // In a real scenario, the user would update, and the new migration would be pending.
    // For this test, we have to manually revert the last one and then run pending.
    conn.revert_last_migration(MIGRATIONS).unwrap();

    // Verify it's back to the incorrect value (or whatever it was before, in this case, it was just inserted)
    let ac: Aircraft = aircraft::table
        .filter(aircraft::manufacturer.eq("Aviat"))
        .filter(aircraft::variant.eq("Husky A-1C"))
        .first(&mut conn)
        .unwrap();
    assert_eq!(ac.aircraft_range, 14400);

    // 4. Run pending migrations again, which should apply our fix
    conn.run_pending_migrations(MIGRATIONS).unwrap();

    // 5. Verify the range has been updated to the correct value
    let updated_ac: Aircraft = aircraft::table
        .filter(aircraft::manufacturer.eq("Aviat"))
        .filter(aircraft::variant.eq("Husky A-1C"))
        .first(&mut conn)
        .unwrap();
    assert_eq!(updated_ac.aircraft_range, 695);

    // 6. (Optional but good) Test the down migration
    conn.revert_last_migration(MIGRATIONS).unwrap();
    let reverted_ac: Aircraft = aircraft::table
        .filter(aircraft::manufacturer.eq("Aviat"))
        .filter(aircraft::variant.eq("Husky A-1C"))
        .first(&mut conn)
        .unwrap();
    assert_eq!(reverted_ac.aircraft_range, 14400);
}