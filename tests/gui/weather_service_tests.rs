use diesel::prelude::*;
use diesel_migrations::MigrationHarness;
use flight_planner::database::DatabasePool;
use flight_planner::gui::services::weather_service::WeatherService;
use flight_planner::models::weather::{MetarCacheEntry, WeatherError};
use flight_planner::schema::metar_cache;
use httpmock::prelude::*;
use serde_json::json;

fn setup_test_db() -> DatabasePool {
    let pool = DatabasePool::new(Some(":memory:"), Some(":memory:")).unwrap();
    let mut conn = pool.airport_pool.get().unwrap();
    conn.run_pending_migrations(flight_planner::MIGRATIONS)
        .unwrap();
    pool
}

#[test]
fn test_fetch_metar_success() {
    let server = MockServer::start();
    let metar_mock = server.mock(|when, then| {
        when.method(GET).path("/api/metar/KMCO");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "raw": "KMCO 181253Z 12006KT 10SM FEW030 SCT045 BKN060 22/17 A3012 RMK AO2 SLP198 T02220167",
                "san": "KMCO",
                "flight_rules": "VFR",
                "time": {
                    "repr": "181253Z",
                    "dt": "2023-10-18T12:53:00Z"
                }
            }));
    });

    let pool = setup_test_db();
    let weather_service =
        WeatherService::new("test_api_key".to_string(), pool).with_base_url(server.base_url());
    let result = weather_service.fetch_metar("KMCO");

    metar_mock.assert();
    assert!(result.is_ok());
    let metar = result.unwrap();
    assert_eq!(metar.san, Some("KMCO".to_string()));
    assert_eq!(metar.flight_rules, Some("VFR".to_string()));
}

#[test]
fn test_fetch_metar_caching() {
    let server = MockServer::start();
    let metar_mock = server.mock(|when, then| {
        when.method(GET).path("/api/metar/KLAX");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "san": "KLAX",
                "raw": "KLAX raw metar",
                "time": {
                    "repr": "123456Z"
                }
            }));
    });

    let pool = setup_test_db();
    let weather_service =
        WeatherService::new("test_api_key".to_string(), pool).with_base_url(server.base_url());

    let result1 = weather_service.fetch_metar("KLAX");
    metar_mock.assert();
    assert!(result1.is_ok());

    let result2 = weather_service.fetch_metar("KLAX");
    metar_mock.assert_calls(1); // Should still be 1 call due to caching
    assert!(result2.is_ok());
    assert_eq!(result1.unwrap().raw, result2.unwrap().raw);
}

#[test]
fn test_fetch_metar_station_not_found() {
    let server = MockServer::start();
    let error_mock = server.mock(|when, then| {
        when.method(GET).path("/api/metar/INVALID");
        then.status(400);
    });

    let pool = setup_test_db();
    let weather_service =
        WeatherService::new("test_api_key".to_string(), pool).with_base_url(server.base_url());
    let result = weather_service.fetch_metar("INVALID");

    error_mock.assert();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), WeatherError::StationNotFound));
}

#[test]
fn test_fetch_metar_no_data() {
    let server = MockServer::start();
    let no_content_mock = server.mock(|when, then| {
        when.method(GET).path("/api/metar/NODATA");
        then.status(204);
    });

    let pool = setup_test_db();
    let weather_service =
        WeatherService::new("test_api_key".to_string(), pool).with_base_url(server.base_url());
    let result = weather_service.fetch_metar("NODATA");

    no_content_mock.assert();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), WeatherError::NoData));
}

#[test]
fn test_fetch_metar_api_error() {
    let server = MockServer::start();
    let error_mock = server.mock(|when, then| {
        when.method(GET).path("/api/metar/ERROR");
        then.status(500);
    });

    let pool = setup_test_db();
    let weather_service =
        WeatherService::new("test_api_key".to_string(), pool).with_base_url(server.base_url());
    let result = weather_service.fetch_metar("ERROR");

    error_mock.assert();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), WeatherError::Api(_)));
}

#[test]
fn test_cached_flight_rules_timestamps() {
    let server = MockServer::start();
    let metar_mock = server.mock(|when, then| {
        when.method(GET).path("/api/metar/KJFK");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "raw": "KJFK...",
                "flight_rules": "VFR",
                "time": { "repr": "123456Z" }
            }));
    });

    let pool = setup_test_db();
    let service1 = WeatherService::new("key".into(), pool.clone()).with_base_url(server.base_url());

    // 1. Fetch from API
    let _ = service1.fetch_metar("KJFK");
    metar_mock.assert();

    // 2. Check memory cache (should be fresh)
    let result = service1.get_cached_flight_rules("KJFK");
    assert!(result.is_some());
    let (rules, fetched_at) = result.unwrap();
    assert_eq!(rules, flight_planner::models::weather::FlightRules::VFR);
    // It should be very recent
    assert!(fetched_at.elapsed() < std::time::Duration::from_secs(5));

    // 3. Create new service (empty memory cache, same DB)
    let service2 = WeatherService::new("key".into(), pool.clone()).with_base_url(server.base_url());

    // 4. Check DB cache load (should be old)
    let result2 = service2.get_cached_flight_rules("KJFK");
    assert!(result2.is_some());
    let (rules2, fetched_at2) = result2.unwrap();
    assert_eq!(rules2, flight_planner::models::weather::FlightRules::VFR);
    // It should be considered "old" (loaded from DB). Ideally > 3600s, but on fresh boot it might be less.
    // We try to backdate by at least 600ms in the fallback chain to clear the 500ms animation threshold.
    assert!(fetched_at2.elapsed() > std::time::Duration::from_millis(500));
}

#[test]
fn test_fetch_metar_expired_cache() {
    let server = MockServer::start();
    let metar_mock = server.mock(|when, then| {
        when.method(GET).path("/api/metar/KORD");
        then.status(200)
            .header("content-type", "application/json")
            .json_body(json!({
                "san": "KORD",
                "raw": "KORD NEW METAR",
                "flight_rules": "VFR",
                "time": {
                    "repr": "NEWTIME",
                    "dt": "2023-10-18T13:00:00Z"
                }
            }));
    });

    let pool = setup_test_db();

    // Insert expired entry
    {
        let mut conn = pool.airport_pool.get().unwrap();
        let old_time = chrono::Utc::now() - chrono::Duration::minutes(20); // 20 mins ago (limit is 15)

        let entry = MetarCacheEntry {
            station: "KORD".to_string(),
            raw: "KORD OLD METAR".to_string(),
            flight_rules: Some("IFR".to_string()),
            observation_time: Some("OLDTIME".to_string()),
            observation_dt: Some("2023-10-18T12:00:00Z".to_string()),
            fetched_at: old_time.to_rfc3339(),
        };

        diesel::insert_into(metar_cache::table)
            .values(&entry)
            .execute(&mut conn)
            .unwrap();
    }

    let weather_service =
        WeatherService::new("test_api_key".to_string(), pool).with_base_url(server.base_url());

    let result = weather_service.fetch_metar("KORD");

    metar_mock.assert(); // Ensure API was called
    assert!(result.is_ok());
    let metar = result.unwrap();
    assert_eq!(metar.raw, Some("KORD NEW METAR".to_string()));
    assert_eq!(metar.flight_rules, Some("VFR".to_string()));
}
