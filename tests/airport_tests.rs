mod common;

use common::{create_test_aircraft, setup_test_db};
use flight_planner::errors::AirportSearchError;
use flight_planner::models::{Aircraft, Airport, Runway};
use flight_planner::modules::airport::*;
use flight_planner::traits::AirportOperations;
use std::collections::HashMap;
use std::sync::Arc;

#[cfg(feature = "gui")]
use flight_planner::models::airport::SpatialAirport;
#[cfg(feature = "gui")]
use rstar::RTree;

#[test]
fn test_get_random_airport() {
    let mut database_connections = setup_test_db();
    let airport = database_connections.get_random_airport().unwrap();
    assert!(!airport.ICAO.is_empty());
    assert!(!airport.Name.is_empty());
    assert!(airport.ID > 0);
}

#[test]
fn test_get_destination_airport() {
    let mut database_connections = setup_test_db();
    let aircraft = Aircraft {
        aircraft_range: 30,
        ..create_test_aircraft(1, "Boeing", "737-800", "B738")
    };

    let departure = database_connections.get_airport_by_icao("EHAM").unwrap();

    let destination = database_connections
        .get_destination_airport(&aircraft, &departure)
        .unwrap();
    assert_eq!(destination.ICAO, "EHRD");
}

#[test]
fn test_get_random_airport_for_aircraft() {
    let mut database_connections = setup_test_db();
    let aircraft = Aircraft {
        takeoff_distance: Some(6090),
        ..create_test_aircraft(1, "Boeing", "737-800", "B738")
    };
    let airport = database_connections
        .get_random_airport_for_aircraft(&aircraft)
        .unwrap();
    assert_eq!(airport.ICAO, "EHAM");
}

#[test]
fn test_get_runways_for_airport() {
    let mut database_connections = setup_test_db();
    let airport = database_connections.get_random_airport().unwrap();
    let runways = database_connections
        .get_runways_for_airport(&airport)
        .unwrap();
    assert_eq!(runways.len(), 2);
}

#[test]
fn test_get_destination_airport_with_suitable_runway() {
    let mut database_connections = setup_test_db();
    let departure = database_connections.get_airport_by_icao("EHAM").unwrap();
    let destination = database_connections
        .get_destination_airport_with_suitable_runway(&departure, 30, 2000)
        .unwrap();
    assert_eq!(destination.ICAO, "EHRD");
}

#[test]
fn test_get_airport_within_distance() {
    let mut database_connections = setup_test_db();
    let departure = database_connections.get_airport_by_icao("EHAM").unwrap();
    let destination = database_connections
        .get_airport_within_distance(&departure, 30)
        .unwrap();
    assert_eq!(destination.ICAO, "EHRD");
}

#[test]
fn test_get_airports() {
    let mut database_connections = setup_test_db();
    let airports = database_connections.get_airports().unwrap();
    assert_eq!(airports.len(), 3);
}

#[test]
fn test_get_airport_by_icao() {
    let mut database_connections = setup_test_db();
    let airport = database_connections.get_airport_by_icao("EHAM").unwrap();
    assert_eq!(airport.ICAO, "EHAM");
}

