use diesel::{prelude::*, r2d2::ConnectionManager};
use r2d2::Pool;
use std::path::{Path, PathBuf};

use crate::{errors::Error, traits::DatabaseOperations};

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

/// Get the path to the installation shared data directory.
///
/// Resolution order (non-Windows):
/// 1. FLIGHT_PLANNER_SHARE_DIR environment variable (full path)
/// 2. FLIGHT_PLANNER_PREFIX environment variable + "/share/flight-planner"
/// 3. Compile-time INSTALL_PREFIX (from build.rs) + "/share/flight-planner"
/// 4. Default: "/usr/local/share/flight-planner"
///
/// Best-effort and side-effect free.
pub(crate) fn get_install_shared_data_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        // On Windows we don't currently install shared data; return current dir
        PathBuf::from(".")
    }

    #[cfg(not(target_os = "windows"))]
    {
        // 1) Full share dir override
        if let Ok(dir) = std::env::var("FLIGHT_PLANNER_SHARE_DIR") {
            return PathBuf::from(dir);
        }

        // 2) Prefix override via env
        if let Ok(prefix) = std::env::var("FLIGHT_PLANNER_PREFIX") {
            return PathBuf::from(prefix).join("share/flight-planner");
        }

        // 3) Compile-time prefix from build.rs (INSTALL_PREFIX)
        if let Some(prefix) = option_env!("INSTALL_PREFIX") {
            return PathBuf::from(prefix).join("share/flight-planner");
        }

        // 4) Fallback default
        PathBuf::from("/usr/local/share/flight-planner")
    }
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
            SqliteConnection::establish(path_str).map_err(Error::from)
        }

        let aircraft_path = get_aircraft_db_path()?;
        let airport_path = get_airport_db_path()?;

        let aircraft_connection = establish_database_connection(&aircraft_path)?;
        let airport_connection = establish_database_connection(&airport_path)?;

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

impl DatabasePool {
    pub fn new() -> Result<Self, Error> {
        fn establish_database_pool(
            database_path: &Path,
        ) -> Result<Pool<ConnectionManager<SqliteConnection>>, Error> {
            let Some(path_str) = database_path.to_str() else {
                return Err(Error::InvalidPath(database_path.display().to_string()));
            };
            let manager = ConnectionManager::<SqliteConnection>::new(path_str);
            Pool::builder().build(manager).map_err(Error::from)
        }

        let aircraft_path = get_aircraft_db_path()?;
        let airport_path = get_airport_db_path()?;

        let aircraft_pool = establish_database_pool(&aircraft_path)?;
        let airport_pool = establish_database_pool(&airport_path)?;

        Ok(Self {
            aircraft_pool,
            airport_pool,
        })
    }
}

impl Default for DatabasePool {
    fn default() -> Self {
        Self::new().expect("Failed to initialize database pool")
    }
}

impl DatabaseOperations for DatabasePool {}
