use super::*;

use sqlite;

fn create_airport_database() -> AirportDatabase {
    let connection = sqlite::open(":memory:").unwrap();
    initialize_airport_db(&connection);
    AirportDatabase { connection }
}

fn create_aircraft_database() -> AircraftDatabase {
    let connection = sqlite::open(":memory:").unwrap();
    initialize_aircraft_db(&connection);
    AircraftDatabase { connection }
}

#[test]
fn test_insert_airport() {
    let airport = Airport {
        id: 1,
        name: "Test Airport".to_string(),
        icao_code: "".to_string(),
        latitude: 0.0,
        longtitude: 0.0,
        elevation: 0,
    };

    let airport_database = create_airport_database();
    airport_database.insert_airport(&airport).unwrap();
    let result = airport_database.get_random_airport().unwrap();
    assert_eq!(result, airport);
}

#[test]
fn test_haversine_distance_nm() {
    let airport1 = Airport {
        id: 1,
        name: "Test Airport 1".to_string(),
        icao_code: "".to_string(),
        latitude: 0.0,
        longtitude: 0.0,
        elevation: 0,
    };

    let airport2 = Airport {
        id: 2,
        name: "Test Airport 2".to_string(),
        icao_code: "".to_string(),
        latitude: 0.0,
        longtitude: 1.0,
        elevation: 0,
    };

    let airport_database = create_airport_database();
    let distance = airport_database.haversine_distance_nm(&airport1, &airport2);
    assert_eq!(distance, 60);
}

#[test]
fn test_insert_aircarft() {
    let aircraft = Aircraft {
        id: 1,
        manufacturer: "Test Manufacturer".to_string(),
        icao_code: "TEST".to_string(),
        variant: "Test Variant".to_string(),
        flown: false,
        aircraft_range: 0,
        category: "Test Category".to_string(),
        cruise_speed: 0,
        date_flown: None,
    };

    let aircraft_database = create_aircraft_database();
    aircraft_database.insert_aircraft(&aircraft).unwrap();
    let result = aircraft_database.get_all_aircraft().unwrap();
    assert_eq!(result.len(), 1);

    let result = result.get(0).unwrap();
    assert_eq!(result, &aircraft);

    let aircraft_2 = Aircraft {
        id: 2,
        manufacturer: "Test Manufacturer".to_string(),
        icao_code: "TEST".to_string(),
        variant: "Test Variant".to_string(),
        flown: false,
        aircraft_range: 0,
        category: "Test Category".to_string(),
        cruise_speed: 0,
        date_flown: Some("2021-01-01".to_string()),
    };

    aircraft_database.insert_aircraft(&aircraft_2).unwrap();
    let result = aircraft_database.get_all_aircraft().unwrap();
    assert_eq!(result.len(), 2);

    let result = result.get(1).unwrap();
    assert_eq!(result, &aircraft_2);
}

#[test]
fn test_get_unflown_aircraft_count() {
    let aircraft_database = create_aircraft_database();

    let count = aircraft_database.get_unflown_aircraft_count().unwrap();
    assert_eq!(count, 0);

    let unflown_aircraft = Aircraft {
        id: 1,
        manufacturer: "Test Manufacturer".to_string(),
        icao_code: "TEST".to_string(),
        variant: "Test Variant".to_string(),
        flown: false,
        aircraft_range: 0,
        category: "Test Category".to_string(),
        cruise_speed: 0,
        date_flown: None,
    };

    aircraft_database
        .insert_aircraft(&unflown_aircraft)
        .unwrap();

    let count = aircraft_database.get_unflown_aircraft_count().unwrap();
    assert_eq!(count, 1);

    let flown_aircraft = Aircraft {
        id: 2,
        manufacturer: "Test Manufacturer".to_string(),
        icao_code: "TEST".to_string(),
        variant: "Test Variant".to_string(),
        flown: true,
        aircraft_range: 0,
        category: "Test Category".to_string(),
        cruise_speed: 0,
        date_flown: None,
    };

    aircraft_database.insert_aircraft(&flown_aircraft).unwrap();

    let count = aircraft_database.get_unflown_aircraft_count().unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_insert_runway() {
    let runway = Runway {
        id: 1,
        airport_id: 1,
        ident: "09".to_string(),
        true_heading: 90.0,
        length: 3000,
        width: 45,
        surface: "Asphalt".to_string(),
        latitude: 0.0,
        longtitude: 0.0,
        elevation: 0,
    };

    let airport = Airport {
        id: 1,
        name: "Test Airport".to_string(),
        icao_code: "".to_string(),
        latitude: 0.0,
        longtitude: 0.0,
        elevation: 0,
    };

    let airport_database = create_airport_database();
    airport_database.insert_airport(&airport).unwrap();
    airport_database.insert_runway(&runway).unwrap();

    let result = airport_database
        .get_runways_for_airport(airport.id)
        .unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result.get(0).unwrap(), &runway);
}

