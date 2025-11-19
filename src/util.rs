use crate::models::Airport;
use diesel::define_sql_function;

define_sql_function! {fn random() -> Text;}

/// The conversion factor from meters to feet.
pub const METERS_TO_FEET: f64 = 3.28084;

/// Calculates the great-circle distance between two airports using the haversine formula.
///
/// # Arguments
///
/// * `airport_1` - The first airport.
/// * `airport_2` - The second airport.
///
/// # Returns
///
/// The distance between the two airports in nautical miles, rounded to the nearest integer.
#[must_use]
pub fn calculate_haversine_distance_nm(airport_1: &Airport, airport_2: &Airport) -> i32 {
    let earth_radius_nm = 3440.0;
    let lat1 = airport_1.Latitude.to_radians();
    let lon1 = airport_1.Longtitude.to_radians();
    let lat2 = airport_2.Latitude.to_radians();
    let lon2 = airport_2.Longtitude.to_radians();

    let lat = lat2 - lat1;
    let lon = lon2 - lon1;

    let a = (lat1.cos() * lat2.cos()).mul_add((lon / 2.0).sin().powi(2), (lat / 2.0).sin().powi(2));
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    #[allow(clippy::cast_possible_truncation)]
    return (earth_radius_nm * c).round() as i32;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_haversine_distance_nm() {
        let airport1 = Airport {
            ICAO: "KSFO".to_string(),
            Name: "San Francisco International Airport".to_string(),
            Latitude: 37.62131,
            Longtitude: -122.37896,
            ..Default::default()
        };
        let airport2 = Airport {
            ICAO: "KJFK".to_string(),
            Name: "John F. Kennedy International Airport".to_string(),
            Latitude: 40.64131,
            Longtitude: -73.77814,
            ..Default::default()
        };
        assert_eq!(calculate_haversine_distance_nm(&airport1, &airport2), 2242);
    }
}
