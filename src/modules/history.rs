use diesel::prelude::*;
use diesel::result::Error;

use crate::database::{DatabaseConnections, DatabasePool};
use crate::models::{Aircraft, Airport, History, NewHistory};
use crate::schema::history::dsl::{history, id};
use crate::traits::{AircraftOperations, HistoryOperations};

/// Marks a flight route as completed by adding it to history and updating the aircraft.
/// This is a high-level operation that combines multiple database operations.
///
/// # Arguments
///
/// * `database_pool` - The database pool
/// * `departure` - The departure airport
/// * `arrival` - The arrival airport  
/// * `aircraft` - The aircraft used for the flight
///
/// # Returns
///
/// Returns a Result indicating success or failure.
pub fn mark_flight_completed(
    database_pool: &mut DatabasePool,
    departure: &Airport,
    arrival: &Airport,
    aircraft: &Aircraft,
) -> Result<(), Box<dyn std::error::Error>> {
    // Add route to history
    database_pool.add_to_history(departure, arrival, aircraft)?;

    // Update aircraft as flown
    let mut updated_aircraft = aircraft.clone();
    updated_aircraft.date_flown = Some(crate::date_utils::get_current_date_utc());
    updated_aircraft.flown = 1;

    database_pool.update_aircraft(&updated_aircraft)?;

    Ok(())
}

fn create_history(
    departure: &Airport,
    arrival: &Airport,
    aircraft_record: &Aircraft,
) -> NewHistory {
    let date_string = crate::date_utils::get_current_date_utc();
    let distance = crate::util::calculate_haversine_distance_nm(departure, arrival);

    NewHistory {
        departure_icao: departure.ICAO.clone(),
        arrival_icao: arrival.ICAO.clone(),
        aircraft: aircraft_record.id,
        date: date_string,
        distance: Some(distance),
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
