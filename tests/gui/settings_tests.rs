use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use flight_planner::database::DatabasePool;
use flight_planner::gui::services::AppService;
use serial_test::serial;
use std::error::Error;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
pub const AIRPORT_MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations_airport_database");

#[test]
#[serial]
fn test_set_and_get_api_key() -> Result<(), Box<dyn Error>> {
    // Arrange
    let pool = DatabasePool::new(
        Some("file:aircraft_test.db?mode=memory&cache=shared"),
        Some("file:airport_test.db?mode=memory&cache=shared"),
    )
    .map_err(|e| e.to_string())?;
    pool.aircraft_pool
        .get()?
        .run_pending_migrations(MIGRATIONS)
        .unwrap();
    pool.airport_pool
        .get()?
        .run_pending_migrations(AIRPORT_MIGRATIONS)
        .unwrap();
    let mut app_service = AppService::new(pool.clone())?;
    let api_key = "test_api_key";

    // Act
    app_service.set_api_key(api_key)?;
    let retrieved_key = app_service.get_api_key()?;

    // Assert
    assert_eq!(retrieved_key, Some(api_key.to_string()));

    Ok(())
}
