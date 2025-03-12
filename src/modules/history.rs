use diesel::prelude::*;
use diesel::result::Error;

use crate::database::{DatabaseConnections, DatabasePool};
use crate::models::{Aircraft, Airport, History, NewHistory};
use crate::schema::history::dsl::{history, id};
use crate::traits::HistoryOperations;

fn create_history(
    departure: &Airport,
    arrival: &Airport,
    aircraft_record: &Aircraft,
) -> NewHistory {
    let date_string = chrono::Local::now().format("%Y-%m-%d").to_string();

    NewHistory {
        departure_icao: departure.ICAO.clone(),
        arrival_icao: arrival.ICAO.clone(),
        aircraft: aircraft_record.id,
        date: date_string,
    }
}

impl HistoryOperations for DatabaseConnections {
    fn add_to_history(
        &mut self,
        departure: &Airport,
        arrival: &Airport,
        aircraft_record: &Aircraft,
    ) -> Result<(), Error> {
        let record = create_history(departure, arrival, aircraft_record);

        diesel::insert_into(history)
            .values(&record)
            .execute(&mut self.aircraft_connection)?;

        Ok(())
    }

    fn get_history(&mut self) -> Result<Vec<History>, Error> {
        let records: Vec<History> = history
            .order(id.desc())
            .load(&mut self.aircraft_connection)?;

        Ok(records)
    }
}

impl HistoryOperations for DatabasePool {
    fn add_to_history(
        &mut self,
        departure: &Airport,
        arrival: &Airport,
        aircraft_record: &Aircraft,
    ) -> Result<(), Error> {
        let conn = &mut self.aircraft_pool.get().unwrap();
        let record = create_history(departure, arrival, aircraft_record);

        diesel::insert_into(history).values(&record).execute(conn)?;

        Ok(())
    }

    fn get_history(&mut self) -> Result<Vec<History>, Error> {
        let conn = &mut self.aircraft_pool.get().unwrap();
        let records: Vec<History> = history.order(id.desc()).load(conn)?;

        Ok(records)
    }
}

#[cfg(test)]
mod tests {
    use crate::database::DatabaseConnections;
    use crate::models::{Aircraft, Airport};
    use crate::traits::HistoryOperations;
    use diesel::connection::SimpleConnection;
    use diesel::{Connection, SqliteConnection};

    fn setup_test_db() -> DatabaseConnections {
        let aircraft_connection = SqliteConnection::establish(":memory:").unwrap();
        let airport_connection = SqliteConnection::establish(":memory:").unwrap();

        let mut database_connections = DatabaseConnections {
            aircraft_connection,
            airport_connection,
        };

        database_connections
            .aircraft_connection
            .batch_execute(
                "
                CREATE TABLE history (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    departure_icao TEXT NOT NULL,
                    arrival_icao TEXT NOT NULL,
                    aircraft INTEGER NOT NULL,
                    date TEXT NOT NULL
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
                INSERT INTO aircraft (manufacturer, variant, icao_code, flown, aircraft_range, category, cruise_speed, date_flown, takeoff_distance)
                VALUES ('Boeing', '737-800', 'B738', 0, 3000, 'A', 450, '2024-12-10', 2000);
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
                       ('Rotterdam The Hague Airport', 'EHRD', NULL, 51.9561, 4.4397, -13, 5000, NULL, 180, 4000);
                ",
            )
            .expect("Failed to create test data");

        database_connections
    }

    #[test]
    fn test_add_to_history() {
        let mut database_connections = setup_test_db();
        let departure = Airport {
            ID: 1,
            Name: "Amsterdam Airport Schiphol".to_string(),
            ICAO: "EHAM".to_string(),
            PrimaryID: None,
            Latitude: 52.3086,
            Longtitude: 4.7639,
            Elevation: -11,
            TransitionAltitude: Some(10000),
            TransitionLevel: None,
            SpeedLimit: Some(230),
            SpeedLimitAltitude: Some(6000),
        };
        let arrival = Airport {
            ID: 2,
            Name: "Rotterdam The Hague Airport".to_string(),
            ICAO: "EHRD".to_string(),
            PrimaryID: None,
            Latitude: 51.9561,
            Longtitude: 4.4397,
            Elevation: -13,
            TransitionAltitude: Some(5000),
            TransitionLevel: None,
            SpeedLimit: Some(180),
            SpeedLimitAltitude: Some(4000),
        };
        let aircraft_record = Aircraft {
            id: 1,
            manufacturer: "Boeing".to_string(),
            variant: "737-800".to_string(),
            icao_code: "B738".to_string(),
            flown: 0,
            aircraft_range: 3000,
            category: "A".to_string(),
            cruise_speed: 450,
            date_flown: Some("2024-12-10".to_string()),
            takeoff_distance: Some(2000),
        };

        database_connections
            .add_to_history(&departure, &arrival, &aircraft_record)
            .unwrap();

        let history_records = database_connections.get_history().unwrap();
        assert_eq!(history_records.len(), 1);
        assert_eq!(history_records[0].departure_icao, "EHAM");
        assert_eq!(history_records[0].arrival_icao, "EHRD");
        assert_eq!(history_records[0].aircraft, 1);
    }

    #[test]
    fn test_get_history() {
        let mut database_connections = setup_test_db();
        let departure = Airport {
            ID: 1,
            Name: "Amsterdam Airport Schiphol".to_string(),
            ICAO: "EHAM".to_string(),
            PrimaryID: None,
            Latitude: 52.3086,
            Longtitude: 4.7639,
            Elevation: -11,
            TransitionAltitude: Some(10000),
            TransitionLevel: None,
            SpeedLimit: Some(230),
            SpeedLimitAltitude: Some(6000),
        };
        let arrival = Airport {
            ID: 2,
            Name: "Rotterdam The Hague Airport".to_string(),
            ICAO: "EHRD".to_string(),
            PrimaryID: None,
            Latitude: 51.9561,
            Longtitude: 4.4397,
            Elevation: -13,
            TransitionAltitude: Some(5000),
            TransitionLevel: None,
            SpeedLimit: Some(180),
            SpeedLimitAltitude: Some(4000),
        };
        let aircraft_record = Aircraft {
            id: 1,
            manufacturer: "Boeing".to_string(),
            variant: "737-800".to_string(),
            icao_code: "B738".to_string(),
            flown: 0,
            aircraft_range: 3000,
            category: "A".to_string(),
            cruise_speed: 450,
            date_flown: Some("2024-12-10".to_string()),
            takeoff_distance: Some(2000),
        };

        database_connections
            .add_to_history(&departure, &arrival, &aircraft_record)
            .unwrap();
        database_connections
            .add_to_history(&arrival, &departure, &aircraft_record)
            .unwrap();

        let history_records = database_connections.get_history().unwrap();
        assert_eq!(history_records.len(), 2);
        // The order is reversed because the history is ordered by id.desc.
        assert_eq!(history_records[0].departure_icao, "EHRD");
        assert_eq!(history_records[0].arrival_icao, "EHAM");
        assert_eq!(history_records[1].departure_icao, "EHAM");
        assert_eq!(history_records[1].arrival_icao, "EHRD");
    }
}
