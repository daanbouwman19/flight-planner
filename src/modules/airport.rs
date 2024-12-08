use std::collections::HashMap;

use diesel::prelude::*;
use diesel::result::Error;
use rand::seq::SliceRandom;

use crate::models::*;
use crate::schema::Airports::dsl::*;
use crate::traits::AirportOperations;
use crate::DatabaseConnections;
use crate::DatabasePool;

define_sql_function! {fn random() -> Text }

const EARTH_RADIUS_KM: f64 = 6371.0;
const KM_TO_NM: f64 = 0.53995680345572;
const M_TO_FT: f64 = 3.28084;

impl AirportOperations for DatabaseConnections {
    fn get_random_airport(&mut self) -> Result<Airport, Error> {
        let airport: Airport = Airports
            .order(random())
            .limit(1)
            .get_result(&mut self.airport_connection)?;

        Ok(airport)
    }

    fn get_destination_airport(
        &mut self,
        aircraft: &Aircraft,
        departure: &Airport,
    ) -> Result<Airport, Error> {
        let max_aircraft_range_nm = aircraft.aircraft_range;
        let min_takeoff_distance_m = aircraft.takeoff_distance;
        const MAX_ATTEMPTS: usize = 10;

        for _ in 0..MAX_ATTEMPTS {
            let airport = match min_takeoff_distance_m {
                Some(min_takeoff_distance) => self.get_destination_airport_with_suitable_runway(
                    departure,
                    max_aircraft_range_nm,
                    min_takeoff_distance,
                ),
                None => self.get_airport_within_distance(departure, max_aircraft_range_nm),
            };

            match airport {
                Ok(airport) => return Ok(airport),
                Err(Error::NotFound) => continue,
                Err(e) => return Err(e),
            }
        }

        Err(Error::NotFound)
    }

    fn get_random_airport_for_aircraft(&mut self, aircraft: &Aircraft) -> Result<Airport, Error> {
        use crate::schema::{Airports::dsl::*, Runways};
        let min_takeoff_distance_m = aircraft.takeoff_distance;

        match min_takeoff_distance_m {
            Some(min_takeoff_distance) => {
                let min_takeoff_distance_ft = (min_takeoff_distance as f64 * M_TO_FT) as i32;

                let airport = Airports
                    .inner_join(Runways::table)
                    .filter(Runways::Length.ge(min_takeoff_distance_ft))
                    .select(Airports::all_columns())
                    .distinct()
                    .order(random())
                    .first::<Airport>(&mut self.airport_connection)?;

                Ok(airport)
            }
            None => self.get_random_airport(),
        }
    }

    fn get_destination_airport_with_suitable_runway(
        &mut self,
        departure: &Airport,
        max_distance_nm: i32,
        min_takeoff_distance_m: i32,
    ) -> Result<Airport, Error> {
        use crate::schema::Runways;
        let origin_lat = departure.Latitude;
        let origin_lon = departure.Longtitude;

        let max_difference_degrees = (max_distance_nm as f64) / 60.0;
        let min_lat = origin_lat - max_difference_degrees;
        let max_lat = origin_lat + max_difference_degrees;
        let min_lon = origin_lon - max_difference_degrees;
        let max_lon = origin_lon + max_difference_degrees;

        let min_takeoff_distance_ft = (min_takeoff_distance_m as f64 * M_TO_FT) as i32;

        let airport = Airports
            .inner_join(Runways::table)
            .filter(Latitude.ge(min_lat))
            .filter(Latitude.le(max_lat))
            .filter(Longtitude.ge(min_lon))
            .filter(Longtitude.le(max_lon))
            .filter(ID.ne(departure.ID))
            .filter(Runways::Length.ge(min_takeoff_distance_ft))
            .order(random())
            .select(Airports::all_columns())
            .first::<Airport>(&mut self.airport_connection)?;

        if haversine_distance_nm(departure, &airport) > max_distance_nm {
            return Err(Error::NotFound);
        }

        Ok(airport)
    }

    fn get_airport_within_distance(
        &mut self,
        departure: &Airport,
        max_distance_nm: i32,
    ) -> Result<Airport, Error> {
        let origin_lat = departure.Latitude;
        let origin_lon = departure.Longtitude;

        let max_difference_degrees = (max_distance_nm as f64) / 60.0;
        let min_lat = origin_lat - max_difference_degrees;
        let max_lat = origin_lat + max_difference_degrees;
        let min_lon = origin_lon - max_difference_degrees;
        let max_lon = origin_lon + max_difference_degrees;

        let airport = Airports
            .filter(Latitude.ge(min_lat))
            .filter(Latitude.le(max_lat))
            .filter(Longtitude.ge(min_lon))
            .filter(Longtitude.le(max_lon))
            .filter(ID.ne(departure.ID))
            .order(random())
            .first::<Airport>(&mut self.airport_connection)?;

        if haversine_distance_nm(departure, &airport) >= max_distance_nm {
            return Err(Error::NotFound);
        }

        Ok(airport)
    }

