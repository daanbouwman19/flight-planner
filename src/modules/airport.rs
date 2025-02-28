use crate::database::{DatabaseConnections, DatabasePool};
use crate::gui::SpatialAirport;
use crate::models::*;
use crate::schema::Airports::dsl::*;
use crate::traits::{AircraftOperations, AirportOperations};
use crate::util::calculate_haversine_distance_nm;
use diesel::prelude::*;
use rand::prelude::*;
use rstar::{RTree, AABB};
use std::collections::HashMap;
use std::sync::Arc;

define_sql_function! {fn random() -> Text }

const M_TO_FT: f64 = 3.28084;
const MAX_ATTEMPTS: usize = 10;

#[derive(Debug)]
pub enum AirportSearchError {
    NotFound,
    NoSuitableRunway,
    DistanceExceeded,
    Other(std::io::Error),
}

impl From<diesel::result::Error> for AirportSearchError {
    fn from(error: diesel::result::Error) -> Self {
        match error {
            diesel::result::Error::NotFound => AirportSearchError::NotFound,
            _ => AirportSearchError::Other(std::io::Error::new(
                std::io::ErrorKind::Other,
                error.to_string(),
            )),
        }
    }
}

impl From<std::io::Error> for AirportSearchError {
    fn from(error: std::io::Error) -> Self {
        AirportSearchError::Other(error)
    }
}

impl std::error::Error for AirportSearchError {}

impl std::fmt::Display for AirportSearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AirportSearchError::NotFound => write!(f, "Airport not found"),
            AirportSearchError::NoSuitableRunway => write!(f, "No suitable runway found"),
            AirportSearchError::DistanceExceeded => write!(f, "Distance exceeded"),
            AirportSearchError::Other(error) => write!(f, "Other error: {}", error),
        }
    }
}

impl AirportOperations for DatabaseConnections {
    fn get_random_airport(&mut self) -> Result<Airport, AirportSearchError> {
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
    ) -> Result<Airport, AirportSearchError> {
        get_destination_airport(self, aircraft, departure)
    }

    fn get_random_airport_for_aircraft(
        &mut self,
        aircraft: &Aircraft,
    ) -> Result<Airport, AirportSearchError> {
        let min_takeoff_distance_m = aircraft.takeoff_distance;
        if min_takeoff_distance_m.is_some() {
            get_random_airport_for_aircraft(&mut self.airport_connection, min_takeoff_distance_m)
        } else {
            self.get_random_airport()
        }
    }

    fn get_runways_for_airport(
        &mut self,
        airport: &Airport,
    ) -> Result<Vec<Runway>, AirportSearchError> {
        use crate::schema::Runways::dsl::*;

        let runways = Runways
            .filter(AirportID.eq(airport.ID))
            .load::<Runway>(&mut self.airport_connection)?;

        Ok(runways)
    }

    fn get_destination_airport_with_suitable_runway(
        &mut self,
        departure: &Airport,
        max_distance_nm: i32,
        min_takeoff_distance_m: i32,
    ) -> Result<Airport, AirportSearchError> {
        get_destination_airport_with_suitable_runway(
            &mut self.airport_connection,
            departure,
            max_distance_nm,
            min_takeoff_distance_m,
        )
    }

    fn get_airport_within_distance(
        &mut self,
        departure: &Airport,
        max_distance_nm: i32,
    ) -> Result<Airport, AirportSearchError> {
        get_airport_within_distance(&mut self.airport_connection, departure, max_distance_nm)
    }

    fn get_airports(&mut self) -> Result<Vec<Airport>, AirportSearchError> {
        let airports = Airports.load::<Airport>(&mut self.airport_connection)?;

        Ok(airports)
    }

    fn get_airport_by_icao(&mut self, icao: &str) -> Result<Airport, AirportSearchError> {
        get_airport_by_icao(&mut self.airport_connection, icao)
    }
}

impl AirportOperations for DatabasePool {
    fn get_random_airport(&mut self) -> Result<Airport, AirportSearchError> {
        let conn = &mut self.airport_pool.get().unwrap();
        let airport: Airport = Airports.order(random()).limit(1).get_result(conn)?;

        Ok(airport)
    }

    fn get_destination_airport(
        &mut self,
        aircraft: &Aircraft,
        departure: &Airport,
    ) -> Result<Airport, AirportSearchError>
    where
        Self: AircraftOperations,
    {
        get_destination_airport(self, aircraft, departure)
    }

    fn get_random_airport_for_aircraft(
        &mut self,
        aircraft: &Aircraft,
    ) -> Result<Airport, AirportSearchError> {
        let min_takeoff_distance_m = aircraft.takeoff_distance;
        if min_takeoff_distance_m.is_some() {
            get_random_airport_for_aircraft(
                &mut self.airport_pool.get().unwrap(),
                min_takeoff_distance_m,
            )
        } else {
            self.get_random_airport()
        }
    }

    fn get_runways_for_airport(
        &mut self,
        airport: &Airport,
    ) -> Result<Vec<Runway>, AirportSearchError> {
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
    ) -> Result<Airport, AirportSearchError> {
        get_destination_airport_with_suitable_runway(
            &mut self.airport_pool.get().unwrap(),
            departure,
            max_distance_nm,
            min_takeoff_distance_m,
        )
    }

