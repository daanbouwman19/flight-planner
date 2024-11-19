
use diesel::prelude::*;
use diesel::result::Error;
use crate::models::*;
use crate::schema::history::dsl::*;

pub fn add_to_history(
    connection: &mut SqliteConnection,
    departure: &Airport,
    arrival: &Airport,
    aircraft_record: &Aircraft,
) -> Result<(), Error> {
    let date_string = chrono::Local::now().format("%Y-%m-%d").to_string();
    diesel::insert_into(history)
        .values((
            departure_icao.eq(&departure.ICAO),
            arrival_icao.eq(&arrival.ICAO),
            aircraft.eq(&aircraft_record.id),
            date.eq(&date_string),
        ))
        .execute(connection)?;
    Ok(())
}

pub fn get_history(connection: &mut SqliteConnection) -> Result<Vec<History>, Error> {
    let records: Vec<History> = history.load(connection)?;
    Ok(records)
}