use diesel::prelude::*;
use diesel::result::Error;

use crate::models::*;
use crate::schema::aircraft::dsl::*;
use crate::traits::AircraftOperations;
use crate::DatabaseConnections;
use crate::DatabasePool;

define_sql_function! {fn random() -> Text}

impl AircraftOperations for DatabaseConnections {
    fn get_not_flown_count(&mut self) -> Result<i32, Error> {
        let count: i64 = aircraft
            .filter(flown.eq(0))
            .count()
            .get_result(&mut self.aircraft_connection)?;

        Ok(count as i32)
    }

    fn random_not_flown_aircraft(&mut self) -> Result<Aircraft, Error> {
        let record: Aircraft = aircraft
            .filter(flown.eq(0))
            .order(random())
            .limit(1)
            .get_result(&mut self.aircraft_connection)?;

        Ok(record)
    }

    fn get_all_aircraft(&mut self) -> Result<Vec<Aircraft>, Error> {
        let records: Vec<Aircraft> = aircraft.load(&mut self.aircraft_connection)?;

        Ok(records)
    }

    fn update_aircraft(&mut self, record: &Aircraft) -> Result<(), Error> {
        diesel::update(aircraft.find(record.id))
            .set(record)
            .execute(&mut self.aircraft_connection)?;

        Ok(())
    }

    fn random_aircraft(&mut self) -> Result<Aircraft, Error> {
        let record: Aircraft = aircraft
            .order(random())
            .limit(1)
            .get_result(&mut self.aircraft_connection)?;

        Ok(record)
    }

    fn get_aircraft_by_id(&mut self, aircraft_id: i32) -> Result<Aircraft, Error> {
        if aircraft_id < 1 {
            return Err(Error::NotFound);
        }

        let record: Aircraft = aircraft
            .find(aircraft_id)
            .get_result(&mut self.aircraft_connection)?;
        Ok(record)
    }
}

impl AircraftOperations for DatabasePool {
    fn get_not_flown_count(&mut self) -> Result<i32, Error> {
        let conn = &mut self.aircraft_pool.get().unwrap();
        let count: i64 = aircraft.filter(flown.eq(0)).count().get_result(conn)?;

        Ok(count as i32)
    }

    fn random_not_flown_aircraft(&mut self) -> Result<Aircraft, Error> {
        let conn = &mut self.aircraft_pool.get().unwrap();
        let record: Aircraft = aircraft
            .filter(flown.eq(0))
            .order(random())
            .limit(1)
            .get_result(conn)?;

        Ok(record)
    }

    fn get_all_aircraft(&mut self) -> Result<Vec<Aircraft>, Error> {
        let conn = &mut self.aircraft_pool.get().unwrap();
        let records: Vec<Aircraft> = aircraft.load(conn)?;

        Ok(records)
    }

    fn update_aircraft(&mut self, record: &Aircraft) -> Result<(), Error> {
        let conn = &mut self.aircraft_pool.get().unwrap();
        diesel::update(aircraft.find(record.id))
            .set(record)
            .execute(conn)?;

        Ok(())
    }

    fn random_aircraft(&mut self) -> Result<Aircraft, Error> {
        let conn = &mut self.aircraft_pool.get().unwrap();
        let record: Aircraft = aircraft.order(random()).limit(1).get_result(conn)?;

        Ok(record)
    }

    fn get_aircraft_by_id(&mut self, aircraft_id: i32) -> Result<Aircraft, Error> {
        if aircraft_id < 1 {
            return Err(Error::NotFound);
        }

        let conn = &mut self.aircraft_pool.get().unwrap();
        let record: Aircraft = aircraft.find(aircraft_id).get_result(conn)?;
        Ok(record)
    }
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
