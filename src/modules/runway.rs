use diesel::RunQueryDsl;

use crate::database::DatabasePool;
use crate::models::Runway;

impl DatabasePool {
    pub fn get_runways(&self) -> Result<Vec<Runway>, diesel::result::Error> {
        use crate::schema::Runways::dsl::Runways;
        let conn = &mut self.airport_pool.get().unwrap();

        let records: Vec<Runway> = Runways.get_results(conn)?;

        Ok(records)
    }
}

pub fn format_runway(runway: &Runway) -> String {
    format!(
        "Runway: {}, heading: {:.2}, length: {} ft, width: {} ft, surface: {}, elevation: {}ft",
        runway.Ident,
        runway.TrueHeading,
        runway.Length,
        runway.Width,
        runway.Surface,
        runway.Elevation
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    use diesel::connection::SimpleConnection;
    use diesel::r2d2::ConnectionManager;
    use diesel::SqliteConnection;
    use r2d2::Pool;

    fn setup_test_db() -> DatabasePool {
        let manager_aircraft = ConnectionManager::<SqliteConnection>::new(":memory:");
        let manager_airport = ConnectionManager::<SqliteConnection>::new(":memory:");
        let pool_aircraft = Pool::builder().build(manager_aircraft).unwrap();
        let pool_airport = Pool::builder().build(manager_airport).unwrap();

        let database_connections = DatabasePool {
            aircraft_pool: pool_aircraft,
            airport_pool: pool_airport,
        };

        database_connections
            .airport_pool
            .get()
            .unwrap()
            .batch_execute(
                "
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
                INSERT INTO Airports (Name, ICAO, PrimaryID, Latitude, Longtitude, Elevation, TransitionAltitude, TransitionLevel, SpeedLimit, SpeedLimitAltitude)
                VALUES ('Amsterdam Airport Schiphol', 'EHAM', NULL, 52.3086, 4.7639, -11, 10000, NULL, 230, 6000),
                       ('Rotterdam The Hague Airport', 'EHRD', NULL, 51.9561, 4.4397, -13, 5000, NULL, 180, 4000),
                       ('Eindhoven Airport', 'EHEH', NULL, 51.4581, 5.3917, 49, 6000, NULL, 200, 5000);
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
                ",
            )
            .expect("Failed to create test data");
        database_connections
    }

    #[test]
    fn test_get_runways() {
        let pool = setup_test_db();
        let runways = pool.get_runways().unwrap();
        assert_eq!(runways.len(), 6);
    }

    #[test]
    fn test_format_runway() {
        let runway = Runway {
            ID: 1,
            AirportID: 1,
            Ident: "09".to_string(),
            TrueHeading: 92.0,
            Length: 20000,
            Width: 45,
            Surface: "Asphalt".to_string(),
            Latitude: 52.3086,
            Longtitude: 4.7639,
            Elevation: -11,
        };
        let formatted = format_runway(&runway);
        assert_eq!(
            formatted,
            "Runway: 09, heading: 92.00, length: 20000 ft, width: 45 ft, surface: Asphalt, elevation: -11ft"
        );
    }
}
