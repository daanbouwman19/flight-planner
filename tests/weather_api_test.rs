
use flight_planner::modules::weather::get_metar;
use httpmock::prelude::*;

#[tokio::test]
async fn test_get_metar_success() {
    let server = MockServer::start();

    let mock = server.mock(|when, then| {
        when.method(GET).path("/api/metar/KJFK");
        then.status(200)
            .header("content-type", "application/json")
            .body(
                r#"{
                    "raw": "KJFK 121851Z 32012G18KT 10SM FEW030 SCT040 BKN050 10/01 A2992 RMK AO2 SLP132 T01000006",
                    "flight_rules": "VFR",
                    "wind_dir": {"value": 320},
                    "wind_speed": {"value": 12},
                    "visibility": {"value": 10},
                    "temperature": {"value": 10},
                    "dewpoint": {"value": 1},
                    "altimeter": {"value": 29.92},
                    "clouds": [],
                    "summary": ""
                }"#,
            );
    });

    let metar_result = get_metar("KJFK", &server.base_url(), "test_api_key").await;
    assert!(metar_result.is_ok());
    let metar = metar_result.unwrap();

    assert_eq!(
        metar.raw,
        "KJFK 121851Z 32012G18KT 10SM FEW030 SCT040 BKN050 10/01 A2992 RMK AO2 SLP132 T01000006"
    );
    assert_eq!(metar.flight_rules, "VFR");

    mock.assert();
}

#[tokio::test]
async fn test_get_metar_api_error() {
    let server = MockServer::start();

    server.mock(|when, then| {
        when.method(GET).path("/api/metar/KJFK");
        then.status(500);
    });

    let metar_result = get_metar("KJFK", &server.base_url(), "test_api_key").await;
    assert!(metar_result.is_err());
}
