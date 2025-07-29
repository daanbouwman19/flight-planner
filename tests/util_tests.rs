use flight_planner::models::Airport;
use flight_planner::util::*;

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
    assert!(distance == 0);
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
    assert!(distance == 252);
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

    assert!(distance == 5548);
}
