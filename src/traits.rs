use crate::models::*;
use diesel::result::Error;

pub trait AircraftOperations {
    fn get_unflown_aircraft_count(&mut self) -> Result<i32, Error>;
    fn random_unflown_aircraft(&mut self) -> Result<Aircraft, Error>;
    fn get_all_aircraft(&mut self) -> Result<Vec<Aircraft>, Error>;
    fn update_aircraft(&mut self, record: &Aircraft) -> Result<(), Error>;
    fn random_aircraft(&mut self) -> Result<Aircraft, Error>;
    fn get_aircraft_by_id(&mut self, aircraft_id: i32) -> Result<Aircraft, Error>;
}

pub trait AirportOperations: AircraftOperations {
    fn get_random_airport(&mut self) -> Result<Airport, Error>;
    fn get_destination_airport(
        &mut self,
        aircraft: &Aircraft,
        departure: &Airport,
    ) -> Result<Airport, Error>
    where
        Self: AircraftOperations;
    fn get_random_airport_for_aircraft(&mut self, aircraft: &Aircraft) -> Result<Airport, Error>
    where
        Self: AircraftOperations;
    fn get_runways_for_airport(&mut self, airport: &Airport) -> Result<Vec<Runway>, Error>;
    fn get_destination_airport_with_suitable_runway(
        &mut self,
        departure: &Airport,
        max_distance_nm: i32,
        min_takeoff_distance_m: i32,
    ) -> Result<Airport, Error>;
    fn get_airport_within_distance(
        &mut self,
        departure: &Airport,
        max_distance_nm: i32,
    ) -> Result<Airport, Error>;
    fn get_airports(&mut self) -> Result<Vec<Airport>, Error>;
}

pub trait HistoryOperations {
    fn add_to_history(
        &mut self,
        departure: &Airport,
        arrival: &Airport,
        aircraft_record: &Aircraft,
    ) -> Result<(), Error>;
    fn get_history(&mut self) -> Result<Vec<History>, Error>;
}

pub trait DatabaseOperations: AircraftOperations + AirportOperations + HistoryOperations {}