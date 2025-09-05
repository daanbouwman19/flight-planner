use diesel::{prelude::*, r2d2::ConnectionManager};
use r2d2::Pool;
use std::path::PathBuf;

use crate::traits::DatabaseOperations;

/// Get the path to the aircraft database file in the application data directory
pub fn get_aircraft_db_path() -> PathBuf {
    crate::get_app_data_dir().join("data.db")
}

/// Get the path to the airports database file
/// 
/// This first checks if airports.db3 exists in the application data directory.
/// If not found, it falls back to the current working directory for backward compatibility.
pub fn get_airport_db_path() -> PathBuf {
    let app_data_path = crate::get_app_data_dir().join("airports.db3");
    
    // If airports.db3 exists in app data directory, use it
    if app_data_path.exists() {
        return app_data_path;
    }
    
    // Otherwise, fall back to current working directory for backward compatibility
    PathBuf::from("airports.db3")
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
        fn establish_database_connection(database_path: &PathBuf) -> SqliteConnection {
            SqliteConnection::establish(database_path.to_str().unwrap()).unwrap_or_else(|_| {
                panic!("Error connecting to {}", database_path.display());
            })
        }

        let aircraft_path = get_aircraft_db_path();
        let airport_path = get_airport_db_path();
        
        let aircraft_connection = establish_database_connection(&aircraft_path);
        let airport_connection = establish_database_connection(&airport_path);

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
            database_path: &PathBuf,
        ) -> Pool<ConnectionManager<SqliteConnection>> {
            let manager = ConnectionManager::<SqliteConnection>::new(database_path.to_str().unwrap());
            Pool::builder().build(manager).unwrap()
        }

        let aircraft_path = get_aircraft_db_path();
        let airport_path = get_airport_db_path();
        
        let aircraft_pool = establish_database_pool(&aircraft_path);
        let airport_pool = establish_database_pool(&airport_path);

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
