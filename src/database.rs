use diesel::{prelude::*, r2d2::ConnectionManager};
use r2d2::Pool;
use std::path::PathBuf;

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
        Self::new(None, None).expect("Failed to initialize database connections")
    }
}

impl DatabaseOperations for DatabaseConnections {}

// Helper function to resolve database URL
fn get_db_url(
    url: Option<&str>,
    default_path_fn: fn() -> Result<PathBuf, Error>,
) -> Result<String, Error> {
    match url {
        Some(url) => Ok(url.to_string()),
        None => {
            let path = default_path_fn()?;
            path.to_str()
                .map(String::from)
                .ok_or_else(|| Error::InvalidPath(path.display().to_string()))
        }
    }
}

impl DatabaseConnections {
    pub fn new(aircraft_db_url: Option<&str>, airport_db_url: Option<&str>) -> Result<Self, Error> {
        let aircraft_url = get_db_url(aircraft_db_url, get_aircraft_db_path)?;
        let airport_url = get_db_url(airport_db_url, get_airport_db_path)?;

        let aircraft_connection = SqliteConnection::establish(&aircraft_url)?;
        let airport_connection = SqliteConnection::establish(&airport_url)?;

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
    pub fn new(aircraft_db_url: Option<&str>, airport_db_url: Option<&str>) -> Result<Self, Error> {
        let aircraft_url = get_db_url(aircraft_db_url, get_aircraft_db_path)?;
        let airport_url = get_db_url(airport_db_url, get_airport_db_path)?;

        let aircraft_manager = ConnectionManager::<SqliteConnection>::new(aircraft_url);
        let airport_manager = ConnectionManager::<SqliteConnection>::new(airport_url);

        let aircraft_pool = Pool::builder().build(aircraft_manager)?;
        let airport_pool = Pool::builder().build(airport_manager)?;

        Ok(Self {
            aircraft_pool,
            airport_pool,
        })
    }
}

impl Default for DatabasePool {
    fn default() -> Self {
        Self::new(None, None).expect("Failed to initialize database pool")
    }
}

impl DatabaseOperations for DatabasePool {}
