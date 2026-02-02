mod common;

use flight_planner::modules::data_operations::DataOperations;
use flight_planner::traits::{AircraftOperations, HistoryOperations};
use std::sync::Arc;

use common::{create_test_aircraft, create_test_airport, setup_test_db, setup_test_pool_db};

#[test]
fn test_add_to_history() {
    let mut database_connections = setup_test_db();
    let departure = create_test_airport(1, "Amsterdam Airport Schiphol", "EHAM");
    let arrival = create_test_airport(2, "Rotterdam The Hague Airport", "EHRD");
    let aircraft_record = create_test_aircraft(1, "Boeing", "737-800", "B738");

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
    let departure = create_test_airport(1, "Amsterdam Airport Schiphol", "EHAM");
    let arrival = create_test_airport(2, "Rotterdam The Hague Airport", "EHRD");
    let aircraft_record = create_test_aircraft(1, "Boeing", "737-800", "B738");

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

    let aircraft_record = create_test_aircraft(1, "Boeing", "737-800", "B738");

    let departure = flight_planner::models::Airport {
        Latitude: 52.3086,
        Longtitude: 4.7639,
        ..create_test_airport(1, "Amsterdam Schiphol", "EHAM")
    };

    let arrival = flight_planner::models::Airport {
        Latitude: 51.9569,
        Longtitude: 4.4372,
        ..create_test_airport(2, "Rotterdam The Hague Airport", "EHRD")
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
fn test_statistics_tie_breaking() {
    // Scenario 1: Aircraft Tie Breaking (by ID)
    {
        let mut db = setup_test_db();
        let ac1 = create_test_aircraft(1, "Boeing", "737-800", "B738");
        let ac2 = create_test_aircraft(2, "Airbus", "A320", "A320");
        let list = vec![Arc::new(ac1.clone()), Arc::new(ac2.clone())];

        let dep = create_test_airport(1, "Dep", "DEP");
        let arr = create_test_airport(2, "Arr", "ARR");

        // Add 1 flight for each aircraft on the same route -> Tie
        db.add_to_history(&dep, &arr, &ac1).unwrap();
        db.add_to_history(&dep, &arr, &ac2).unwrap();

        let stats =
            DataOperations::calculate_statistics_from_history(&db.get_history().unwrap(), &list);
        assert_eq!(stats.total_flights, 2);
        assert_eq!(
            stats.most_flown_aircraft,
            Some("Boeing 737-800".to_string()),
            "Tie-break: Lower ID wins"
        );
    }

    // Scenario 2: Airport Tie Breaking (Alphabetical)
    {
        let mut db = setup_test_db();
        let ac = create_test_aircraft(1, "Boeing", "737-800", "B738");
        let list = vec![Arc::new(ac.clone())];

        // LSZH vs EHAM. Both visited twice. EHAM comes first alphabetically.
        let lszh = create_test_airport(1, "Zurich", "LSZH");
        let eham = create_test_airport(2, "Amsterdam", "EHAM");

        db.add_to_history(&lszh, &eham, &ac).unwrap();
        db.add_to_history(&eham, &lszh, &ac).unwrap();

        let stats =
            DataOperations::calculate_statistics_from_history(&db.get_history().unwrap(), &list);
        assert_eq!(
            stats.most_visited_airport,
            Some("EHAM".to_string()),
            "Tie-break: Alphabetical wins"
        );
    }
}

#[test]
fn test_add_history_entry() {
    let mut db_pool = setup_test_pool_db();

    let departure = Arc::new(create_test_airport(1, "Amsterdam Airport Schiphol", "EHAM"));
    let destination = Arc::new(create_test_airport(
        2,
        "Rotterdam The Hague Airport",
        "EHRD",
    ));
    let aircraft = Arc::new(create_test_aircraft(1, "Boeing", "737-800", "B738"));

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
