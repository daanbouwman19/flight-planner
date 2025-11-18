#[cfg(test)]
mod tests {
    use flight_planner::models::weather::Metar;

    use serde_json::json;

    #[test]
    fn test_metar_deserialization() {
        let json_data = json!({
            "raw": "KJFK 181951Z 32015G24KT 10SM FEW050 12/M06 A3004 RMK AO2 PK WND 32029/1932 SLP172 T01171061",
            "flight_rules": "VFR",
            "san": "KJFK",
            "time": {
                "repr": "181951Z",
                "dt": "2023-11-18T19:51:00Z"
            }
        });

        let metar: Metar = serde_json::from_value(json_data).expect("Failed to deserialize METAR");

        assert_eq!(metar.raw, Some("KJFK 181951Z 32015G24KT 10SM FEW050 12/M06 A3004 RMK AO2 PK WND 32029/1932 SLP172 T01171061".to_string()));
        assert_eq!(metar.flight_rules, Some("VFR".to_string()));
        assert_eq!(metar.san, Some("KJFK".to_string()));
        assert!(metar.time.is_some());
        assert_eq!(metar.time.unwrap().repr, Some("181951Z".to_string()));
    }
}
