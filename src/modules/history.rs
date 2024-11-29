use diesel::prelude::*;
use diesel::result::Error;

use crate::models::*;
use crate::schema::history::dsl::*;
use crate::traits::HistoryOperations;
use crate::DatabaseConnections;
use crate::DatabasePool;

#[derive(Insertable)]
#[diesel(table_name = crate::schema::history)]
struct HistoryForm<'a> {
    date: String,
    departure_icao: &'a str,
    arrival_icao: &'a str,
    aircraft: i32,
}

fn create_history<'a>(
    departure: &'a Airport,
    arrival: &'a Airport,
    aircraft_record: &'a Aircraft,
) -> HistoryForm<'a> {
    let date_string = chrono::Local::now().format("%Y-%m-%d").to_string();

    HistoryForm {
        date: date_string,
        departure_icao: &departure.ICAO,
        arrival_icao: &arrival.ICAO,
        aircraft: aircraft_record.id,
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
        let records: Vec<History> = history.load(&mut self.aircraft_connection)?;

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
        let records: Vec<History> = history.load(conn)?;

        Ok(records)
    }
}
