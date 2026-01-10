use diesel::SqliteConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use flight_planner::database::DatabasePool;
use flight_planner::gui::services::{AppService, Services};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");
pub const AIRPORT_MIGRATIONS: EmbeddedMigrations =
    embed_migrations!("./migrations_airport_database");

#[test]
fn test_services_new() {
    // Create a dummy DB pool
    let aircraft_manager = ConnectionManager::<SqliteConnection>::new("file::memory:");
    let aircraft_pool = Pool::builder().max_size(1).build(aircraft_manager).unwrap();
    let airport_manager = ConnectionManager::<SqliteConnection>::new("file::memory:");
    let airport_pool = Pool::builder().max_size(1).build(airport_manager).unwrap();

    // Run migrations
    aircraft_pool
        .get()
        .unwrap()
        .run_pending_migrations(MIGRATIONS)
        .unwrap();
    airport_pool
        .get()
        .unwrap()
        .run_pending_migrations(AIRPORT_MIGRATIONS)
        .unwrap();

    let db_pool = DatabasePool {
        aircraft_pool,
        airport_pool,
    };

    let app_service = AppService::new(db_pool).unwrap();
    let services = Services::new(app_service, "api_key".to_string());

    assert_eq!(services.weather.api_key(), "api_key");
    assert!(!services.popup.is_alert_visible());
    assert!(!services.search.is_search_pending());
}
