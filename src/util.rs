use crate::models::Airport;
use diesel::define_sql_function;

define_sql_function! {fn random() -> Text;}

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