#[cfg(feature = "gui")]
#[test]
fn test_get_random_destination_airport_fast() {
    let mut database_connections = setup_test_db();
    let aircraft = create_test_aircraft(1, "Boeing", "737-800", "B738");
    let departure = database_connections.get_airport_by_icao("EHAM").unwrap();
    let departure_arc = Arc::new(departure);

    let airports = database_connections.get_airports().unwrap();
    let all_airports: Vec<flight_planner::models::airport::CachedAirport> = airports
        .into_iter()
        .map(|a| flight_planner::models::airport::CachedAirport::new(Arc::new(a)))
        .collect();
    let eham_runway_data = database_connections
        .get_runways_for_airport(&departure_arc)
        .unwrap();

    let arrival = database_connections.get_airport_by_icao("EHRD").unwrap();
    let ehrd_runway_data = database_connections
        .get_runways_for_airport(&arrival)
        .unwrap();

    let mut runway_map: HashMap<i32, Vec<Runway>> = HashMap::new();
    runway_map.insert(departure_arc.ID, eham_runway_data);
    runway_map.insert(arrival.ID, ehrd_runway_data);

    let spatial_airports = RTree::bulk_load(
        all_airports
            .iter()
            .map(|airport| {
                let longest_runway = runway_map
                    .get(&airport.inner.ID)
                    .and_then(|runways| runways.iter().map(|r| r.Length).max())
                    .unwrap_or(0);
                SpatialAirport {
                    airport: airport.clone(),
                    longest_runway_length: longest_runway,
                }
            })
            .collect(),
    );

    let mut rng = rand::rng();
    let departure_cached =
        flight_planner::models::airport::CachedAirport::new(departure_arc.clone());
    let candidate = get_random_destination_airport_fast(
        &aircraft,
        &departure_cached,
        Some(all_airports.as_slice()),
        &spatial_airports,
        &mut rng,
    );

    assert!(candidate.is_some());
    let destination_airport: &Airport = &candidate.unwrap().inner;
    assert_eq!(destination_airport.ICAO, "EHRD");
}

#[test]
fn test_format_airport() {
    let airport = Airport {
        ID: 1,
        Name: "Amsterdam Airport Schiphol".to_string(),
        ICAO: "EHAM".to_string(),
        PrimaryID: None,
        Latitude: 52.3086,
        Longtitude: 4.7639,
        Elevation: -11,
        TransitionAltitude: Some(10000),
        TransitionLevel: None,
        SpeedLimit: Some(230),
        SpeedLimitAltitude: Some(6000),
    };
    let formatted = format_airport(&airport);
    assert_eq!(
        formatted,
        "Amsterdam Airport Schiphol (EHAM), altitude: -11"
    );
}

#[test]
fn test_get_destination_airport_within_range() {
    let mut database_connections = setup_test_db();
    let aircraft = Aircraft {
        aircraft_range: 25,
        ..create_test_aircraft(1, "Boeing", "737-800", "B738")
    };
    let departure = database_connections.get_airport_by_icao("EHAM").unwrap();
    let destination = database_connections
        .get_destination_airport(&aircraft, &departure)
        .unwrap();
    assert_eq!(destination.ICAO, "EHRD");
}

#[test]
fn test_get_destination_airport_out_of_range() {
    let mut database_connections = setup_test_db();
    let aircraft = Aircraft {
        aircraft_range: 23,
        ..create_test_aircraft(1, "Boeing", "737-800", "B738")
    };
    let departure = database_connections.get_airport_by_icao("EHAM").unwrap();
    let destination = database_connections.get_destination_airport(&aircraft, &departure);
    assert!(matches!(destination, Err(AirportSearchError::NotFound)));
}

#[test]
fn test_get_destination_airport_no_suitable_runway() {
    let mut database_connections = setup_test_db();
    let aircraft = Aircraft {
        takeoff_distance: Some(4000),
        ..create_test_aircraft(1, "Boeing", "737-800", "B738")
    };
    let departure = database_connections.get_airport_by_icao("EHAM").unwrap();
    let destination = database_connections.get_destination_airport(&aircraft, &departure);
    assert!(matches!(destination, Err(AirportSearchError::NotFound)));
}

#[test]
fn test_get_destination_airport_all_suitable_runways() {
    let mut database_connections = setup_test_db();
    let aircraft = create_test_aircraft(1, "Boeing", "737-800", "B738");
    let departure = database_connections.get_airport_by_icao("EHAM").unwrap();
    let destination = database_connections
        .get_destination_airport(&aircraft, &departure)
        .unwrap();
    assert!(!destination.ICAO.is_empty());
}

