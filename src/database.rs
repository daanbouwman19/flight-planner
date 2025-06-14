use diesel::{prelude::*, r2d2::ConnectionManager};
use r2d2::Pool;
use directories::ProjectDirs;
use std::path::PathBuf;
use std::fs;
use std::sync::LazyLock;
use std::error::Error as StdError;
use std::boxed::Box;

use crate::traits::DatabaseOperations;

// Define PROJECT_DIRS using LazyLock
static PROJECT_DIRS: LazyLock<Option<ProjectDirs>> = LazyLock::new(|| {
    ProjectDirs::from("com.github.flightplanner.FlightPlanner", "FlightPlanner",  "FlightPlannerApp")
});

pub fn get_app_data_dir() -> Result<PathBuf, std::io::Error> {
    let base_dirs = PROJECT_DIRS.as_ref().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not determine project directories. Check system configuration.",
        )
    })?;
    let data_dir = base_dirs.data_dir().join("flight-planner");
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir)?;
    }
    Ok(data_dir)
}

pub fn aircraft_db_path() -> Result<PathBuf, std::io::Error> {
    Ok(get_app_data_dir()?.join("data.db"))
}

pub fn airport_db_path() -> Result<PathBuf, std::io::Error> {
    Ok(get_app_data_dir()?.join("airports.db3"))
}

pub struct DatabaseConnections {
    pub aircraft_connection: SqliteConnection,
    pub airport_connection: SqliteConnection,
}

// Removed Default implementation for DatabaseConnections

impl DatabaseOperations for DatabaseConnections {}

impl DatabaseConnections {
    pub fn new() -> Result<Self, Box<dyn StdError>> {
        fn establish_database_connection(database_name: &str) -> Result<SqliteConnection, diesel::ConnectionError> {
            SqliteConnection::establish(database_name)
        }

        let air_db_path_obj = aircraft_db_path().map_err(|e| Box::new(e) as Box<dyn StdError>)?;
        let aircraft_db_str = air_db_path_obj.to_str()
            .ok_or_else(|| Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Aircraft DB path is not valid UTF-8")) as Box<dyn StdError>)?;

        let ap_db_path_obj = airport_db_path().map_err(|e| Box::new(e) as Box<dyn StdError>)?;
        let airport_db_str = ap_db_path_obj.to_str()
            .ok_or_else(|| Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Airport DB path is not valid UTF-8")) as Box<dyn StdError>)?;

        let aircraft_connection = establish_database_connection(aircraft_db_str)?;
        let airport_connection = establish_database_connection(airport_db_str)?;

        Ok(Self {
            aircraft_connection,
            airport_connection,
        })
    }
}

pub struct DatabasePool {
    pub aircraft_pool: Pool<ConnectionManager<SqliteConnection>>,
    pub airport_pool: Pool<ConnectionManager<SqliteConnection>>,
}

// Removed Default implementation for DatabasePool

impl DatabasePool {
    pub fn new() -> Result<Self, Box<dyn StdError>> {
        fn establish_database_pool(database_name: &str) -> Result<Pool<ConnectionManager<SqliteConnection>>, r2d2::Error> {
            let manager = ConnectionManager::<SqliteConnection>::new(database_name);
            Pool::builder().build(manager)
        }

        let air_db_path_obj = aircraft_db_path().map_err(|e| Box::new(e) as Box<dyn StdError>)?;
        let aircraft_db_str = air_db_path_obj.to_str()
            .ok_or_else(|| Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Aircraft DB path is not valid UTF-8")) as Box<dyn StdError>)?;

        let ap_db_path_obj = airport_db_path().map_err(|e| Box::new(e) as Box<dyn StdError>)?;
        let airport_db_str = ap_db_path_obj.to_str()
            .ok_or_else(|| Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Airport DB path is not valid UTF-8")) as Box<dyn StdError>)?;

        let aircraft_pool = establish_database_pool(aircraft_db_str)?;
        let airport_pool = establish_database_pool(airport_db_str)?;

        Ok(Self {
            aircraft_pool,
            airport_pool,
        })
    }
}

// Removed Default for DatabasePool (already handled by commenting above, but ensure it's gone)

impl DatabaseOperations for DatabasePool {}
