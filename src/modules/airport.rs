use crate::database::{DatabaseConnections, DatabasePool};
use crate::errors::AirportSearchError;
use crate::models::airport::SpatialAirport;
use crate::models::{Aircraft, Airport, Runway};
use crate::schema::Airports::dsl::{Airports, ID, Latitude, Longtitude};
use crate::traits::{AircraftOperations, AirportOperations};
use crate::util::{calculate_haversine_distance_nm, random};
use diesel::prelude::*;
use rand::prelude::*;
use rstar::{AABB, RTree};
use std::collections::HashMap;
use std::sync::Arc;

const M_TO_FT: f64 = 3.28084;
const MAX_ATTEMPTS: usize = 50;

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
        use crate::schema::Runways::dsl::{AirportID, Runways};

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
        use crate::schema::Runways::dsl::{AirportID, Runways};
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

/// Formats an `Airport` struct into a human-readable string.
///
/// # Arguments
///
/// * `airport` - A reference to the `Airport` struct to format.
///
/// # Returns
///
/// A `String` containing the formatted airport details.
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

        if airport.is_ok() {
            return airport;
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

    let max_difference_degrees = f64::from(max_distance_nm) / 60.0;
    let min_lat = origin_lat - max_difference_degrees;
    let max_lat = origin_lat + max_difference_degrees;
    let min_lon = origin_lon - max_difference_degrees;
    let max_lon = origin_lon + max_difference_degrees;

    #[allow(clippy::cast_possible_truncation)]
    let min_takeoff_distance_ft: i32 = (f64::from(min_takeoff_distance_m) * M_TO_FT).round() as i32;

    let airport: Airport = Airports
        .inner_join(Runways::table)
        .filter(Latitude.ge(min_lat))
        .filter(Latitude.le(max_lat))
        .filter(Longtitude.ge(min_lon))
        .filter(Longtitude.le(max_lon))
        .filter(ID.ne(departure.ID))
        .filter(Runways::Length.ge(min_takeoff_distance_ft))
        .order(random())
        .select(Airports::all_columns())
        .first(db)?;

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

    let max_difference_degrees = f64::from(max_distance_nm) / 60.0;
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
    use crate::schema::{Airports::dsl::Airports, Runways};

    if let Some(min_takeoff_distance) = min_takeoff_distance_m {
        #[allow(clippy::cast_possible_truncation)]
        let min_takeoff_distance_ft = (f64::from(min_takeoff_distance) * M_TO_FT).round() as i32;

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

/// Finds suitable destination airports for a given aircraft and departure airport.
///
/// This function uses an R-tree for efficient spatial searching and a pre-computed
/// runway map to quickly identify airports that are within the aircraft's range
/// and have at least one runway long enough for takeoff.
///
/// # Arguments
///
/// * `aircraft` - The aircraft for which to find destinations.
/// * `departure` - The departure airport.
/// * `spatial_airports` - An R-tree of all airports for fast spatial queries.
/// * `runways_by_airport` - A map from airport ID to its runways.
///
/// # Returns
///
/// An iterator over suitable destination airports.
pub fn get_destination_airports_with_suitable_runway_fast<'a>(
    aircraft: &'a Aircraft,
    departure: &'a Arc<Airport>,
    spatial_airports: &'a RTree<SpatialAirport>,
    runways_by_airport: &'a HashMap<i32, Arc<Vec<Runway>>>,
) -> impl Iterator<Item = &'a Arc<Airport>> {
    let max_distance_nm = aircraft.aircraft_range;
    let search_radius_deg = f64::from(max_distance_nm) / 60.0;
    #[allow(clippy::cast_possible_truncation)]
    let takeoff_distance_ft: Option<i32> = aircraft
        .takeoff_distance
        .map(|d| (f64::from(d) * M_TO_FT).round() as i32);

    let min_point = [
        departure.Latitude - search_radius_deg,
        departure.Longtitude - search_radius_deg,
    ];
    let max_point = [
        departure.Latitude + search_radius_deg,
        departure.Longtitude + search_radius_deg,
    ];
    let search_envelope = AABB::from_corners(min_point, max_point);

    // Pre-filter by spatial envelope and collect into a vector to avoid repeated iterator overhead
    let candidate_airports: Vec<_> = spatial_airports
        .locate_in_envelope(&search_envelope)
        .filter_map(move |spatial_airport| {
            let airport = &spatial_airport.airport;
            if airport.ID == departure.ID {
                return None;
            }

            // Quick runway check using pre-computed data
            runways_by_airport.get(&airport.ID).and_then(|runways| {
                let has_suitable_runway = match takeoff_distance_ft {
                    Some(required_distance) => {
                        // Check if any runway meets the requirement
                        runways.iter().any(|r| r.Length >= required_distance)
                    }
                    None => !runways.is_empty(), // Any runway is fine
                };
                has_suitable_runway.then_some(airport)
            })
        })
        .collect();

    candidate_airports.into_iter()
}

fn get_airport_by_icao(
    db: &mut SqliteConnection,
    icao: &str,
) -> Result<Airport, AirportSearchError> {
    use crate::schema::Airports::dsl::ICAO;

    let airport = Airports.filter(ICAO.eq(icao)).first::<Airport>(db)?;
    Ok(airport)
}

pub fn get_airport_with_suitable_runway_fast<'a>(
    aircraft: &'a Aircraft,
    all_airports: &'a [Arc<Airport>],
    runways_by_airport: &'a HashMap<i32, Arc<Vec<Runway>>>,
) -> Result<Arc<Airport>, AirportSearchError> {
    let mut rng = rand::rng();
    for _ in 0..MAX_ATTEMPTS {
        let Some(airport) = all_airports.choose(&mut rng) else {
            continue;
        };

        let Some(runways) = runways_by_airport.get(&airport.ID) else {
            log::warn!("No runway data found for airport ID: {}", airport.ID);
            continue;
        };

        let Some(longest_runway) = runways.iter().max_by_key(|r| r.Length) else {
            log::warn!("Empty runway list for airport ID: {}", airport.ID);
            continue;
        };

        if aircraft.takeoff_distance.is_none_or(|takeoff_distance_m| {
            #[allow(clippy::cast_possible_truncation)]
            let required_distance_ft = (f64::from(takeoff_distance_m) * M_TO_FT).round() as i32;
            longest_runway.Length >= required_distance_ft
        }) {
            return Ok(Arc::clone(airport));
        }
    }
    Err(AirportSearchError::NotFound)
}
