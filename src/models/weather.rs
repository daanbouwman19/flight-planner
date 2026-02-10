use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Metar {
    pub raw: Option<String>,
    pub flight_rules: Option<String>,
    pub san: Option<String>, // Station identifier
    pub time: Option<MetarTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MetarTime {
    pub repr: Option<String>,
    pub dt: Option<String>,
}

#[derive(Debug, Clone)]
pub enum WeatherError {
    Request(String),
    Api(String), // Store status code as string to avoid lifetime/dependency issues in model if possible, or just use String for simplicity in this context
    Parse(String),
    NoData,
    StationNotFound,
}

impl std::fmt::Display for WeatherError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WeatherError::Request(e) => write!(f, "Network Error: {}", e),
            WeatherError::Api(s) => write!(f, "API Error: {}", s),
            WeatherError::Parse(e) => write!(f, "Parse Error: {}", e),
            WeatherError::NoData => write!(f, "No METAR data available"),
            WeatherError::StationNotFound => write!(f, "Station not found"),
        }
    }
}

impl std::error::Error for WeatherError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlightRules {
    VFR,
    MVFR,
    IFR,
    LIFR,
    Unknown,
}

impl From<&str> for FlightRules {
    fn from(s: &str) -> Self {
        match s {
            "VFR" => Self::VFR,
            "MVFR" => Self::MVFR,
            "IFR" => Self::IFR,
            "LIFR" => Self::LIFR,
            _ => Self::Unknown,
        }
    }
}

impl FlightRules {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::VFR => "VFR",
            Self::MVFR => "MVFR",
            Self::IFR => "IFR",
            Self::LIFR => "LIFR",
            Self::Unknown => "N/A",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::VFR => {
                "Visual Flight Rules\n\nCeiling > 3,000 ft AND\nVisibility > 5 statute miles"
            }
            Self::MVFR => {
                "Marginal Visual Flight Rules\n\nCeiling 1,000 to 3,000 ft OR\nVisibility 3 to 5 statute miles"
            }
            Self::IFR => {
                "Instrument Flight Rules\n\nCeiling 500 to < 1,000 ft OR\nVisibility 1 to < 3 statute miles"
            }
            Self::LIFR => {
                "Low Instrument Flight Rules\n\nCeiling < 500 ft OR\nVisibility < 1 statute mile"
            }
            Self::Unknown => "Flight category unknown",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flight_rules_description() {
        assert!(
            FlightRules::VFR
                .description()
                .contains("Visual Flight Rules")
        );
        assert!(FlightRules::VFR.description().contains("> 3,000 ft"));

        assert!(
            FlightRules::MVFR
                .description()
                .contains("Marginal Visual Flight Rules")
        );
        assert!(
            FlightRules::MVFR
                .description()
                .contains("1,000 to 3,000 ft")
        );

        assert!(
            FlightRules::IFR
                .description()
                .contains("Instrument Flight Rules")
        );
        assert!(FlightRules::IFR.description().contains("500 to < 1,000 ft"));

        assert!(
            FlightRules::LIFR
                .description()
                .contains("Low Instrument Flight Rules")
        );
        assert!(FlightRules::LIFR.description().contains("< 500 ft"));

        assert_eq!(
            FlightRules::Unknown.description(),
            "Flight category unknown"
        );
    }

    #[test]
    fn test_flight_rules_from_str() {
        assert_eq!(FlightRules::from("VFR"), FlightRules::VFR);
        assert_eq!(FlightRules::from("MVFR"), FlightRules::MVFR);
        assert_eq!(FlightRules::from("IFR"), FlightRules::IFR);
        assert_eq!(FlightRules::from("LIFR"), FlightRules::LIFR);
        assert_eq!(FlightRules::from("OTHER"), FlightRules::Unknown);
    }

    #[test]
    fn test_flight_rules_as_str() {
        assert_eq!(FlightRules::VFR.as_str(), "VFR");
        assert_eq!(FlightRules::MVFR.as_str(), "MVFR");
        assert_eq!(FlightRules::IFR.as_str(), "IFR");
        assert_eq!(FlightRules::LIFR.as_str(), "LIFR");
        assert_eq!(FlightRules::Unknown.as_str(), "N/A");
    }

    #[test]
    fn test_weather_error_display() {
        assert_eq!(
            format!("{}", WeatherError::Request("timeout".into())),
            "Network Error: timeout"
        );
        assert_eq!(
            format!("{}", WeatherError::Api("404".into())),
            "API Error: 404"
        );
        assert_eq!(
            format!("{}", WeatherError::Parse("bad json".into())),
            "Parse Error: bad json"
        );
        assert_eq!(
            format!("{}", WeatherError::NoData),
            "No METAR data available"
        );
        assert_eq!(
            format!("{}", WeatherError::StationNotFound),
            "Station not found"
        );
    }

    #[cfg(feature = "gui")]
    mod json_tests {
        use crate::models::weather::Metar;
        use serde_json::json;

        #[test]
        fn test_metar_deserialization_full() {
            let json_data = json!({
                "raw": "KJFK 181951Z 32015G24KT 10SM FEW050 12/M06 A3004 RMK AO2 PK WND 32029/1932 SLP172 T01171061",
                "flight_rules": "VFR",
                "san": "KJFK",
                "time": {
                    "repr": "181951Z",
                    "dt": "2023-11-18T19:51:00Z"
                }
            });

            let metar: Metar =
                serde_json::from_value(json_data).expect("Failed to deserialize METAR");

            assert_eq!(
                metar.raw,
                Some(
                    "KJFK 181951Z 32015G24KT 10SM FEW050 12/M06 A3004 RMK AO2 PK WND 32029/1932 SLP172 T01171061"
                        .to_string()
                )
            );
            assert_eq!(metar.flight_rules, Some("VFR".to_string()));
            assert_eq!(metar.san, Some("KJFK".to_string()));
            assert!(metar.time.is_some());
            assert_eq!(metar.time.unwrap().repr, Some("181951Z".to_string()));
        }

        #[test]
        fn test_metar_deserialization_partial() {
            let json_data = json!({
                "san": "KJFK",
                // Missing raw, flight_rules, time
            });

            let metar: Metar =
                serde_json::from_value(json_data).expect("Failed to deserialize partial METAR");

            assert_eq!(metar.san, Some("KJFK".to_string()));
            assert!(metar.raw.is_none());
            assert!(metar.flight_rules.is_none());
            assert!(metar.time.is_none());
        }

        #[test]
        fn test_metar_deserialization_empty() {
            let json_data = json!({});

            let metar: Metar =
                serde_json::from_value(json_data).expect("Failed to deserialize empty METAR");

            assert!(metar.san.is_none());
            assert!(metar.raw.is_none());
            assert!(metar.flight_rules.is_none());
            assert!(metar.time.is_none());
        }

        #[test]
        fn test_metar_deserialization_invalid_type() {
            let json_data = json!({
                "san": 12345 // Should be string
            });

            let result: Result<Metar, _> = serde_json::from_value(json_data);
            assert!(result.is_err(), "Should fail to deserialize invalid type");
        }
    }
}
