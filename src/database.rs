use diesel::{prelude::*, r2d2::ConnectionManager};
use r2d2::Pool;
use std::path::{Path, PathBuf};

use crate::{traits::DatabaseOperations, errors::Error};

/// Get the path to the aircraft database file in the application data directory
pub fn get_aircraft_db_path() -> Result<PathBuf, Error> {
    let base = crate::get_app_data_dir()?;
    Ok(base.join("data.db"))
}

/// Get the path to the airports database file
/// 
/// This first checks if airports.db3 exists in the application data directory.
/// If not found, it falls back to the current working directory for backward compatibility.
pub fn get_airport_db_path() -> Result<PathBuf, Error> {
    let base = crate::get_app_data_dir()?;
    let app_data_path = base.join("airports.db3");
    if app_data_path.exists() {
        return Ok(app_data_path);
    }
    Ok(PathBuf::from("airports.db3"))
}

pub struct DatabaseConnections {
    pub aircraft_connection: SqliteConnection,
    pub airport_connection: SqliteConnection,
}

impl Default for DatabaseConnections {
    fn default() -> Self {
    Self::new().expect("Failed to initialize database connections")
    }
}

impl DatabaseOperations for DatabaseConnections {}

impl DatabaseConnections {
    pub fn new() -> Result<Self, Error> {
        fn establish_database_connection(database_path: &Path) -> Result<SqliteConnection, Error> {
            let Some(path_str) = database_path.to_str() else {
                return Err(Error::InvalidPath(database_path.display().to_string()));
            };
            SqliteConnection::establish(path_str).map_err(|e| Error::Other(std::io::Error::other(e.to_string())))
        }

        let aircraft_path = get_aircraft_db_path()?;
        let airport_path = get_airport_db_path()?;

        let aircraft_connection = establish_database_connection(&aircraft_path)?;
        let airport_connection = establish_database_connection(&airport_path)?;

        Ok(Self { aircraft_connection, airport_connection })
    }
}

pub struct DatabasePool {
    pub aircraft_pool: Pool<ConnectionManager<SqliteConnection>>,
    pub airport_pool: Pool<ConnectionManager<SqliteConnection>>,
}

impl DatabasePool {
    pub fn new() -> Result<Self, Error> {
        fn establish_database_pool(
            database_path: &Path,
        ) -> Result<Pool<ConnectionManager<SqliteConnection>>, Error> {
            let Some(path_str) = database_path.to_str() else {
                return Err(Error::InvalidPath(database_path.display().to_string()));
            };
            let manager = ConnectionManager::<SqliteConnection>::new(path_str);
            Pool::builder().build(manager).map_err(|e| Error::Other(std::io::Error::other(e.to_string())))
        }

        let aircraft_path = get_aircraft_db_path()?;
        let airport_path = get_airport_db_path()?;

        let aircraft_pool = establish_database_pool(&aircraft_path)?;
        let airport_pool = establish_database_pool(&airport_path)?;

        Ok(Self { aircraft_pool, airport_pool })
    }
}

impl Default for DatabasePool {
    fn default() -> Self {
    Self::new().expect("Failed to initialize database pool")
    }
}

impl DatabaseOperations for DatabasePool {}
