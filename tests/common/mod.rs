use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use flight_planner::database::{DatabaseConnections, DatabasePool};
use rand::prelude::*;
use std::env;
use std::path::PathBuf;
use std::sync::Mutex;

// We export these structs so tests can use them directly
#[allow(dead_code)]
pub struct TestDbCleanup {
    pub aircraft_path: PathBuf,
    pub airport_path: PathBuf,
}

impl Drop for TestDbCleanup {
    fn drop(&mut self) {
        if self.aircraft_path.exists() {
            let _ = std::fs::remove_file(&self.aircraft_path);
        }
        if self.airport_path.exists() {
            let _ = std::fs::remove_file(&self.airport_path);
        }
    }
}

#[allow(dead_code)]
pub struct TempDir {
    pub path: PathBuf,
}

impl TempDir {
    #[allow(dead_code)]
    pub fn new(prefix: &str) -> Self {
        let mut rng = rand::rng();
        let suffix: u64 = rng.random();
        // Sanitize prefix to prevent path traversal or invalid characters
        let safe_prefix: String = prefix
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '_' || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        let name = format!("{}_{}", safe_prefix, suffix);

        // CodeQL fix: Taint tracking sees env::temp_dir() or env::current_dir() as tainted.
        // We use a relative path "target/test_tmp" which implicitly uses the current working directory (project root).
        // This avoids reading tainted environment variables like TMPDIR.
        let mut base = std::path::PathBuf::from("target");
        base.push("test_tmp");

        // Ensure we don't traverse up
        let path = base.join(name);

        if path.exists() {
            let _ = std::fs::remove_dir_all(&path);
        }
        std::fs::create_dir_all(&path).expect("Failed to create temp dir");
        Self { path }
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        if self.path.exists() {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }
}

#[allow(dead_code)]
pub struct TestPool {
    pub pool: DatabasePool,
    pub _keep_alive: (SqliteConnection, SqliteConnection),
}

impl std::ops::Deref for TestPool {
    type Target = DatabasePool;
    fn deref(&self) -> &Self::Target {
        &self.pool
    }
}

impl std::ops::DerefMut for TestPool {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.pool
    }
}

#[allow(dead_code)]
pub fn create_test_airport(id: i32, name: &str, icao: &str) -> flight_planner::models::Airport {
    flight_planner::models::Airport {
        ID: id,
        Name: name.to_string(),
        ICAO: icao.to_string(),
        Latitude: 52.0,  // Default used in history_tests
        Longtitude: 4.0, // Default used in history_tests
        Elevation: 0,
        ..Default::default()
    }
}

#[allow(dead_code)]
#[cfg(feature = "gui")]
pub fn create_test_spatial_airport(
    id: i32,
    lat: f64,
    lon: f64,
    runway_len: i32,
) -> flight_planner::models::airport::SpatialAirport {
    let mut airport = create_test_airport(id, &format!("Airport {}", id), &format!("APT{}", id));
    airport.Latitude = lat;
    airport.Longtitude = lon;

    flight_planner::models::airport::SpatialAirport {
        airport: flight_planner::models::airport::CachedAirport::new(
            std::sync::Arc::new(airport),
            runway_len,
        ),
    }
}

#[allow(dead_code)]
pub fn create_test_aircraft(
    id: i32,
    manufacturer: &str,
    variant: &str,
    icao: &str,
) -> flight_planner::models::Aircraft {
    flight_planner::models::Aircraft {
        id,
        manufacturer: manufacturer.to_string(),
        variant: variant.to_string(),
        icao_code: icao.to_string(),
        flown: 0,
        aircraft_range: 3000,
        category: "A".to_string(),
        cruise_speed: 450,
        date_flown: None,
        takeoff_distance: Some(2000),
    }
}

/// Creates a default `NewAircraft` for testing (a Boeing 737-800).
#[allow(dead_code)]
pub fn create_test_new_aircraft() -> flight_planner::models::NewAircraft {
    flight_planner::models::NewAircraft {
        manufacturer: "Boeing".to_string(),
        variant: "737-800".to_string(),
        icao_code: "B738".to_string(),
        flown: 0,
        aircraft_range: 3000,
        category: "A".to_string(),
        cruise_speed: 450,
        date_flown: None,
        takeoff_distance: Some(2000),
    }
}

