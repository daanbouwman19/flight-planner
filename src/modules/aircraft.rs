use diesel::prelude::*;
use diesel::result::Error;

use crate::models::*;
use crate::schema::aircraft::dsl::*;
use crate::traits::AircraftOperations;
use crate::DatabaseConnections;

define_sql_function! {fn random() -> Text}

impl crate::DatabaseConnections {
    fn get_unflown_aircraft_count(&mut self) -> Result<i32, Error> {
        let count: i64 = aircraft
            .filter(flown.eq(0))
            .count()
            .get_result(&mut self.aircraft_connection)?;

        Ok(count as i32)
    }

    fn random_unflown_aircraft(&mut self) -> Result<Aircraft, Error> {
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

impl AircraftOperations for DatabaseConnections {
    fn get_unflown_aircraft_count(&mut self) -> Result<i32, Error> {
        self.get_unflown_aircraft_count()
    }

    fn random_unflown_aircraft(&mut self) -> Result<Aircraft, Error> {
        self.random_unflown_aircraft()
    }

    fn get_all_aircraft(&mut self) -> Result<Vec<Aircraft>, Error> {
        self.get_all_aircraft()
    }

    fn update_aircraft(&mut self, record: &Aircraft) -> Result<(), Error> {
        self.update_aircraft(record)
    }

    fn random_aircraft(&mut self) -> Result<Aircraft, Error> {
        self.random_aircraft()
    }

    fn get_aircraft_by_id(&mut self, aircraft_id: i32) -> Result<Aircraft, Error> {
        self.get_aircraft_by_id(aircraft_id)
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

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::errors::ValidationError;

    impl DatabaseConnections {
        pub fn insert_aircraft(&mut self, record: &Aircraft) -> Result<(), ValidationError> {
            if record.flown < 0 {
                return Err(ValidationError::InvalidData(
                    "Flown cannot be negative".to_string(),
                ));
            }

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
                .execute(&mut self.aircraft_connection)
                .map_err(|e| ValidationError::DatabaseError(e.to_string()))?;

            Ok(())
        }

        pub fn mark_all_aircraft_unflown(&mut self) -> Result<(), Error> {
            diesel::update(aircraft)
                .set(flown.eq(0))
                .execute(&mut self.aircraft_connection)?;

            Ok(())
        }
    }
}