#[test]
fn test_get_runways_for_airport() {
    let airport = Airport {
        id: 1,
        name: "Test Airport".to_string(),
        icao_code: "".to_string(),
        latitude: 0.0,
        longtitude: 0.0,
        elevation: 0,
    };

    let airport_database = create_airport_database();
    airport_database.insert_airport(&airport).unwrap();

    let runways = vec![
        Runway {
            id: 1,
            airport_id: 1,
            ident: "09".to_string(),
            true_heading: 90.0,
            length: 3000,
            width: 45,
            surface: "Asphalt".to_string(),
            latitude: 0.0,
            longtitude: 0.0,
            elevation: 0,
        },
        Runway {
            id: 2,
            airport_id: 1,
            ident: "27".to_string(),
            true_heading: 270.0,
            length: 3000,
            width: 45,
            surface: "Asphalt".to_string(),
            latitude: 0.0,
            longtitude: 0.0,
            elevation: 0,
        },
    ];

    for runway in runways.iter() {
        airport_database.insert_runway(runway).unwrap();
    }

    let result = airport_database
        .get_runways_for_airport(airport.id)
        .unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result.get(0).unwrap(), &runways[0]);
    assert_eq!(result.get(1).unwrap(), &runways[1]);
}

#[test]
fn test_get_random_airport() {
    let airport1 = Airport {
        id: 1,
        name: "Test Airport 1".to_string(),
        icao_code: "".to_string(),
        latitude: 0.0,
        longtitude: 0.0,
        elevation: 0,
    };

    let airport2 = Airport {
        id: 2,
        name: "Test Airport 2".to_string(),
        icao_code: "".to_string(),
        latitude: 0.0,
        longtitude: 1.0,
        elevation: 0,
    };

    let runway1 = Runway {
        id: 1,
        airport_id: 1,
        ident: "09".to_string(),
        true_heading: 90.0,
        length: 3000,
        width: 45,
        surface: "Asphalt".to_string(),
        latitude: 0.0,
        longtitude: 0.0,
        elevation: 0,
    };

    let runway2 = Runway {
        id: 2,
        airport_id: 2,
        ident: "09".to_string(),
        true_heading: 90.0,
        length: 3000,
        width: 45,
        surface: "Asphalt".to_string(),
        latitude: 0.0,
        longtitude: 0.0,
        elevation: 0,
    };

    let airport_database = create_airport_database();

    airport_database.insert_airport(&airport1).unwrap();
    airport_database.insert_airport(&airport2).unwrap();
    airport_database.insert_runway(&runway1).unwrap();
    airport_database.insert_runway(&runway2).unwrap();

    let result = airport_database.get_random_airport().unwrap();
    assert!(result == airport1 || result == airport2);
}

