use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use flight_planner::database::{DatabaseConnections, DatabasePool};
use rand::Rng;
use std::path::PathBuf;

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
pub struct TestPool {
    pub pool: DatabasePool,
    pub _cleanup: TestDbCleanup,
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
pub fn setup_test_pool_db() -> TestPool {
    let mut rng = rand::rng();
    let aircraft_db_url = format!("test_aircraft_pooled_{}.db", rng.random::<u64>());
    let airport_db_url = format!("test_airport_pooled_{}.db", rng.random::<u64>());

    // No need to remove file as we use unique names, but cleanup handles it on drop.

    let mut aircraft_conn = SqliteConnection::establish(&aircraft_db_url).unwrap();
    // Only execute the parts relevant to each DB
    aircraft_conn.batch_execute("
        CREATE TABLE history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            departure_icao TEXT NOT NULL,
            arrival_icao TEXT NOT NULL,
            aircraft INTEGER NOT NULL,
            date TEXT NOT NULL,
            distance INTEGER
        );
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
        INSERT INTO aircraft (id, manufacturer, variant, icao_code, flown, aircraft_range, category, cruise_speed, date_flown, takeoff_distance)
        VALUES (1, 'Boeing', '737-800', 'B738', 0, 3000, 'A', 450, NULL, 2000);
    ").unwrap();

    let mut airport_conn = SqliteConnection::establish(&airport_db_url).unwrap();
    airport_conn.batch_execute("
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
               (2, 'Rotterdam The Hague Airport', 'EHRD', NULL, 51.9561, 4.4397, -13, 5000, NULL, 180, 4000);
    ").unwrap();

    let pool = DatabasePool::new(Some(&aircraft_db_url), Some(&airport_db_url)).unwrap();

    TestPool {
        pool,
        _cleanup: TestDbCleanup {
            aircraft_path: PathBuf::from(aircraft_db_url),
            airport_path: PathBuf::from(airport_db_url),
        },
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

    database_connections.aircraft_connection.batch_execute("
        CREATE TABLE history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            departure_icao TEXT NOT NULL,
            arrival_icao TEXT NOT NULL,
            aircraft INTEGER NOT NULL,
            date TEXT NOT NULL,
            distance INTEGER
        );
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
        INSERT INTO aircraft (id, manufacturer, variant, icao_code, flown, aircraft_range, category, cruise_speed, date_flown, takeoff_distance)
        VALUES (1, 'Boeing', '737-800', 'B738', 0, 3000, 'A', 450, NULL, 2000);
    ").unwrap();

    database_connections.airport_connection.batch_execute("
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
               (2, 'Rotterdam The Hague Airport', 'EHRD', NULL, 51.9561, 4.4397, -13, 5000, NULL, 180, 4000);
    ").unwrap();

    database_connections
}