    fn get_runways_for_airport(&mut self, airport: &Airport) -> Result<Vec<Runway>, Error> {
        use crate::schema::Runways::dsl::*;

        let runways = Runways
            .filter(AirportID.eq(airport.ID))
            .load::<Runway>(&mut self.airport_connection)?;

        Ok(runways)
    }

    fn get_airports(&mut self) -> Result<Vec<crate::models::Airport>, Error> {
        let airports = Airports.load::<Airport>(&mut self.airport_connection)?;

        Ok(airports)
    }
}

impl AirportOperations for DatabasePool {
    fn get_random_airport(&mut self) -> Result<Airport, Error> {
        let conn = &mut self.airport_pool.get().unwrap();
        let airport: Airport = Airports.order(random()).limit(1).get_result(conn)?;

        Ok(airport)
    }

    fn get_destination_airport(
        &mut self,
        aircraft: &Aircraft,
        departure: &Airport,
    ) -> Result<Airport, Error>
    where
        Self: crate::AircraftOperations,
    {
        let max_aircraft_range_nm = aircraft.aircraft_range;
        let min_takeoff_distance_m = aircraft.takeoff_distance;
        const MAX_ATTEMPTS: usize = 10;

        for _ in 0..MAX_ATTEMPTS {
            let airport = match min_takeoff_distance_m {
                Some(min_takeoff_distance) => self.get_destination_airport_with_suitable_runway(
                    departure,
                    max_aircraft_range_nm,
                    min_takeoff_distance,
                ),
                None => self.get_airport_within_distance(departure, max_aircraft_range_nm),
            };

            match airport {
                Ok(airport) => return Ok(airport),
                Err(Error::NotFound) => continue,
                Err(e) => return Err(e),
            }
        }

        Err(Error::NotFound)
    }

    fn get_random_airport_for_aircraft(&mut self, aircraft: &Aircraft) -> Result<Airport, Error>
    where
        Self: crate::AircraftOperations,
    {
        use crate::schema::{Airports::dsl::*, Runways};
        let conn = &mut self.airport_pool.get().unwrap();
        let min_takeoff_distance_m = aircraft.takeoff_distance;

        match min_takeoff_distance_m {
            Some(min_takeoff_distance) => {
                let min_takeoff_distance_ft = (min_takeoff_distance as f64 * M_TO_FT) as i32;

                let airport = Airports
                    .inner_join(Runways::table)
                    .filter(Runways::Length.ge(min_takeoff_distance_ft))
                    .select(Airports::all_columns())
                    .distinct()
                    .order(random())
                    .first::<Airport>(conn)?;

                Ok(airport)
            }
            None => self.get_random_airport(),
        }
    }

    fn get_runways_for_airport(&mut self, airport: &Airport) -> Result<Vec<Runway>, Error> {
        use crate::schema::Runways::dsl::*;
        let conn = &mut self.airport_pool.get().unwrap();

        let runways = Runways
            .filter(AirportID.eq(airport.ID))
            .load::<Runway>(conn)?;

        Ok(runways)
    }

    fn get_destination_airport_with_suitable_runway(
        &mut self,
        departure: &Airport,
        max_distance_nm: i32,
        min_takeoff_distance_m: i32,
    ) -> Result<Airport, Error> {
        use crate::schema::Runways;
        let conn = &mut self.airport_pool.get().unwrap();
        let origin_lat = departure.Latitude;
        let origin_lon = departure.Longtitude;

        let max_difference_degrees = (max_distance_nm as f64) / 60.0;
        let min_lat = origin_lat - max_difference_degrees;
        let max_lat = origin_lat + max_difference_degrees;
        let min_lon = origin_lon - max_difference_degrees;
        let max_lon = origin_lon + max_difference_degrees;

        let min_takeoff_distance_ft = (min_takeoff_distance_m as f64 * M_TO_FT) as i32;

        let airport = Airports
            .inner_join(Runways::table)
            .filter(Latitude.ge(min_lat))
            .filter(Latitude.le(max_lat))
            .filter(Longtitude.ge(min_lon))
            .filter(Longtitude.le(max_lon))
            .filter(ID.ne(departure.ID))
            .filter(Runways::Length.ge(min_takeoff_distance_ft))
            .order(random())
            .select(Airports::all_columns())
            .first::<Airport>(conn)?;

        if haversine_distance_nm(departure, &airport) > max_distance_nm {
            return Err(Error::NotFound);
        }

        Ok(airport)
    }

