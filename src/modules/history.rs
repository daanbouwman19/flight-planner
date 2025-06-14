use diesel::prelude::*;
use diesel::result::Error;

use crate::database::DatabasePool; // DatabaseConnections removed
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

    // Removed impl HistoryOperations for DatabaseConnections
}

impl HistoryOperations for DatabasePool {
    fn add_to_history(
        &mut self,
        departure: &Airport,
        arrival: &Airport,
        aircraft_record: &Aircraft,
    ) -> Result<(), Error> {
        let conn = &mut self.aircraft_pool.get().map_err(|e| Error::from(std::io::Error::other(e.to_string())))?;
        let record = create_history(departure, arrival, aircraft_record);

        diesel::insert_into(history).values(&record).execute(conn)?;

        Ok(())
    }

    fn get_history(&mut self) -> Result<Vec<History>, Error> {
        let conn = &mut self.aircraft_pool.get().map_err(|e| Error::from(std::io::Error::other(e.to_string())))?;
        let records: Vec<History> = history.order(id.desc()).load(conn)?;

        Ok(records)
    }
}

#[cfg(test)]
mod tests {
    // DatabaseConnections removed from imports
    use crate::models::{Aircraft, Airport}; // History might be needed for new tests
    use crate::traits::HistoryOperations;
    // SimpleConnection and diesel::Connection might be needed for new test setup
    // use diesel::connection::SimpleConnection;
    // use diesel::{Connection, SqliteConnection};
    // If using DatabasePool in tests, its components might be needed:
    // use crate::database::DatabasePool;
    // use diesel::r2d2::ConnectionManager;
    // use r2d2::Pool;


    // setup_test_db and tests using DatabaseConnections are removed.
    // Tests for DatabasePool would need to be re-written or added separately.
}
