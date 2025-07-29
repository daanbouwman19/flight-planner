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
