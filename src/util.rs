use crate::models::Airport;
use diesel::define_sql_function;

define_sql_function! {fn random() -> Text;}

#[must_use]
pub fn calculate_haversine_distance_nm(airport_1: &Airport, airport_2: &Airport) -> f64 {
    let earth_radius_nm = 3440.0;
    let lat1 = airport_1.Latitude.to_radians();
    let lon1 = airport_1.Longtitude.to_radians();
    let lat2 = airport_2.Latitude.to_radians();
    let lon2 = airport_2.Longtitude.to_radians();

    let lat = lat2 - lat1;
    let lon = lon2 - lon1;

    let a = (lat1.cos() * lat2.cos()).mul_add((lon / 2.0).sin().powi(2), (lat / 2.0).sin().powi(2));
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    (earth_radius_nm * c).round()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Airport;

    #[test]
    fn test_calculate_haversine_distance_nm_same_airport() {
        let airport = Airport {
            ID: 1,
            Name: "Test Airport".to_string(),
            ICAO: "TEST".to_string(),
            PrimaryID: None,
            Latitude: 52.0,
            Longtitude: 4.0,
            Elevation: 0,
            TransitionAltitude: None,
            TransitionLevel: None,
            SpeedLimit: None,
            SpeedLimitAltitude: None,
        };

        let distance = calculate_haversine_distance_nm(&airport, &airport);
        assert!((distance - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_calculate_haversine_distance_nm_different_airports() {
        let airport1 = Airport {
            ID: 1,
            Name: "Airport 1".to_string(),
            ICAO: "AIR1".to_string(),
            PrimaryID: None,
            Latitude: 52.0,
            Longtitude: 4.0,
            Elevation: 0,
            TransitionAltitude: None,
            TransitionLevel: None,
            SpeedLimit: None,
            SpeedLimitAltitude: None,
        };

        let airport2 = Airport {
            ID: 2,
            Name: "Airport 2".to_string(),
            ICAO: "AIR2".to_string(),
            PrimaryID: None,
            Latitude: 48.0,
            Longtitude: 2.0,
            Elevation: 0,
            TransitionAltitude: None,
            TransitionLevel: None,
            SpeedLimit: None,
            SpeedLimitAltitude: None,
        };

        let distance = calculate_haversine_distance_nm(&airport1, &airport2);
        println!("distance is: {distance}");
        assert!((distance - 252.0).abs() < 1e-10);
    }

    #[test]
    fn test_calculate_haversine_distance_nm_negative_coordinates() {
        let airport1 = Airport {
            ID: 1,
            Name: "Airport 1".to_string(),
            ICAO: "AIR1".to_string(),
            PrimaryID: None,
            Latitude: -34.0,
            Longtitude: -58.0,
            Elevation: 0,
            TransitionAltitude: None,
            TransitionLevel: None,
            SpeedLimit: None,
            SpeedLimitAltitude: None,
        };

        let airport2 = Airport {
            ID: 2,
            Name: "Airport 2".to_string(),
            ICAO: "AIR2".to_string(),
            PrimaryID: None,
            Latitude: 40.0,
            Longtitude: 2.0,
            Elevation: 0,
            TransitionAltitude: None,
            TransitionLevel: None,
            SpeedLimit: None,
            SpeedLimitAltitude: None,
        };

        let distance = calculate_haversine_distance_nm(&airport1, &airport2);
        println!("distance is: {distance}");

        assert!((distance - 5548.0).abs() < 1e-10);
    }
}
