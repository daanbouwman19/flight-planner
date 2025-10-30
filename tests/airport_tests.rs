use diesel::connection::SimpleConnection;
use diesel::{Connection, SqliteConnection};
use flight_planner::database::DatabaseConnections;
use flight_planner::errors::AirportSearchError;
use flight_planner::models::airport::SpatialAirport;
use flight_planner::models::{Aircraft, Airport, Runway};
use flight_planner::modules::airport::*;
use flight_planner::traits::AirportOperations;
use rstar::RTree;
use std::collections::HashMap;
use std::sync::Arc;

fn setup_test_db() -> DatabaseConnections {
    let aircraft_connection = SqliteConnection::establish(":memory:").unwrap();
    let airport_connection = SqliteConnection::establish(":memory:").unwrap();

    let mut database_connections = DatabaseConnections {
        aircraft_connection,
        airport_connection,
    };

    database_connections
        .airport_connection
        .batch_execute(
            "
            CREATE TABLE Airports (
                ID INTEGER PRIMARY KEY AUTOINCREMENT,
                Name TEXT NOT NULL,
                ICAO TEXT NOT NULL,
                PrimaryID INTEGER,
                Latitude REAL NOT NULL,
                Longtitude REAL NOT NULL,
                Elevation INTEGER NOT NULL,
                TransitionAltitude INTEGER,
                TransitionLevel INTEGER,
                SpeedLimit INTEGER,
                SpeedLimitAltitude INTEGER
            );
            INSERT INTO Airports (Name, ICAO, PrimaryID, Latitude, Longtitude, Elevation, TransitionAltitude, TransitionLevel, SpeedLimit, SpeedLimitAltitude)
            VALUES ('Amsterdam Airport Schiphol', 'EHAM', NULL, 52.3086, 4.7639, -11, 10000, NULL, 230, 6000),
                   ('Rotterdam The Hague Airport', 'EHRD', NULL, 51.9561, 4.4397, -13, 5000, NULL, 180, 4000),
                   ('Eindhoven Airport', 'EHEH', NULL, 51.4581, 5.3917, 49, 6000, NULL, 200, 5000);
            CREATE TABLE Runways (
                ID INTEGER PRIMARY KEY AUTOINCREMENT,
                AirportID INTEGER NOT NULL,
                Ident TEXT NOT NULL,
                TrueHeading REAL NOT NULL,
                Length INTEGER NOT NULL,
                Width INTEGER NOT NULL,
                Surface TEXT NOT NULL,
                Latitude REAL NOT NULL,
                Longtitude REAL NOT NULL,
                Elevation INTEGER NOT NULL
            );
            INSERT INTO Runways (AirportID, Ident, TrueHeading, Length, Width, Surface, Latitude, Longtitude, Elevation)
            VALUES (1, '09', 92.0, 20000, 45, 'Asphalt', 52.3086, 4.7639, -11),
                   (1, '18R', 184.0, 10000, 45, 'Asphalt', 52.3086, 4.7639, -11),
                   (2, '06', 62.0, 10000, 45, 'Asphalt', 51.9561, 4.4397, -13),
                   (2, '24', 242.0, 10000, 45, 'Asphalt', 51.9561, 4.4397, -13),
                   (3, '03', 32.0, 10000, 45, 'Asphalt', 51.4581, 5.3917, 49),
                   (3, '21', 212.0, 10000, 45, 'Asphalt', 51.4581, 5.3917, 49);
            ",
        )
        .expect("Failed to create test data");

    database_connections
}

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
        id: 1,
        manufacturer: "Boeing".to_string(),
        variant: "737-800".to_string(),
        icao_code: "B738".to_string(),
        flown: 0,
        aircraft_range: 30,
        category: "A".to_string(),
        cruise_speed: 450,
        date_flown: Some("2024-12-10".to_string()),
        takeoff_distance: Some(2000),
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
        id: 1,
        manufacturer: "Boeing".to_string(),
        variant: "737-800".to_string(),
        icao_code: "B738".to_string(),
        flown: 0,
        aircraft_range: 3000,
        category: "A".to_string(),
        cruise_speed: 450,
        date_flown: Some("2024-12-10".to_string()),
        takeoff_distance: Some(6090),
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

