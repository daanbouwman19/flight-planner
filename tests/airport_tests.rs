mod common;

use common::{create_test_aircraft, setup_test_db};
use flight_planner::database::DatabaseConnections;
use flight_planner::errors::AirportSearchError;
use flight_planner::models::{Aircraft, Airport, Runway};
use flight_planner::modules::airport::*;
use flight_planner::traits::AirportOperations;
use std::collections::HashMap;
use std::sync::Arc;

type RunwayMap = HashMap<i32, Arc<Vec<Runway>>>;

fn setup_airports_and_runways(
    database_connections: &mut DatabaseConnections,
) -> (Vec<Arc<Airport>>, RunwayMap) {
    let airports = database_connections
        .get_airports()
        .expect("Failed to get airports");
    let all_airports: Vec<Arc<Airport>> = airports.into_iter().map(Arc::new).collect();

    let all_runways: RunwayMap = all_airports
        .iter()
        .map(|airport| {
            let runways = database_connections
                .get_runways_for_airport(airport)
                .expect("Failed to get runways for airport");
            (airport.ID, Arc::new(runways))
        })
        .collect();

    (all_airports, all_runways)
}

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

    let all_airports: Vec<flight_planner::models::airport::CachedAirport> = airports
        .into_iter()
        .map(|a| {
            let longest_runway = runway_map
                .get(&a.ID)
                .and_then(|runways| runways.iter().map(|r| r.Length).max())
                .unwrap_or(0);
            flight_planner::models::airport::CachedAirport::new(Arc::new(a), longest_runway)
        })
        .collect();

    let spatial_airports = RTree::bulk_load(
        all_airports
            .iter()
            .map(|airport| SpatialAirport {
                airport: airport.clone(),
            })
            .collect(),
    );

    let suitable_airports_filtered: Vec<flight_planner::models::airport::CachedAirport> =
        all_airports
            .iter()
            .filter(|a| runway_map.contains_key(&a.inner.ID))
            .cloned()
            .collect();

    let mut rng = rand::rng();
    let departure_longest = runway_map
        .get(&departure_arc.ID)
        .and_then(|runways| runways.iter().map(|r| r.Length).max())
        .unwrap_or(0);
    let departure_cached = flight_planner::models::airport::CachedAirport::new(
        departure_arc.clone(),
        departure_longest,
    );
    let candidate = get_random_destination_airport_fast(
        &aircraft,
        &departure_cached,
        Some(suitable_airports_filtered.as_slice()),
        &spatial_airports,
        &mut rng,
    );

    assert!(candidate.is_some());
    let destination_airport: &Airport = &candidate.unwrap().inner;
    assert_eq!(destination_airport.ICAO, "EHRD");
}

fn create_simple_test_airport(name: &str, icao: &str, elevation: i32) -> Airport {
    Airport {
        ID: 1,
        Name: name.to_string(),
        ICAO: icao.to_string(),
        Elevation: elevation,
        ..Default::default()
    }
}

#[test]
fn test_format_airport_parameterized() {
    let cases = vec![
        (
            "Amsterdam Airport Schiphol",
            "EHAM",
            -11,
            "Amsterdam Airport Schiphol (EHAM), altitude: -11",
        ),
        ("Denver", "KDEN", 5434, "Denver (KDEN), altitude: 5434"),
        ("Sea Level", "TEST", 0, "Sea Level (TEST), altitude: 0"),
        ("", "VOID", 100, " (VOID), altitude: 100"),
    ];

    for (name, icao, elevation, expected) in cases {
        let airport = create_simple_test_airport(name, icao, elevation);
        assert_eq!(format_airport(&airport), expected);
    }
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
fn test_get_airport_with_suitable_runway_fast_parameterized() {
    let mut database_connections = setup_test_db();
    let (all_airports, all_runways) = setup_airports_and_runways(&mut database_connections);

    struct TestCase {
        description: &'static str,
        takeoff_distance: Option<i32>,
        should_succeed: bool,
    }

    let cases = vec![
        TestCase {
            description: "Suitable runway exists (1000m)",
            takeoff_distance: Some(1000),
            should_succeed: true,
        },
        TestCase {
            description: "No suitable runway exists (100,000m)",
            takeoff_distance: Some(100_000),
            should_succeed: false,
        },
    ];

    let mut rng = rand::rng();

    for case in cases {
        let aircraft = Aircraft {
            takeoff_distance: case.takeoff_distance,
            ..create_test_aircraft(1, "Boeing", "737-800", "B738")
        };

        let result =
            get_airport_with_suitable_runway_fast(&aircraft, &all_airports, &all_runways, &mut rng);

        if case.should_succeed {
            let airport = result.unwrap_or_else(|e| {
                panic!(
                    "Failed case: {}. Expected success, got error: {:?}",
                    case.description, e
                )
            });
            assert!(!airport.ICAO.is_empty(), "Airport ICAO should not be empty");
        } else {
            assert!(
                matches!(result, Err(AirportSearchError::NotFound)),
                "Failed case: {}. Expected NotFound, got: {:?}",
                case.description,
                result
            );
        }
    }
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
