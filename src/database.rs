use diesel::{prelude::*, r2d2::ConnectionManager};
use r2d2::Pool;

use crate::traits::DatabaseOperations;

pub const AIRCRAFT_DB_FILENAME: &str = "data.db";
pub const AIRPORT_DB_FILENAME: &str = "airports.db3";

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
        fn establish_database_connection(database_name: &str) -> SqliteConnection {
            SqliteConnection::establish(database_name).unwrap_or_else(|_| {
                panic!("Error connecting to {}", database_name);
            })
        }

        let aircraft_connection = establish_database_connection(AIRCRAFT_DB_FILENAME);
        let airport_connection = establish_database_connection(AIRPORT_DB_FILENAME);

        DatabaseConnections {
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
            database_name: &str,
        ) -> Pool<ConnectionManager<SqliteConnection>> {
            let manager = ConnectionManager::<SqliteConnection>::new(database_name);
            Pool::builder().build(manager).unwrap()
        }

        let aircraft_pool = establish_database_pool(AIRCRAFT_DB_FILENAME);
        let airport_pool = establish_database_pool(AIRPORT_DB_FILENAME);

        DatabasePool {
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