#[test]
fn test_get_airport_with_suitable_runway_fast_unit() {
    let mut database_connections = setup_test_db();
    let aircraft = Aircraft {
        takeoff_distance: Some(1000),
        ..create_test_aircraft(1, "Boeing", "737-800", "B738")
    };

    let airports = database_connections.get_airports().unwrap();
    let all_airports: Vec<Arc<Airport>> = airports.into_iter().map(Arc::new).collect();

    let mut runway_map: HashMap<i32, Vec<Runway>> = HashMap::new();

    for airport in &all_airports {
        runway_map.insert(
            airport.ID,
            database_connections
                .get_runways_for_airport(airport)
                .unwrap(),
        );
    }

    let all_runways: HashMap<i32, Arc<Vec<Runway>>> = runway_map
        .into_iter()
        .map(|(id, runways)| (id, Arc::new(runways)))
        .collect();

    let rng = &mut rand::rng();
    let result = get_airport_with_suitable_runway_fast(&aircraft, &all_airports, &all_runways, rng);
    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(!result.ICAO.is_empty());
}

#[test]
fn test_get_airport_with_suitable_runway_fast_no_suitable() {
    let mut database_connections = setup_test_db();
    let aircraft = Aircraft {
        takeoff_distance: Some(100_000),
        ..create_test_aircraft(1, "Boeing", "737-800", "B738")
    };

    let airports = database_connections.get_airports().unwrap();
    let all_airports: Vec<Arc<Airport>> = airports.into_iter().map(Arc::new).collect();

    let mut runway_map: HashMap<i32, Vec<Runway>> = HashMap::new();

    for airport in &all_airports {
        runway_map.insert(
            airport.ID,
            database_connections
                .get_runways_for_airport(airport)
                .unwrap(),
        );
    }

    let rng = &mut rand::rng();
    let all_runways: HashMap<i32, Arc<Vec<Runway>>> = runway_map
        .into_iter()
        .map(|(id, runways)| (id, Arc::new(runways)))
        .collect();
    let airport =
        get_airport_with_suitable_runway_fast(&aircraft, &all_airports, &all_runways, rng);

    assert!(matches!(airport, Err(AirportSearchError::NotFound)));
}

#[test]
fn test_get_airport_with_suitable_runway_fast_missing_data() {
    let mut database_connections = setup_test_db();
    let aircraft = Aircraft {
        date_flown: None,
        takeoff_distance: Some(1000),
        ..create_test_aircraft(1, "Boeing", "737-800", "B738")
    };

    let airports = database_connections.get_airports().unwrap();
    let all_airports: Vec<Arc<Airport>> = airports.into_iter().map(Arc::new).collect();

    // Empty map simulates missing data
    let runways_bymap: HashMap<i32, Arc<Vec<Runway>>> = HashMap::new();
    let rng = &mut rand::rng();

    let result =
        get_airport_with_suitable_runway_fast(&aircraft, &all_airports, &runways_bymap, rng);
    assert!(matches!(result, Err(AirportSearchError::NotFound)));
}

#[test]
fn test_get_airport_with_suitable_runway_fast_empty_runways() {
    let mut database_connections = setup_test_db();
    let aircraft = Aircraft {
        date_flown: None,
        takeoff_distance: Some(1000),
        ..create_test_aircraft(1, "Boeing", "737-800", "B738")
    };

    let airports = database_connections.get_airports().unwrap();
    let all_airports: Vec<Arc<Airport>> = airports.into_iter().map(Arc::new).collect();

    let mut runway_map: HashMap<i32, Arc<Vec<Runway>>> = HashMap::new();
    for airport in &all_airports {
        runway_map.insert(airport.ID, Arc::new(Vec::new())); // Empty runway list
    }

    let rng = &mut rand::rng();
    let result = get_airport_with_suitable_runway_fast(&aircraft, &all_airports, &runway_map, rng);
    assert!(matches!(result, Err(AirportSearchError::NotFound)));
}
