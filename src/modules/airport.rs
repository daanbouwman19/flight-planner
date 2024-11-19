use crate::schema::Airports::dsl::*;
use crate::models::*;
use diesel::prelude::*;
use diesel::result::Error;

const EARTH_RADIUS_KM: f64 = 6371.0;
const KM_TO_NM: f64 = 0.53995680345572;
const M_TO_FT: f64 = 3.28084;

define_sql_function! {fn random() -> Text }

#[cfg(test)]
pub fn insert_airport(connection: &mut SqliteConnection, record: &Airport) -> Result<(), Error> {
    diesel::insert_into(Airports)
        .values(record)
        .execute(connection)?;

    Ok(())
}

pub fn get_random_airport(connection: &mut SqliteConnection) -> Result<Airport, Error> {
    let airport: Airport = Airports.order(random()).limit(1).get_result(connection)?;

    Ok(airport)
}

pub fn get_destination_airport(
    connection: &mut SqliteConnection,
    aircraft: &Aircraft,
    departure: &Airport,
) -> Result<Airport, Error> {
    let max_aircraft_range_nm = aircraft.aircraft_range;
    let min_takeoff_distance_m = aircraft.takeoff_distance;
    let origin_lat = departure.Latitude;
    let origin_lon = departure.Longtitude;

    let max_difference_degrees = (max_aircraft_range_nm as f64) / 60.0;
    let min_lat = origin_lat - max_difference_degrees;
    let max_lat = origin_lat + max_difference_degrees;
    let min_lon = origin_lon - max_difference_degrees;
    let max_lon = origin_lon + max_difference_degrees;

    const MAX_ATTEMPTS: usize = 10;

    for _ in 0..MAX_ATTEMPTS {
        match min_takeoff_distance_m {
            Some(min_takeoff_distance) => {
                let min_takeoff_distance_ft = (min_takeoff_distance as f64 * M_TO_FT) as i32;
                let airport = Airports
                    .filter(Latitude.ge(min_lat))
                    .filter(Latitude.le(max_lat))
                    .filter(Longtitude.ge(min_lon))
                    .filter(Longtitude.le(max_lon))
                    .filter(ID.ne(departure.ID))
                    .inner_join(crate::schema::Runways::table)
                    .filter(crate::schema::Runways::Length.ge(min_takeoff_distance_ft))
                    .select(Airports::all_columns())
                    .distinct()
                    .order(random())
                    .first::<Airport>(connection)?;

                let distance = haversine_distance_nm(departure, &airport);
                if distance > max_aircraft_range_nm {
                    continue;
                } else {
                    return Ok(airport);
                }
            }
            None => {
                let airport = Airports
                    .filter(Latitude.ge(min_lat))
                    .filter(Latitude.le(max_lat))
                    .filter(Longtitude.ge(min_lon))
                    .filter(Longtitude.le(max_lon))
                    .filter(ID.ne(departure.ID))
                    .order(random())
                    .first::<Airport>(connection)?;

                let distance = haversine_distance_nm(departure, &airport);
                if distance > max_aircraft_range_nm {
                    continue;
                } else {
                    return Ok(airport);
                }
            }
        }
    }

    Err(Error::NotFound)
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

pub fn get_random_airport_for_aircraft(
    connection: &mut SqliteConnection,
    aircraft: &Aircraft,
) -> Result<Airport, Error> {
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
                .first::<Airport>(connection)?;

            Ok(airport)
        }
        None => get_random_airport(connection),
    }
}