#[test]
fn test_get_destination_airport_with_suitable_runway_fast() {
    let mut database_connections = setup_test_db();
    let aircraft = Aircraft {
        id: 1,
        manufacturer: "Boeing".to_string(),
        variant: "737-800".to_string(),
        icao_code: "B738".to_string(),
        flown: 0,
        aircraft_range: 3000,
        category: "A".to_string(),
        cruise_speed: 450,
        date_flown: Some("2024-12-10".to_string()),
        takeoff_distance: Some(2000),
    };
    let departure = database_connections.get_airport_by_icao("EHAM").unwrap();
    let departure_arc = Arc::new(departure);

    let airports = database_connections.get_airports().unwrap();
    let all_airports: Vec<Arc<Airport>> = airports.into_iter().map(Arc::new).collect();
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

    let all_runways: HashMap<i32, Arc<Vec<Runway>>> = runway_map
        .into_iter()
        .map(|(id, runways)| (id, Arc::new(runways)))
        .collect();

    let spatial_airports = RTree::bulk_load(
        all_airports
            .iter()
            .map(|airport| SpatialAirport {
                airport: Arc::clone(airport),
            })
            .collect(),
    );

    let candidates = get_destination_airports_with_suitable_runway_fast(
        &aircraft,
        &departure_arc,
        &spatial_airports,
        &all_runways,
    );
    // candidates is a Vec<&Arc<Airport>>; take the first candidate and assert
    let first_candidate = candidates
        .first()
        .expect("Should have at least one candidate");
    let destination_airport: &Airport = (*first_candidate).as_ref();
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
        id: 1,
        manufacturer: "Boeing".to_string(),
        variant: "737-800".to_string(),
        icao_code: "B738".to_string(),
        flown: 0,
        aircraft_range: 25,
        category: "A".to_string(),
        cruise_speed: 450,
        date_flown: Some("2024-12-10".to_string()),
        takeoff_distance: Some(2000),
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
        id: 1,
        manufacturer: "Boeing".to_string(),
        variant: "737-800".to_string(),
        icao_code: "B738".to_string(),
        flown: 0,
        aircraft_range: 23,
        category: "A".to_string(),
        cruise_speed: 450,
        date_flown: Some("2024-12-10".to_string()),
        takeoff_distance: Some(2000),
    };
    let departure = database_connections.get_airport_by_icao("EHAM").unwrap();
    let destination = database_connections.get_destination_airport(&aircraft, &departure);
    assert!(matches!(destination, Err(AirportSearchError::NotFound)));
}

#[test]
fn test_get_destination_airport_no_suitable_runway() {
    let mut database_connections = setup_test_db();
    let aircraft = Aircraft {
        id: 1,
        manufacturer: "Boeing".to_string(),
        variant: "737-800".to_string(),
        icao_code: "B738".to_string(),
        flown: 0,
        aircraft_range: 3000,
        category: "A".to_string(),
        cruise_speed: 450,
        date_flown: Some("2024-12-10".to_string()),
        takeoff_distance: Some(4000),
    };
    let departure = database_connections.get_airport_by_icao("EHAM").unwrap();
    let destination = database_connections.get_destination_airport(&aircraft, &departure);
    assert!(matches!(destination, Err(AirportSearchError::NotFound)));
}

#[test]
fn test_get_destination_airport_all_suitable_runways() {
    let mut database_connections = setup_test_db();
    let aircraft = Aircraft {
        id: 1,
        manufacturer: "Boeing".to_string(),
        variant: "737-800".to_string(),
        icao_code: "B738".to_string(),
        flown: 0,
        aircraft_range: 3000,
        category: "A".to_string(),
        cruise_speed: 450,
        date_flown: Some("2024-12-10".to_string()),
        takeoff_distance: Some(2000),
    };
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
        id: 1,
        manufacturer: "Boeing".to_string(),
        variant: "737-800".to_string(),
        icao_code: "B738".to_string(),
        flown: 0,
        aircraft_range: 3000,
        category: "A".to_string(),
        cruise_speed: 450,
        date_flown: Some("2024-12-10".to_string()),
        takeoff_distance: Some(1000),
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
        id: 1,
        manufacturer: "Boeing".to_string(),
        variant: "737-800".to_string(),
        icao_code: "B738".to_string(),
        flown: 0,
        aircraft_range: 3000,
        category: "A".to_string(),
        cruise_speed: 450,
        date_flown: Some("2024-12-10".to_string()),
        takeoff_distance: Some(100_000),
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
