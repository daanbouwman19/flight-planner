use crate::models::*;
use crate::schema::aircraft::dsl::*;
use diesel::prelude::*;
use diesel::result::Error;

define_sql_function! {fn random() -> Text }

#[cfg(test)]
pub fn insert_aircraft(connection: &mut SqliteConnection, record: &Aircraft) -> Result<(), Error> {
    let new_aircraft = AircraftForm {
        manufacturer: &record.manufacturer,
        variant: &record.variant,
        icao_code: &record.icao_code,
        flown: record.flown,
        aircraft_range: record.aircraft_range,
        category: &record.category,
        cruise_speed: record.cruise_speed,
        date_flown: record.date_flown.as_deref(),
    };

    diesel::insert_into(aircraft)
        .values(&new_aircraft)
        .execute(connection)?;

    Ok(())
}

pub fn get_unflown_aircraft_count(connection: &mut SqliteConnection) -> Result<i32, Error> {
    let count: i64 = aircraft
        .filter(flown.eq(0))
        .count()
        .get_result(connection)?;

    Ok(count as i32)
}

#[cfg(test)]
pub fn mark_all_aircraft_unflown(connection: &mut SqliteConnection) -> Result<(), Error> {
    diesel::update(aircraft)
        .set(flown.eq(0))
        .execute(connection)?;

    Ok(())
}

pub fn random_unflown_aircraft(connection: &mut SqliteConnection) -> Result<Aircraft, Error> {
    let record: Aircraft = aircraft
        .filter(flown.eq(0))
        .order(random())
        .limit(1)
        .get_result(connection)?;

    Ok(record)
}

pub fn get_all_aircraft(connection: &mut SqliteConnection) -> Result<Vec<Aircraft>, Error> {
    let records: Vec<Aircraft> = aircraft.load(connection)?;

    Ok(records)
}

pub fn update_aircraft(connection: &mut SqliteConnection, record: &Aircraft) -> Result<(), Error> {
    diesel::update(aircraft.find(record.id))
        .set(record)
        .execute(connection)?;

    Ok(())
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::aircraft)]
struct AircraftForm<'a> {
    manufacturer: &'a str,
    variant: &'a str,
    icao_code: &'a str,
    flown: i32,
    aircraft_range: i32,
    category: &'a str,
    cruise_speed: i32,
    date_flown: Option<&'a str>,
}
