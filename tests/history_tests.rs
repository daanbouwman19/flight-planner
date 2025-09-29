use diesel::connection::SimpleConnection;
use diesel::{Connection, SqliteConnection};
use flight_planner::database::{DatabaseConnections, DatabasePool};
use flight_planner::models::{Aircraft, Airport};
use flight_planner::modules::data_operations::DataOperations;
use flight_planner::traits::{AircraftOperations, HistoryOperations};
use std::sync::Arc;

fn setup_test_pool_db() -> DatabasePool {
    let aircraft_db_url = "test_aircraft_pooled.db";
    let airport_db_url = "test_airport_pooled.db";

    // Clean up previous runs
    if std::path::Path::new(aircraft_db_url).exists() {
        std::fs::remove_file(aircraft_db_url).unwrap();
    }
    if std::path::Path::new(airport_db_url).exists() {
        std::fs::remove_file(airport_db_url).unwrap();
    }

    let mut aircraft_conn = SqliteConnection::establish(aircraft_db_url).unwrap();
    let mut airport_conn = SqliteConnection::establish(airport_db_url).unwrap();

    aircraft_conn
        .batch_execute(
            "
        CREATE TABLE history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            departure_icao TEXT NOT NULL,
            arrival_icao TEXT NOT NULL,
            aircraft INTEGER NOT NULL,
            date TEXT NOT NULL,
            distance INTEGER
        );
        CREATE TABLE aircraft (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            manufacturer TEXT NOT NULL,
            variant TEXT NOT NULL,
            icao_code TEXT NOT NULL,
            flown INTEGER NOT NULL,
            aircraft_range INTEGER NOT NULL,
            category TEXT NOT NULL,
            cruise_speed INTEGER NOT NULL,
            date_flown TEXT,
            takeoff_distance INTEGER
        );
        INSERT INTO aircraft (id, manufacturer, variant, icao_code, flown, aircraft_range, category, cruise_speed, date_flown, takeoff_distance)
        VALUES (1, 'Boeing', '737-800', 'B738', 0, 3000, 'A', 450, NULL, 2000);
    ",
        )
        .unwrap();

    airport_conn
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
        INSERT INTO Airports (ID, Name, ICAO, PrimaryID, Latitude, Longtitude, Elevation, TransitionAltitude, TransitionLevel, SpeedLimit, SpeedLimitAltitude)
        VALUES (1, 'Amsterdam Airport Schiphol', 'EHAM', NULL, 52.3086, 4.7639, -11, 10000, NULL, 230, 6000),
               (2, 'Rotterdam The Hague Airport', 'EHRD', NULL, 51.9561, 4.4397, -13, 5000, NULL, 180, 4000);
    ",
        )
        .unwrap();

    DatabasePool::new(Some(aircraft_db_url), Some(airport_db_url)).unwrap()
}

fn setup_test_db() -> DatabaseConnections {
    let aircraft_connection = SqliteConnection::establish(":memory:").unwrap();
    let airport_connection = SqliteConnection::establish(":memory:").unwrap();

    let mut database_connections = DatabaseConnections {
        aircraft_connection,
        airport_connection,
    };

    database_connections
        .aircraft_connection
        .batch_execute(
            "
            CREATE TABLE history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                departure_icao TEXT NOT NULL,
                arrival_icao TEXT NOT NULL,
                aircraft INTEGER NOT NULL,
                date TEXT NOT NULL,
                distance INTEGER
            );
            CREATE TABLE aircraft (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                manufacturer TEXT NOT NULL,
                variant TEXT NOT NULL,
                icao_code TEXT NOT NULL,
                flown INTEGER NOT NULL,
                aircraft_range INTEGER NOT NULL,
                category TEXT NOT NULL,
                cruise_speed INTEGER NOT NULL,
                date_flown TEXT,
                takeoff_distance INTEGER
            );
            INSERT INTO aircraft (manufacturer, variant, icao_code, flown, aircraft_range, category, cruise_speed, date_flown, takeoff_distance)
            VALUES ('Boeing', '737-800', 'B738', 0, 3000, 'A', 450, '2024-12-10', 2000);
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
                   ('Rotterdam The Hague Airport', 'EHRD', NULL, 51.9561, 4.4397, -13, 5000, NULL, 180, 4000);
            ",
        )
        .expect("Failed to create test data");

    database_connections
}

