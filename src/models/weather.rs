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
}
