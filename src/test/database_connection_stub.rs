use crate::traits::*;

pub struct DatabaseConnectionsStub {
    
}

impl DatabaseOperations for DatabaseConnectionsStub {}

impl AircraftOperations for DatabaseConnectionsStub {
    fn get_unflown_aircraft_count(&mut self) -> Result<i32, diesel::result::Error> {
        todo!()
    }

    fn random_unflown_aircraft(&mut self) -> Result<crate::models::Aircraft, diesel::result::Error> {
        todo!()
    }

    fn get_all_aircraft(&mut self) -> Result<Vec<crate::models::Aircraft>, diesel::result::Error> {
        todo!()
    }

    fn update_aircraft(&mut self, record: &crate::models::Aircraft) -> Result<(), diesel::result::Error> {
        todo!()
    }

    fn random_aircraft(&mut self) -> Result<crate::models::Aircraft, diesel::result::Error> {
        todo!()
    }

    fn get_aircraft_by_id(&mut self, aircraft_id: i32) -> Result<crate::models::Aircraft, diesel::result::Error> {
        todo!()
    }
}

impl AirportOperations for DatabaseConnectionsStub {
    fn get_random_airport(&mut self) -> Result<crate::models::Airport, diesel::result::Error> {
        todo!()
    }

    fn get_destination_airport(
        &mut self,
        aircraft: &crate::models::Aircraft,
        departure: &crate::models::Airport,
    ) -> Result<crate::models::Airport, diesel::result::Error>
    where
        Self: AircraftOperations {
        todo!()
    }

    fn get_random_airport_for_aircraft(&mut self, aircraft: &crate::models::Aircraft) -> Result<crate::models::Airport, diesel::result::Error>
    where
        Self: AircraftOperations {
        todo!()
    }

    fn get_runways_for_airport(&mut self, airport: &crate::models::Airport) -> Result<Vec<crate::models::Runway>, diesel::result::Error> {
        todo!()
    }

    fn get_destination_airport_with_suitable_runway(
        &mut self,
        departure: &crate::models::Airport,
        max_distance_nm: i32,
        min_takeoff_distance_m: i32,
    ) -> Result<crate::models::Airport, diesel::result::Error> {
        todo!()
    }

    fn get_airport_within_distance(
        &mut self,
        departure: &crate::models::Airport,
        max_distance_nm: i32,
    ) -> Result<crate::models::Airport, diesel::result::Error> {
        todo!()
    }
}

impl HistoryOperations for DatabaseConnectionsStub {
    fn add_to_history(
        &mut self,
        departure: &crate::models::Airport,
        arrival: &crate::models::Airport,
        aircraft_record: &crate::models::Aircraft,
    ) -> Result<(), diesel::result::Error> {
        todo!()
    }

    fn get_history(&mut self) -> Result<Vec<crate::models::History>, diesel::result::Error> {
        todo!()
    }
}