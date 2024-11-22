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
        takeoff_distance: record.takeoff_distance,
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

pub fn random_aircraft(connection: &mut SqliteConnection) -> Result<Aircraft, Error> {
    let record: Aircraft = aircraft.order(random()).limit(1).get_result(connection)?;

    Ok(record)
}

pub fn get_aircraft_by_id(
    connection: &mut SqliteConnection,
    aircraft_id: i32,
) -> Result<Aircraft, Error> {
    let record: Aircraft = aircraft.find(aircraft_id).get_result(connection)?;

    Ok(record)
}

pub fn format_aircraft(ac: &Aircraft) -> String {
    format!(
        "id: {}, {} {}{}, range: {}, category: {}, cruise speed: {} knots, takeoff distance: {}",
        ac.id,
        ac.manufacturer,
        ac.variant,
        if ac.icao_code.is_empty() {
            "".to_string()
        } else {
            format!(" ({})", ac.icao_code)
        },
        ac.aircraft_range,
        ac.category,
        ac.cruise_speed,
        ac.takeoff_distance
            .map_or("unknown".to_string(), |d| format!("{} m", d)),
    )
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
    takeoff_distance: Option<i32>,
}