    fn get_airport_within_distance(
        &mut self,
        departure: &Airport,
        max_distance_nm: i32,
    ) -> Result<Airport, AirportSearchError> {
        get_airport_within_distance(
            &mut self.airport_pool.get().unwrap(),
            departure,
            max_distance_nm,
        )
    }

    fn get_airports(&mut self) -> Result<Vec<Airport>, AirportSearchError> {
        let conn = &mut self.airport_pool.get().unwrap();
        let airports = Airports.load::<Airport>(conn)?;

        Ok(airports)
    }

    fn get_airport_by_icao(&mut self, icao: &str) -> Result<Airport, AirportSearchError> {
        get_airport_by_icao(&mut self.airport_pool.get().unwrap(), icao)
    }
}

pub fn format_airport(airport: &Airport) -> String {
    format!(
        "{} ({}), altitude: {}",
        airport.Name, airport.ICAO, airport.Elevation
    )
}

fn get_destination_airport<T: AirportOperations>(
    db: &mut T,
    aircraft: &Aircraft,
    departure: &Airport,
) -> Result<Airport, AirportSearchError> {
    let max_aircraft_range_nm = aircraft.aircraft_range;
    let min_takeoff_distance_m = aircraft.takeoff_distance;

    for _ in 0..MAX_ATTEMPTS {
        let airport = match min_takeoff_distance_m {
            Some(min_takeoff_distance) => db.get_destination_airport_with_suitable_runway(
                departure,
                max_aircraft_range_nm,
                min_takeoff_distance,
            ),
            None => db.get_airport_within_distance(departure, max_aircraft_range_nm),
        };

        match airport {
            Ok(airport) => return Ok(airport),
            Err(_) => {
                continue;
            }
        }
    }

    Err(AirportSearchError::NotFound)
}
fn get_destination_airport_with_suitable_runway(
    db: &mut SqliteConnection,
    departure: &Airport,
    max_distance_nm: i32,
    min_takeoff_distance_m: i32,
) -> Result<Airport, AirportSearchError> {
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
        .first::<Airport>(db)?;

    let distance = calculate_haversine_distance_nm(departure, &airport);

    if distance >= max_distance_nm {
        return Err(AirportSearchError::DistanceExceeded);
    }

    Ok(airport)
}

fn get_airport_within_distance(
    db: &mut SqliteConnection,
    departure: &Airport,
    max_distance_nm: i32,
) -> Result<Airport, AirportSearchError> {
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
        .first::<Airport>(db)?;

    let distance = calculate_haversine_distance_nm(departure, &airport);

    if distance >= max_distance_nm {
        return Err(AirportSearchError::DistanceExceeded);
    }

    Ok(airport)
}

fn get_random_airport_for_aircraft(
    db: &mut SqliteConnection,
    min_takeoff_distance_m: Option<i32>,
) -> Result<Airport, AirportSearchError> {
    use crate::schema::{Airports::dsl::*, Runways};

    if let Some(min_takeoff_distance) = min_takeoff_distance_m {
        let min_takeoff_distance_ft = (min_takeoff_distance as f64 * M_TO_FT) as i32;

        let airport = Airports
            .inner_join(Runways::table)
            .filter(Runways::Length.ge(min_takeoff_distance_ft))
            .select(Airports::all_columns())
            .distinct()
            .order(random())
            .first::<Airport>(db)?;

        Ok(airport)
    } else {
        Err(AirportSearchError::NoSuitableRunway)
    }
}

pub fn get_destination_airport_with_suitable_runway_fast<'a>(
    aircraft: &'a Aircraft,
    departure: &'a Airport,
    spatial_airports: &'a RTree<SpatialAirport>,
    runways_by_airport: &'a HashMap<i32, Arc<Vec<Runway>>>,
) -> Result<&'a Arc<Airport>, AirportSearchError> {
    let max_distance_nm = aircraft.aircraft_range;
    let search_radius_deg = max_distance_nm as f64 / 60.0;
    let takeoff_distance_ft: Option<i32> = aircraft
        .takeoff_distance
        .map(|d| (d as f64 * M_TO_FT) as i32);

    let min_point = [
        departure.Latitude - search_radius_deg,
        departure.Longtitude - search_radius_deg,
    ];
    let max_point = [
        departure.Latitude + search_radius_deg,
        departure.Longtitude + search_radius_deg,
    ];
    let search_envelope = AABB::from_corners(min_point, max_point);
    let candidate_airports = spatial_airports.locate_in_envelope(&search_envelope);

    let mut suitable_airports: Vec<&Arc<Airport>> = candidate_airports
        .filter_map(|spatial_airport| {
            let airport = &spatial_airport.airport;
            runways_by_airport.get(&airport.ID).and_then(|runways| {
                runways
                    .iter()
                    .max_by_key(|r| r.Length)
                    .and_then(|longest_runway| match takeoff_distance_ft {
                        Some(takeoff_distance) if longest_runway.Length < takeoff_distance => None,
                        _ => Some(airport),
                    })
            })
        })
        .collect();

    let mut rng = rand::rng();
    suitable_airports.shuffle(&mut rng);

    match suitable_airports.first().copied() {
        Some(airport) => Ok(airport),
        None => Err(AirportSearchError::NoSuitableRunway),
    }
}

fn get_airport_by_icao(
    db: &mut SqliteConnection,
    icao: &str,
) -> Result<Airport, AirportSearchError> {
    let airport = Airports.filter(ICAO.eq(icao)).first::<Airport>(db)?;

    Ok(airport)
}
