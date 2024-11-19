use crate::models::*;
use crate::schema::history::dsl::*;
use diesel::prelude::*;
use diesel::result::Error;

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

pub fn add_to_history(
    connection: &mut SqliteConnection,
    departure: &Airport,
    arrival: &Airport,
    aircraft_record: &Aircraft,
) -> Result<(), Error> {
    let record = create_history(departure, arrival, aircraft_record);

    diesel::insert_into(history)
        .values(&record)
        .execute(connection)?;

    Ok(())
}

pub fn get_history(connection: &mut SqliteConnection) -> Result<Vec<History>, Error> {
    let records: Vec<History> = history.load(connection)?;

    Ok(records)
}