    fn get_airport_within_distance(
        &mut self,
        departure: &Airport,
        max_distance_nm: i32,
    ) -> Result<Airport, Error> {
        let conn = &mut self.airport_pool.get().unwrap();
        let origin_lat = departure.Latitude;
        let origin_lon = departure.Longtitude;

        let max_difference_degrees = (max_distance_nm as f64) / 60.0;
        let min_lat = origin_lat - max_difference_degrees;
        let max_lat = origin_lat + max_difference_degrees;
        let min_lon = origin_lon - max_difference_degrees;
        let max_lon = origin_lon + max_difference_degrees;

        let airport = Airports
            .filter(Latitude.ge(min_lat))
            .filter(Latitude.le(max_lat))
            .filter(Longtitude.ge(min_lon))
            .filter(Longtitude.le(max_lon))
            .filter(ID.ne(departure.ID))
            .order(random())
            .first::<Airport>(conn)?;

        if haversine_distance_nm(departure, &airport) >= max_distance_nm {
            return Err(Error::NotFound);
        }

        Ok(airport)
    }

    fn get_airports(&mut self) -> Result<Vec<Airport>, Error> {
        let conn = &mut self.airport_pool.get().unwrap();
        let airports = Airports.load::<Airport>(conn)?;

        Ok(airports)
    }
}

pub fn haversine_distance_nm(airport1: &Airport, airport2: &Airport) -> i32 {
    let lat1 = airport1.Latitude.to_radians();
    let lon1 = airport1.Longtitude.to_radians();
    let lat2 = airport2.Latitude.to_radians();
    let lon2 = airport2.Longtitude.to_radians();

    let dlat = lat2 - lat1;
    let dlon = lon2 - lon1;

    let a = (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    let distance_km = EARTH_RADIUS_KM * c;

    f64::round(distance_km * KM_TO_NM) as i32
}

pub fn format_airport(airport: &Airport) -> String {
    format!(
        "{} ({}), altitude: {}",
        airport.Name, airport.ICAO, airport.Elevation
    )
}

pub fn get_destination_airport_with_suitable_runway_fast(
    aircraft: &Aircraft,
    departure: &Airport,
    airports_by_grid: &HashMap<(i32, i32), Vec<&Airport>>,
    runways_by_airport: &HashMap<i32, Vec<Runway>>,
    grid_size: f64,
) -> Result<Airport, std::io::Error> {
    const TOLERANCE_FACTOR: f64 = 0.9;
    const CANDIDATE_CAPACITY: usize = 100;

    let max_distance_nm = aircraft.aircraft_range;

    let min_takeoff_distance_m = aircraft.takeoff_distance.unwrap_or(i32::MAX);
    let origin_lat = departure.Latitude;
    let origin_lon = departure.Longtitude;

    let max_difference_degrees = (max_distance_nm as f64) / 60.0;
    let min_lat_bin = ((origin_lat - max_difference_degrees) / grid_size).floor() as i32;
    let max_lat_bin = ((origin_lat + max_difference_degrees) / grid_size).floor() as i32;
    let min_lon_bin = ((origin_lon - max_difference_degrees) / grid_size).floor() as i32;
    let max_lon_bin = ((origin_lon + max_difference_degrees) / grid_size).floor() as i32;

    let min_takeoff_distance_ft =
        (min_takeoff_distance_m as f64 * M_TO_FT * TOLERANCE_FACTOR) as i32;
    let mut candidate_airports: Vec<&Airport> = Vec::with_capacity(CANDIDATE_CAPACITY);

    for lat_bin in min_lat_bin..=max_lat_bin {
        for lon_bin in min_lon_bin..=max_lon_bin {
            if let Some(airports) = airports_by_grid.get(&(lat_bin, lon_bin)) {
                candidate_airports.extend(
                    airports
                        .iter()
                        .filter(|&&airport| airport.ID != departure.ID),
                );
            }
        }
    }

    let mut rng = rand::thread_rng();
    candidate_airports.shuffle(&mut rng);

    for airport in candidate_airports {
        if let Some(suitable_runways) = runways_by_airport.get(&airport.ID) {
            if suitable_runways
                .iter()
                .any(|r| r.Length >= min_takeoff_distance_ft)
            {
                let distance_nm = haversine_distance_nm(departure, airport);
                if distance_nm <= max_distance_nm {
                    return Ok(airport.clone());
                }
            }
        }
    }

    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "No suitable airport found",
    ))
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::errors::ValidationError;

    impl DatabaseConnections {
        pub fn insert_airport(&mut self, record: &Airport) -> Result<(), ValidationError> {
            if record.Name.is_empty() || record.ICAO.is_empty() {
                return Err(ValidationError::InvalidData(
                    "Name and ICAO code cannot be empty".to_string(),
                ));
            }
            diesel::insert_into(Airports)
                .values(record)
                .execute(&mut self.airport_connection)
                .map_err(|e| ValidationError::DatabaseError(e.to_string()))?;

            Ok(())
        }
    }
}