#[allow(dead_code)]
pub fn create_test_runway(id: i32, airport_id: i32, ident: &str) -> flight_planner::models::Runway {
    flight_planner::models::Runway {
        ID: id,
        AirportID: airport_id,
        Ident: ident.to_string(),
        TrueHeading: 90.0,
        Length: 10000,
        Width: 45,
        Surface: "Asphalt".to_string(),
        Latitude: 52.0,
        Longtitude: 4.0,
        Elevation: 0,
    }
}

#[allow(dead_code)]
pub fn create_test_history(
    id: i32,
    aircraft_id: i32,
    departure_icao: &str,
    arrival_icao: &str,
    date: &str,
    distance: i32,
) -> flight_planner::models::History {
    flight_planner::models::History {
        id,
        aircraft: aircraft_id,
        departure_icao: departure_icao.to_string(),
        arrival_icao: arrival_icao.to_string(),
        date: date.to_string(),
        distance: Some(distance),
    }
}

#[allow(dead_code)]
pub fn create_history_schema(conn: &mut SqliteConnection) {
    conn.batch_execute(
        "
        CREATE TABLE history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            departure_icao TEXT NOT NULL,
            arrival_icao TEXT NOT NULL,
            aircraft INTEGER NOT NULL,
            date TEXT NOT NULL,
            distance INTEGER
        );
    ",
    )
    .expect("Failed to create history table");
}

#[allow(dead_code)]
pub fn create_aircraft_schema(conn: &mut SqliteConnection) {
    conn.batch_execute(
        "
        CREATE TABLE aircraft (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            manufacturer TEXT NOT NULL,
            variant TEXT NOT NULL,
            icao_code TEXT NOT NULL,
            flown INTEGER NOT NULL,
            aircraft_range INTEGER NOT NULL,
            category TEXT NOT NULL,
            cruise_speed INTEGER NOT NULL,
            date_flown TEXT,
            takeoff_distance INTEGER
        );
    ",
    )
    .expect("Failed to create aircraft table");
}

