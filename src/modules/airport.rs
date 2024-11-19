
use diesel::prelude::*;
use diesel::result::Error;
use crate::models::*;
use crate::schema::Airports::dsl::*;

const EARTH_RADIUS_KM: f64 = 6371.0;
const KM_TO_NM: f64 = 0.53995680345572;

define_sql_function! {fn random() -> Text }

#[cfg(test)]
pub fn insert_airport(
    connection: &mut SqliteConnection,
    record: &Airport,
) -> Result<(), Error> {
    diesel::insert_into(Airports)
        .values(record)
        .execute(connection)?;

    Ok(())
}

pub fn get_random_airport(
    connection: &mut SqliteConnection,
) -> Result<Airport, Error> {
    let airport: Airport = Airports.order(random()).limit(1).get_result(connection)?;
    Ok(airport)
}

pub fn get_destination_airport(
    connection: &mut SqliteConnection,
    aircraft: &Aircraft,
    departure: &Airport,
) -> Result<Airport, Error> {
    let max_aircraft_range_nm = aircraft.aircraft_range;
    let origin_lat = departure.Latitude;
    let origin_lon = departure.Longtitude;

    let max_difference_degrees = (max_aircraft_range_nm as f64) / 60.0;
    let min_lat = origin_lat - max_difference_degrees;
    let max_lat = origin_lat + max_difference_degrees;
    let min_lon = origin_lon - max_difference_degrees;
    let max_lon = origin_lon + max_difference_degrees;

    let airport: Airport = Airports
        .filter(Latitude.between(min_lat, max_lat))
        .filter(Longtitude.between(min_lon, max_lon))
        .filter(ID.ne(departure.ID))
        .order(random())
        .get_result(connection)?;

    let distance = haversine_distance_nm(departure, &airport);
    if distance > aircraft.aircraft_range {
        return get_destination_airport(connection, aircraft, departure);
    }

    Ok(airport)
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

// pub fn get_random_airport_for_aircraft(
//     &self,
//     _aircraft: &Aircaft,
// ) -> Result<Airport, sqlite::Error> {
//     let query = "SELECT * FROM `Airports` ORDER BY RANDOM() LIMIT 1";

//     let mut stmt = self.connection.prepare(query)?;

//     let mut cursor = stmt.iter();

//     if let Some(result) = cursor.next() {
//         let row = result?;
//         let airport = Airport {
//             id: row.read::<i64, _>("ID"),
//             name: row.read::<&str, _>("Name").to_string(),
//             icao_code: row.read::<&str, _>("ICAO").to_string(),
//             latitude: row.read::<f64, _>("Latitude"),
//             longtitude: row.read::<f64, _>("Longtitude"),
//             elevation: row.read::<i64, _>("Elevation"),
//             runways: self.create_runway_vec(row.read::<i64, _>("ID")),
//         };

//         return Ok(airport);
//     }

//     Err(sqlite::Error {
//         code: Some(sqlite::ffi::SQLITE_ERROR as isize),
//         message: Some("No rows returned".to_string()),
//     })
// }