#[test]
fn test_add_to_history() {
    let mut database_connections = setup_test_db();
    let departure = Airport {
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
    let arrival = Airport {
        ID: 2,
        Name: "Rotterdam The Hague Airport".to_string(),
        ICAO: "EHRD".to_string(),
        PrimaryID: None,
        Latitude: 51.9561,
        Longtitude: 4.4397,
        Elevation: -13,
        TransitionAltitude: Some(5000),
        TransitionLevel: None,
        SpeedLimit: Some(180),
        SpeedLimitAltitude: Some(4000),
    };
    let aircraft_record = Aircraft {
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

    database_connections
        .add_to_history(&departure, &arrival, &aircraft_record)
        .unwrap();

    let history_records = database_connections.get_history().unwrap();
    assert_eq!(history_records.len(), 1);
    assert_eq!(history_records[0].departure_icao, "EHAM");
    assert_eq!(history_records[0].arrival_icao, "EHRD");
    assert_eq!(history_records[0].aircraft, 1);
}

#[test]
fn test_get_history() {
    let mut database_connections = setup_test_db();
    let departure = Airport {
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
    let arrival = Airport {
        ID: 2,
        Name: "Rotterdam The Hague Airport".to_string(),
        ICAO: "EHRD".to_string(),
        PrimaryID: None,
        Latitude: 51.9561,
        Longtitude: 4.4397,
        Elevation: -13,
        TransitionAltitude: Some(5000),
        TransitionLevel: None,
        SpeedLimit: Some(180),
        SpeedLimitAltitude: Some(4000),
    };
    let aircraft_record = Aircraft {
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

    database_connections
        .add_to_history(&departure, &arrival, &aircraft_record)
        .unwrap();
    database_connections
        .add_to_history(&arrival, &departure, &aircraft_record)
        .unwrap();

    let history_records = database_connections.get_history().unwrap();
    assert_eq!(history_records.len(), 2);
    // The order is reversed because the history is ordered by id.desc.
    assert_eq!(history_records[0].departure_icao, "EHRD");
    assert_eq!(history_records[0].arrival_icao, "EHAM");
    assert_eq!(history_records[1].departure_icao, "EHAM");
    assert_eq!(history_records[1].arrival_icao, "EHRD");
}

#[test]
fn test_history_with_distance() {
    let mut database_connections = setup_test_db();

    let aircraft_record = Aircraft {
        id: 1,
        manufacturer: "Boeing".to_string(),
        variant: "737-800".to_string(),
        icao_code: "B738".to_string(),
        flown: 0,
        aircraft_range: 3000,
        category: "A".to_string(),
        cruise_speed: 450,
        date_flown: None,
        takeoff_distance: Some(2000),
    };

    let departure = Airport {
        ID: 1,
        Name: "Amsterdam Schiphol".to_string(),
        ICAO: "EHAM".to_string(),
        PrimaryID: None,
        Latitude: 52.3086,
        Longtitude: 4.7639,
        Elevation: -11,
        TransitionAltitude: Some(3000),
        TransitionLevel: Some(60),
        SpeedLimit: Some(250),
        SpeedLimitAltitude: Some(10000),
    };

    let arrival = Airport {
        ID: 2,
        Name: "Rotterdam The Hague Airport".to_string(),
        ICAO: "EHRD".to_string(),
        PrimaryID: None,
        Latitude: 51.9569,
        Longtitude: 4.4372,
        Elevation: -14,
        TransitionAltitude: Some(3000),
        TransitionLevel: Some(60),
        SpeedLimit: Some(250),
        SpeedLimitAltitude: Some(10000),
    };

    // Add a flight to history
    database_connections
        .add_to_history(&departure, &arrival, &aircraft_record)
        .unwrap();

    let history_records = database_connections.get_history().unwrap();
    assert_eq!(history_records.len(), 1);
    assert!(history_records[0].distance.is_some());
    assert!(history_records[0].distance.unwrap() > 0);
}

#[test]
fn test_deterministic_statistics_tie_breaking() {
    let mut database_connections = setup_test_db();

    // Create test aircraft with different IDs
    let aircraft1 = Aircraft {
        id: 1,
        manufacturer: "Boeing".to_string(),
        variant: "737-800".to_string(),
        icao_code: "B738".to_string(),
        flown: 0,
        aircraft_range: 3000,
        category: "A".to_string(),
        cruise_speed: 450,
        date_flown: None,
        takeoff_distance: Some(2000),
    };

    let aircraft2 = Aircraft {
        id: 2,
        manufacturer: "Airbus".to_string(),
        variant: "A320".to_string(),
        icao_code: "A320".to_string(),
        flown: 0,
        aircraft_range: 3000,
        category: "A".to_string(),
        cruise_speed: 450,
        date_flown: None,
        takeoff_distance: Some(2000),
    };

    let departure = Airport {
        ID: 1,
        Name: "Amsterdam Schiphol".to_string(),
        ICAO: "EHAM".to_string(),
        PrimaryID: None,
        Latitude: 52.3086,
        Longtitude: 4.7639,
        Elevation: -11,
        TransitionAltitude: Some(3000),
        TransitionLevel: Some(60),
        SpeedLimit: Some(250),
        SpeedLimitAltitude: Some(10000),
    };

    let arrival = Airport {
        ID: 2,
        Name: "Rotterdam The Hague Airport".to_string(),
        ICAO: "EHRD".to_string(),
        PrimaryID: None,
        Latitude: 51.9569,
        Longtitude: 4.4372,
        Elevation: -14,
        TransitionAltitude: Some(3000),
        TransitionLevel: Some(60),
        SpeedLimit: Some(250),
        SpeedLimitAltitude: Some(10000),
    };

    // Add equal flights for both aircraft to create a tie scenario
    database_connections
        .add_to_history(&departure, &arrival, &aircraft1)
        .unwrap();
    database_connections
        .add_to_history(&departure, &arrival, &aircraft2)
        .unwrap();

    let history_records = database_connections.get_history().unwrap();
    assert_eq!(history_records.len(), 2);

    // Create aircraft list for statistics calculation
    let aircraft_list: Vec<Arc<Aircraft>> =
        vec![Arc::new(aircraft1.clone()), Arc::new(aircraft2.clone())];

    // Calculate statistics using the actual implementation
    let stats = DataOperations::calculate_statistics_from_history(&history_records, &aircraft_list);

    // Verify deterministic tie-breaking: aircraft1 has lower ID (1) than aircraft2 (2)
    // So aircraft1 should be selected as "most flown" in case of ties
    assert_eq!(stats.total_flights, 2);
    assert_eq!(
        stats.most_flown_aircraft,
        Some("Boeing 737-800".to_string())
    );

    // Test that the result is stable by calculating multiple times
    let stats2 =
        DataOperations::calculate_statistics_from_history(&history_records, &aircraft_list);
    assert_eq!(stats.most_flown_aircraft, stats2.most_flown_aircraft);

    // Verify airport tie-breaking as well
    // Both airports appear twice (EHAM twice, EHRD twice), but EHAM comes first alphabetically
    assert_eq!(stats.most_visited_airport, Some("EHAM".to_string()));
}

#[test]
fn test_deterministic_statistics_airport_tie_breaking() {
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
        date_flown: None,
        takeoff_distance: Some(2000),
    };

    let airport_a = Airport {
        ID: 1,
        Name: "Zurich".to_string(),
        ICAO: "LSZH".to_string(), // Z comes after E alphabetically
        PrimaryID: None,
        Latitude: 47.4647,
        Longtitude: 8.5492,
        Elevation: 1416,
        TransitionAltitude: Some(5000),
        TransitionLevel: Some(70),
        SpeedLimit: Some(250),
        SpeedLimitAltitude: Some(10000),
    };

    let airport_b = Airport {
        ID: 2,
        Name: "Amsterdam".to_string(),
        ICAO: "EHAM".to_string(), // E comes before Z alphabetically
        PrimaryID: None,
        Latitude: 52.3086,
        Longtitude: 4.7639,
        Elevation: -11,
        TransitionAltitude: Some(3000),
        TransitionLevel: Some(60),
        SpeedLimit: Some(250),
        SpeedLimitAltitude: Some(10000),
    };

    // Create equal visits to both airports (2 visits each)
    database_connections
        .add_to_history(&airport_a, &airport_b, &aircraft)
        .unwrap(); // LSZH->EHAM
    database_connections
        .add_to_history(&airport_b, &airport_a, &aircraft)
        .unwrap(); // EHAM->LSZH

    let history_records = database_connections.get_history().unwrap();
    assert_eq!(history_records.len(), 2);

    let aircraft_list: Vec<Arc<Aircraft>> = vec![Arc::new(aircraft)];
    let stats = DataOperations::calculate_statistics_from_history(&history_records, &aircraft_list);

    // Both airports have 2 visits, but EHAM should win alphabetically over LSZH
    assert_eq!(stats.most_visited_airport, Some("EHAM".to_string()));
}

#[test]
fn test_add_history_entry() {
    let mut db_pool = setup_test_pool_db();

    let departure = Arc::new(Airport {
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
    });
    let destination = Arc::new(Airport {
        ID: 2,
        Name: "Rotterdam The Hague Airport".to_string(),
        ICAO: "EHRD".to_string(),
        PrimaryID: None,
        Latitude: 51.9561,
        Longtitude: 4.4397,
        Elevation: -13,
        TransitionAltitude: Some(5000),
        TransitionLevel: None,
        SpeedLimit: Some(180),
        SpeedLimitAltitude: Some(4000),
    });
    let aircraft = Arc::new(Aircraft {
        id: 1,
        manufacturer: "Boeing".to_string(),
        variant: "737-800".to_string(),
        icao_code: "B738".to_string(),
        flown: 0,
        aircraft_range: 3000,
        category: "A".to_string(),
        cruise_speed: 450,
        date_flown: None,
        takeoff_distance: Some(2000),
    });

    // Add a history entry
    DataOperations::add_history_entry(&mut db_pool, &aircraft, &departure, &destination).unwrap();

    // Verify history record was added
    let history_records = db_pool.get_history().unwrap();
    assert_eq!(history_records.len(), 1);
    assert_eq!(history_records[0].departure_icao, "EHAM");
    assert_eq!(history_records[0].arrival_icao, "EHRD");
    assert_eq!(history_records[0].aircraft, 1);

    // Verify aircraft status is unchanged
    let db_aircraft = db_pool.get_aircraft_by_id(1).unwrap();
    assert_eq!(db_aircraft.flown, 0); // Should still be 0
    assert!(db_aircraft.date_flown.is_none());
}