#[allow(dead_code)]
pub fn init_aircraft_db(conn: &mut SqliteConnection) {
    create_history_schema(conn);
    create_aircraft_schema(conn);
    conn.batch_execute("
        INSERT INTO aircraft (id, manufacturer, variant, icao_code, flown, aircraft_range, category, cruise_speed, date_flown, takeoff_distance)
        VALUES (1, 'Boeing', '737-800', 'B738', 0, 3000, 'A', 450, NULL, 2000);
    ").unwrap();
}

#[allow(dead_code)]
pub fn init_airport_db(conn: &mut SqliteConnection) {
    conn.batch_execute("
        CREATE TABLE Airports (
            ID INTEGER PRIMARY KEY AUTOINCREMENT,
            Name TEXT NOT NULL,
            ICAO TEXT NOT NULL,
            PrimaryID INTEGER,
            Latitude REAL NOT NULL,
            Longtitude REAL NOT NULL,
            Elevation INTEGER NOT NULL,
            TransitionAltitude INTEGER,
            TransitionLevel INTEGER,
            SpeedLimit INTEGER,
            SpeedLimitAltitude INTEGER
        );
        INSERT INTO Airports (ID, Name, ICAO, PrimaryID, Latitude, Longtitude, Elevation, TransitionAltitude, TransitionLevel, SpeedLimit, SpeedLimitAltitude)
        VALUES (1, 'Amsterdam Airport Schiphol', 'EHAM', NULL, 52.3086, 4.7639, -11, 10000, NULL, 230, 6000),
               (2, 'Rotterdam The Hague Airport', 'EHRD', NULL, 51.9561, 4.4397, -13, 5000, NULL, 180, 4000),
               (3, 'Eindhoven Airport', 'EHEH', NULL, 51.4581, 5.3917, 49, 6000, NULL, 200, 5000);
        CREATE TABLE Runways (
            ID INTEGER PRIMARY KEY AUTOINCREMENT,
            AirportID INTEGER NOT NULL,
            Ident TEXT NOT NULL,
            TrueHeading REAL NOT NULL,
            Length INTEGER NOT NULL,
            Width INTEGER NOT NULL,
            Surface TEXT NOT NULL,
            Latitude REAL NOT NULL,
            Longtitude REAL NOT NULL,
            Elevation INTEGER NOT NULL
        );
        INSERT INTO Runways (AirportID, Ident, TrueHeading, Length, Width, Surface, Latitude, Longtitude, Elevation)
        VALUES (1, '09', 92.0, 20000, 45, 'Asphalt', 52.3086, 4.7639, -11),
               (1, '18R', 184.0, 10000, 45, 'Asphalt', 52.3086, 4.7639, -11),
               (2, '06', 62.0, 10000, 45, 'Asphalt', 51.9561, 4.4397, -13),
               (2, '24', 242.0, 10000, 45, 'Asphalt', 51.9561, 4.4397, -13),
               (3, '03', 32.0, 10000, 45, 'Asphalt', 51.4581, 5.3917, 49),
               (3, '21', 212.0, 10000, 45, 'Asphalt', 51.4581, 5.3917, 49);
        CREATE TABLE metar_cache (
            station TEXT PRIMARY KEY NOT NULL,
            raw TEXT NOT NULL,
            flight_rules TEXT,
            observation_time TEXT,
            observation_dt TEXT,
            fetched_at TEXT NOT NULL
        );
    ").unwrap();
}

#[allow(dead_code)]
pub fn setup_test_pool_db() -> TestPool {
    let mut rng = rand::rng();
    let aircraft_db_url = format!(
        "file:memdb_aircraft_pool_{}?mode=memory&cache=shared",
        rng.random::<u64>()
    );
    let airport_db_url = format!(
        "file:memdb_airport_pool_{}?mode=memory&cache=shared",
        rng.random::<u64>()
    );

    let mut aircraft_conn = SqliteConnection::establish(&aircraft_db_url).unwrap();
    // Only execute the parts relevant to each DB
    init_aircraft_db(&mut aircraft_conn);

    let mut airport_conn = SqliteConnection::establish(&airport_db_url).unwrap();
    init_airport_db(&mut airport_conn);

    let pool = DatabasePool::new(Some(&aircraft_db_url), Some(&airport_db_url)).unwrap();

    TestPool {
        pool,
        _keep_alive: (aircraft_conn, airport_conn),
    }
}

#[allow(dead_code)]
pub fn setup_test_db() -> DatabaseConnections {
    let mut rng = rand::rng();
    let aircraft_url = format!(
        "file:memdb_aircraft_{}?mode=memory&cache=shared",
        rng.random::<u64>()
    );
    let airport_url = format!(
        "file:memdb_airport_{}?mode=memory&cache=shared",
        rng.random::<u64>()
    );

    let mut database_connections =
        DatabaseConnections::new(Some(&aircraft_url), Some(&airport_url)).unwrap();

    init_aircraft_db(&mut database_connections.aircraft_connection);
    init_airport_db(&mut database_connections.airport_connection);

    database_connections
}

#[allow(dead_code)]
pub static ENV_LOCK: Mutex<()> = Mutex::new(());

#[allow(dead_code)]
pub fn with_env_overrides<F, T>(overrides: Vec<(&str, Option<&str>)>, f: F) -> T
where
    F: FnOnce() -> T,
{
    struct RestoreGuard {
        original: Vec<(String, Option<String>)>,
    }

    impl Drop for RestoreGuard {
        fn drop(&mut self) {
            for (key, value) in &self.original {
                match value {
                    Some(val) => unsafe { env::set_var(key, val) },
                    None => unsafe { env::remove_var(key) },
                }
            }
        }
    }

    let _lock = ENV_LOCK.lock().expect("env mutex poisoned");

    let mut original = Vec::new();
    for (key, _) in &overrides {
        original.push((key.to_string(), env::var(key).ok()));
    }

    let guard = RestoreGuard { original };

    for (key, value) in overrides {
        match value {
            Some(val) => unsafe { env::set_var(key, val) },
            None => unsafe { env::remove_var(key) },
        }
    }

    let result = f();
    drop(guard);
    result
}
