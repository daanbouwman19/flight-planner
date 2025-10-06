//! Tests for the weather module, using a mock API server.

use flight_planner::modules::weather;
use mockito::Server;
use reqwest::Client;

#[tokio::test]
async fn test_get_weather_data_success() {
    // Arrange
    let mut server = Server::new_async().await;
    let mock = server
        .mock("GET", "/api/metar/KJFK")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
                "wind_speed": { "value": 10 },
                "visibility": { "value": 10.0 },
                "flight_rules": "VFR"
            }"#,
        )
        .create_async()
        .await;

    let client = Client::new();
    let base_url = &server.url();

    // Act
    let result = weather::get_weather_data(base_url, "KJFK", &client, "dummy_key").await;

    // Assert
    assert!(result.is_ok());
    let metar = result.unwrap();
    assert_eq!(metar.flight_rules, "VFR");
    assert!(metar.wind_speed.is_some());
    assert_eq!(metar.wind_speed.as_ref().unwrap().value, 10);
    assert!(metar.visibility.is_some());
    assert_eq!(metar.visibility.as_ref().unwrap().value, 10.0);
    mock.assert_async().await;
}

#[tokio::test]
async fn test_get_weather_data_not_found() {
    // Arrange
    let mut server = Server::new_async().await;
    let mock = server
        .mock("GET", "/api/metar/INVALID")
        .with_status(404)
        .create_async()
        .await;

    let client = Client::new();
    let base_url = &server.url();

    // Act
    let result = weather::get_weather_data(base_url, "INVALID", &client, "dummy_key").await;

    // Assert
    assert!(result.is_err());
    mock.assert_async().await;
}