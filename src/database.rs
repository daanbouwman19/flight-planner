use diesel::{prelude::*, r2d2::ConnectionManager};
use r2d2::Pool;
use std::path::PathBuf;

use crate::{errors::Error, traits::DatabaseOperations};

/// Get the path to the aircraft database file in the application data directory
///
/// This function constructs the path to the `data.db` file, which is expected
/// to be in the application's data directory.
///
/// # Returns
///
/// A `Result` containing the `PathBuf` to the database file on success,
/// or an `Error` if the application data directory cannot be determined.
pub fn get_aircraft_db_path() -> Result<PathBuf, Error> {
    let base = crate::get_app_data_dir()?;
    Ok(base.join("data.db"))
}

/// Get the path to the airports database file
///
/// This function now checks three locations in order:
/// 1. The application data directory (`~/.local/share/flight-planner/` on Linux).
/// 2. The system-wide shared data directory (`/usr/local/share/flight-planner/` on Linux).
/// 3. The current working directory (for backward compatibility).
///
/// # Returns
///
/// A `Result` containing the `PathBuf` to the airports database file on success,
/// or an `Error` if the application data directory cannot be determined.
pub fn get_airport_db_path() -> Result<PathBuf, Error> {
    // 1. Check application data directory
    let app_data_dir = crate::get_app_data_dir()?;
    let app_data_path = app_data_dir.join("airports.db3");
    if app_data_path.exists() {
        return Ok(app_data_path);
    }

    // 2. Check system-wide shared data directory
    if let Ok(shared_dir) = get_install_shared_data_dir() {
        let shared_path = shared_dir.join("airports.db3");
        if shared_path.exists() {
            return Ok(shared_path);
        }
    }

    // 3. Fallback to current working directory
    Ok(PathBuf::from("airports.db3"))
}

/// Get the path to the installation shared data directory.
///
/// This function determines the directory where shared data files, such as
/// `aircrafts.csv`, are expected to be located. The resolution logic is
/// platform-dependent.
///
/// ## Non-Windows Resolution Order
/// 1. `FLIGHT_PLANNER_SHARE_DIR` environment variable (full path).
/// 2. `FLIGHT_PLANNER_PREFIX` environment variable + `/share/flight-planner`.
/// 3. Compile-time `INSTALL_PREFIX` (from `build.rs`) + `/share/flight-planner`.
/// 4. Default: `/usr/local/share/flight-planner`.
///
/// This function is best-effort and side-effect free.
///
/// # Returns
///
/// A `Result` containing the `PathBuf` to the shared data directory on success,
/// or an `io::Error` on failure.
#[cfg(not(target_os = "windows"))]
pub fn get_install_shared_data_dir() -> Result<PathBuf, std::io::Error> {
    // 1) Full share dir override
    if let Ok(dir) = std::env::var("FLIGHT_PLANNER_SHARE_DIR") {
        return Ok(PathBuf::from(dir));
    }

    // 2) Prefix override via env
    if let Ok(prefix) = std::env::var("FLIGHT_PLANNER_PREFIX") {
        return Ok(PathBuf::from(prefix).join("share/flight-planner"));
    }

    // 3) Compile-time prefix from build.rs (INSTALL_PREFIX)
    if let Some(prefix) = option_env!("INSTALL_PREFIX") {
        return Ok(PathBuf::from(prefix).join("share/flight-planner"));
    }

    // 4) Fallback default
    Ok(PathBuf::from("/usr/local/share/flight-planner"))
}

/// Get the path to the installation shared data directory (Windows-specific).
///
/// On Windows, this function returns the directory containing the executable,
/// allowing `aircrafts.csv` to be found when placed alongside it.
/// This implementation now supports `FLIGHT_PLANNER_SHARE_DIR` for testing
/// and consistency with the non-Windows version.
///
/// # Returns
///
/// A `Result` containing the `PathBuf` to the shared data directory on success,
/// or an `io::Error` on failure.
#[cfg(target_os = "windows")]
pub fn get_install_shared_data_dir() -> Result<PathBuf, std::io::Error> {
    // 1) Share dir override via FLIGHT_PLANNER_SHARE_DIR (for testing and consistency)
    if let Ok(dir) = std::env::var("FLIGHT_PLANNER_SHARE_DIR") {
        return Ok(PathBuf::from(dir));
    }

    // 2) Directory of the executable
    match std::env::current_exe() {
        Ok(mut exe_path) => {
            if exe_path.pop() {
                Ok(exe_path)
            } else {
                Err(std::io::Error::other(
                    "failed to resolve parent directory for current executable",
                ))
            }
        }
        Err(err) => {
            let kind = err.kind();
            Err(std::io::Error::new(
                kind,
                format!("failed to resolve current executable path: {err}"),
            ))
        }
    }
}