#[test]
fn test_get_destination_airport() {
    let aircraft = Aircraft {
        id: 1,
        manufacturer: "Test Manufacturer".to_string(),
        icao_code: "TEST".to_string(),
        variant: "Test Variant".to_string(),
        flown: false,
        aircraft_range: 100,
        category: "Test Category".to_string(),
        cruise_speed: 0,
        date_flown: None,
    };

    let airport_departure = Airport {
        id: 1,
        name: "Test Airport 1".to_string(),
        icao_code: "tst1".to_string(),
        latitude: 0.0,
        longtitude: 0.0,
        elevation: 0,
    };

    let airport_within_range = Airport {
        id: 2,
        name: "Test Airport 2".to_string(),
        icao_code: "tst2".to_string(),
        latitude: 0.0,
        longtitude: 1.0,
        elevation: 0,
    };

    let airport_outside_range = Airport {
        id: 3,
        name: "Test Airport 3".to_string(),
        icao_code: "tst3".to_string(),
        latitude: 0.0,
        longtitude: 2.0,
        elevation: 0,
    };

    let airport_database = create_airport_database();
    airport_database.insert_airport(&airport_departure).unwrap();
    airport_database
        .insert_airport(&airport_within_range)
        .unwrap();
    airport_database
        .insert_airport(&airport_outside_range)
        .unwrap();

    //loop 5 times
    for _ in 0..5 {
        let result = airport_database
            .get_destination_airport(&aircraft, &airport_departure)
            .unwrap();
        assert_eq!(result, airport_within_range);
    }
}

#[test]
fn test_mark_all_aicraft_unflown() {
    let aircraft_database = create_aircraft_database();

    let aircraft = Aircraft {
        id: 1,
        manufacturer: "Test Manufacturer".to_string(),
        icao_code: "TEST".to_string(),
        variant: "Test Variant".to_string(),
        flown: true,
        aircraft_range: 0,
        category: "Test Category".to_string(),
        cruise_speed: 0,
        date_flown: None,
    };

    aircraft_database.insert_aircraft(&aircraft).unwrap();

    let count = aircraft_database.get_unflown_aircraft_count().unwrap();
    assert_eq!(count, 0);

    aircraft_database.mark_all_aircraft_unflown().unwrap();

    let count = aircraft_database.get_unflown_aircraft_count().unwrap();
    assert_eq!(count, 1);
}

#[test]
fn test_all_aircraft() {
    let aircraft_database = create_aircraft_database();

    let aircraft1 = Aircraft {
        id: 1,
        manufacturer: "Test Manufacturer".to_string(),
        icao_code: "TEST".to_string(),
        variant: "Test Variant".to_string(),
        flown: false,
        aircraft_range: 0,
        category: "Test Category".to_string(),
        cruise_speed: 0,
        date_flown: None,
    };

    let aircraft2 = Aircraft {
        id: 2,
        manufacturer: "Test Manufacturer".to_string(),
        icao_code: "TEST".to_string(),
        variant: "Test Variant".to_string(),
        flown: false,
        aircraft_range: 0,
        category: "Test Category".to_string(),
        cruise_speed: 0,
        date_flown: None,
    };

    aircraft_database.insert_aircraft(&aircraft1).unwrap();
    aircraft_database.insert_aircraft(&aircraft2).unwrap();

    let result = aircraft_database.get_all_aircraft().unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result.get(0).unwrap(), &aircraft1);
    assert_eq!(result.get(1).unwrap(), &aircraft2);
}

#[test]
fn test_add_to_history() {
    let aircraft_database = create_aircraft_database();

    let aircraft = Aircraft {
        id: 1,
        manufacturer: "Test Manufacturer".to_string(),
        icao_code: "TEST".to_string(),
        variant: "Test Variant".to_string(),
        flown: false,
        aircraft_range: 0,
        category: "Test Category".to_string(),
        cruise_speed: 0,
        date_flown: None,
    };

    let departure = Airport {
        id: 1,
        name: "Test Airport".to_string(),
        icao_code: "tst1".to_string(),
        latitude: 0.0,
        longtitude: 0.0,
        elevation: 0,
    };

    let arrival = Airport {
        id: 2,
        name: "Test Airport".to_string(),
        icao_code: "tst2".to_string(),
        latitude: 0.0,
        longtitude: 0.0,
        elevation: 0,
    };

    let now = chrono::Local::now();
    let date = now.format("%Y-%m-%d").to_string();

    aircraft_database
        .add_to_history(&departure, &arrival, &aircraft)
        .unwrap();
    let result = aircraft_database.get_history().unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result.get(0).unwrap().arrival_icao, arrival.icao_code);
    assert_eq!(result.get(0).unwrap().departure_icao, departure.icao_code);
    assert_eq!(result.get(0).unwrap().aircraft_id, aircraft.id);
    assert_eq!(result.get(0).unwrap().date, date);
}

