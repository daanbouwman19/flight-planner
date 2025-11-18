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