/// A struct holding direct connections to the aircraft and airport databases.
///
/// This struct is used for operations that require a direct, persistent
/// connection, rather than one from a pool.
pub struct DatabaseConnections {
    /// A connection to the aircraft database.
    pub aircraft_connection: SqliteConnection,
    /// A connection to the airport database.
    pub airport_connection: SqliteConnection,
}

impl Default for DatabaseConnections {
    fn default() -> Self {
        Self::new(None, None).expect("Failed to initialize database connections")
    }
}

impl DatabaseOperations for DatabaseConnections {}

/// Helper function to resolve a database URL.
///
/// If a URL is provided, it is used directly. Otherwise, the `default_path_fn`
/// is called to determine the path to the database file.
///
/// # Arguments
///
/// * `url` - An optional string slice containing the database URL.
/// * `default_path_fn` - A function that returns the default path to the database.
///
/// # Returns
///
/// A `Result` containing the database URL as a `String` on success,
/// or an `Error` on failure.
pub fn get_db_url(
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
    /// Creates a new `DatabaseConnections` instance.
    ///
    /// This function establishes connections to the aircraft and airport databases.
    ///
    /// # Arguments
    ///
    /// * `aircraft_db_url` - An optional URL for the aircraft database. If `None`,
    ///   the default path is used.
    /// * `airport_db_url` - An optional URL for the airport database. If `None`,
    ///   the default path is used.
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `DatabaseConnections` instance on success,
    /// or an `Error` on failure.
    pub fn new(aircraft_db_url: Option<&str>, airport_db_url: Option<&str>) -> Result<Self, Error> {
        let aircraft_url = get_db_url(aircraft_db_url, get_aircraft_db_path)?;
        let airport_url = get_db_url(airport_db_url, get_airport_db_path)?;

        let mut aircraft_connection = SqliteConnection::establish(&aircraft_url)?;
        let mut airport_connection = SqliteConnection::establish(&airport_url)?;

        // Configure SQLite for concurrent access
        use diesel::connection::SimpleConnection;
        aircraft_connection.batch_execute(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA busy_timeout = 15000;
            PRAGMA synchronous = NORMAL;
        ",
        )?;
        airport_connection.batch_execute(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA busy_timeout = 15000;
            PRAGMA synchronous = NORMAL;
        ",
        )?;

        Ok(Self {
            aircraft_connection,
            airport_connection,
        })
    }
}

/// A struct for managing connection pools to the aircraft and airport databases.
///
/// This struct is cloneable and is designed to be shared across threads.
#[derive(Clone)]
pub struct DatabasePool {
    /// A connection pool for the aircraft database.
    pub aircraft_pool: Pool<ConnectionManager<SqliteConnection>>,
    /// A connection pool for the airport database.
    pub airport_pool: Pool<ConnectionManager<SqliteConnection>>,
}

impl DatabasePool {
    /// Creates a new `DatabasePool` instance.
    ///
    /// # Arguments
    ///
    /// * `aircraft_db_url` - An optional URL for the aircraft database. If `None`,
    ///   the default path is used.
    /// * `airport_db_url` - An optional URL for the airport database. If `None`,
    ///   the default path is used.
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `DatabasePool` instance on success,
    /// or an `Error` on failure.
    pub fn new(aircraft_db_url: Option<&str>, airport_db_url: Option<&str>) -> Result<Self, Error> {
        let aircraft_url = get_db_url(aircraft_db_url, get_aircraft_db_path)?;
        let airport_url = get_db_url(airport_db_url, get_airport_db_path)?;

        let aircraft_manager = ConnectionManager::<SqliteConnection>::new(aircraft_url);
        let airport_manager = ConnectionManager::<SqliteConnection>::new(airport_url);

        let aircraft_pool = Pool::builder()
            .connection_customizer(Box::new(SqliteConnectionCustomizer))
            .build(aircraft_manager)?;
        let airport_pool = Pool::builder()
            .connection_customizer(Box::new(SqliteConnectionCustomizer))
            .build(airport_manager)?;

        Ok(Self {
            aircraft_pool,
            airport_pool,
        })
    }
}

#[derive(Debug)]
struct SqliteConnectionCustomizer;

impl diesel::r2d2::CustomizeConnection<SqliteConnection, diesel::r2d2::Error>
    for SqliteConnectionCustomizer
{
    fn on_acquire(&self, conn: &mut SqliteConnection) -> Result<(), diesel::r2d2::Error> {
        use diesel::connection::SimpleConnection;
        conn.batch_execute(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA busy_timeout = 15000;
            PRAGMA synchronous = NORMAL;
        ",
        )
        .map_err(diesel::r2d2::Error::QueryError)
    }
}

impl Default for DatabasePool {
    fn default() -> Self {
        Self::new(None, None).expect("Failed to initialize database pool")
    }
}

impl DatabaseOperations for DatabasePool {}