#[test]
fn test_random_unflown_aircraft() {
    let aircraft_database = create_aircraft_database();

    let aircraft1 = Aircraft {
        id: 1,
        manufacturer: "Test Manufacturer".to_string(),
        icao_code: "TEST".to_string(),
        variant: "Test Variant".to_string(),
        flown: false,
        aircraft_range: 0,
        category: "Test Category".to_string(),
        cruise_speed: 0,
        date_flown: None,
    };

    let aircraft2 = Aircraft {
        id: 2,
        manufacturer: "Test Manufacturer".to_string(),
        icao_code: "TEST".to_string(),
        variant: "Test Variant".to_string(),
        flown: true,
        aircraft_range: 0,
        category: "Test Category".to_string(),
        cruise_speed: 0,
        date_flown: None,
    };

    aircraft_database.insert_aircraft(&aircraft1).unwrap();
    aircraft_database.insert_aircraft(&aircraft2).unwrap();

    let result = aircraft_database.random_unflown_aircraft().unwrap();
    assert_eq!(result, aircraft1);
}

#[test]
fn test_show_functions() {
    let aircraft_database = create_aircraft_database();
    let airport_database = create_airport_database();

    show_all_aircraft(&aircraft_database);
    show_history(&aircraft_database);
    show_random_aircraft_and_route(&aircraft_database, &airport_database);
    show_random_aircraft_with_random_airport(&aircraft_database, &airport_database);
    show_random_airport(&airport_database);
    show_random_unflown_aircraft(&aircraft_database);

    let aircraft = Aircraft {
        id: 1,
        manufacturer: "Test Manufacturer".to_string(),
        icao_code: "TEST".to_string(),
        variant: "Test Variant".to_string(),
        flown: false,
        aircraft_range: 0,
        category: "Test Category".to_string(),
        cruise_speed: 0,
        date_flown: None,
    };

    let airport = Airport {
        id: 1,
        name: "Test Airport".to_string(),
        icao_code: "tst1".to_string(),
        latitude: 0.0,
        longtitude: 0.0,
        elevation: 0,
    };

    let runway = Runway {
        id: 1,
        airport_id: 1,
        ident: "09".to_string(),
        true_heading: 90.0,
        length: 3000,
        width: 45,
        surface: "Asphalt".to_string(),
        latitude: 0.0,
        longtitude: 0.0,
        elevation: 0,
    };

    aircraft_database.insert_aircraft(&aircraft).unwrap();
    airport_database.insert_airport(&airport).unwrap();
    airport_database.insert_runway(&runway).unwrap();

    show_random_aircraft_and_route(&aircraft_database, &airport_database);
    show_random_aircraft_with_random_airport(&aircraft_database, &airport_database);
    show_random_airport(&airport_database);
}

#[test]
fn test_fmt_debug() {
    let aircraft = Aircraft {
        id: 1,
        manufacturer: "Test Manufacturer".to_string(),
        icao_code: "TEST".to_string(),
        variant: "Test Variant".to_string(),
        flown: false,
        aircraft_range: 0,
        category: "Test Category".to_string(),
        cruise_speed: 0,
        date_flown: None,
    };
    println!("{:?}", aircraft);

    let airport = Airport {
        id: 1,
        name: "Test Airport".to_string(),
        icao_code: "tst1".to_string(),
        latitude: 0.0,
        longtitude: 0.0,
        elevation: 0,
    };
    println!("{:?}", airport);

    let runway = Runway {
        id: 1,
        airport_id: 1,
        ident: "09".to_string(),
        true_heading: 90.0,
        length: 3000,
        width: 45,
        surface: "Asphalt".to_string(),
        latitude: 0.0,
        longtitude: 0.0,
        elevation: 0,
    };
    println!("{:?}", runway);

    let history = History {
        id: 1,
        aircraft_id: 1,
        departure_icao: "tst1".to_string(),
        arrival_icao: "tst2".to_string(),
        date: "2021-01-01".to_string(),
    };
    println!("{:?}", history);
}
