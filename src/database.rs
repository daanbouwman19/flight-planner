use diesel::{prelude::*, r2d2::ConnectionManager};
use r2d2::Pool;
use directories::ProjectDirs;
use std::path::PathBuf;
use std::fs;
use once_cell::sync::Lazy;

use crate::traits::DatabaseOperations;

// Define PROJECT_DIRS using Lazy
static PROJECT_DIRS: Lazy<Option<ProjectDirs>> = Lazy::new(|| {
    ProjectDirs::from("com.github.flightplanner.FlightPlanner", "FlightPlanner",  "FlightPlannerApp")
});

fn get_app_data_dir() -> PathBuf {
    let base_dirs = PROJECT_DIRS.as_ref().expect("Could not get project directories");
    let data_dir = base_dirs.data_dir().join("flight-planner");
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir).expect("Failed to create app data directory");
    }
    data_dir
}

pub fn aircraft_db_path() -> PathBuf {
    get_app_data_dir().join("data.db")
}

pub fn airport_db_path() -> PathBuf {
    get_app_data_dir().join("airports.db3")
}

pub struct DatabaseConnections {
    pub aircraft_connection: SqliteConnection,
    pub airport_connection: SqliteConnection,
}

impl Default for DatabaseConnections {
    fn default() -> Self {
        Self::new()
    }
}

impl DatabaseOperations for DatabaseConnections {}

impl DatabaseConnections {
    pub fn new() -> Self {
        fn establish_database_connection(database_path: &str) -> SqliteConnection {
            SqliteConnection::establish(database_path).unwrap_or_else(|_| {
                panic!("Error connecting to {database_path}");
            })
        }

        let aircraft_connection = establish_database_connection(aircraft_db_path().to_str().expect("Aircraft DB path is not valid UTF-8"));
        let airport_connection = establish_database_connection(airport_db_path().to_str().expect("Airport DB path is not valid UTF-8"));

        Self {
            aircraft_connection,
            airport_connection,
        }
    }
}

pub struct DatabasePool {
    pub aircraft_pool: Pool<ConnectionManager<SqliteConnection>>,
    pub airport_pool: Pool<ConnectionManager<SqliteConnection>>,
}

impl DatabasePool {
    pub fn new() -> Self {
        fn establish_database_pool(
            database_path: &str,
        ) -> Pool<ConnectionManager<SqliteConnection>> {
            let manager = ConnectionManager::<SqliteConnection>::new(database_path);
            Pool::builder().build(manager).unwrap()
        }

        let aircraft_pool = establish_database_pool(aircraft_db_path().to_str().expect("Aircraft DB path is not valid UTF-8"));
        let airport_pool = establish_database_pool(airport_db_path().to_str().expect("Airport DB path is not valid UTF-8"));

        Self {
            aircraft_pool,
            airport_pool,
        }
    }
}

impl Default for DatabasePool {
    fn default() -> Self {
        Self::new()
    }
}

impl DatabaseOperations for DatabasePool {}
