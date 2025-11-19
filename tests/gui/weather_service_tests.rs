use diesel_migrations::MigrationHarness;
use flight_planner::database::DatabasePool;
use flight_planner::gui::services::weather_service::WeatherService;
use flight_planner::models::weather::WeatherError;
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
