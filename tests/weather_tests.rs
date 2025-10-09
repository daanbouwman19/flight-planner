use flight_planner::modules::weather::WeatherApi;
use mockito;

#[tokio::test]
async fn test_get_metar_success() {
    let mut server = mockito::Server::new_async().await;
    let mock = server.mock("GET", "/api/metar/KJFK")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"raw": "KJFK 090451Z 19010KT 10SM CLR 25/15 A2992 RMK AO2 SLP134 T02500150", "wind": {"speed_kts": 10.0}}"#)
        .create_async().await;

    let api = WeatherApi::new_with_url("test_api_key".to_string(), server.url());
    let metar = api.get_metar("KJFK").await.unwrap();

    assert_eq!(metar.wind.speed_kts, 10.0);
    assert_eq!(metar.raw, "KJFK 090451Z 19010KT 10SM CLR 25/15 A2992 RMK AO2 SLP134 T02500150");
    mock.assert_async().await;
}

#[tokio::test]
async fn test_get_metar_not_found() {
    let mut server = mockito::Server::new_async().await;
    let mock = server.mock("GET", "/api/metar/INVALID")
        .with_status(404)
        .create_async().await;

    let api = WeatherApi::new_with_url("test_api_key".to_string(), server.url());
    let result = api.get_metar("INVALID").await;

    assert!(result.is_err());
    mock.assert_async().await;
}