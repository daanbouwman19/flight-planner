use crate::schema::aircraft;
use diesel::prelude::*;

#[derive(Queryable, Debug, PartialEq, Eq, Clone, Identifiable, AsChangeset)]
#[diesel(table_name = aircraft)]
#[allow(clippy::struct_field_names)]
pub struct Aircraft {
    pub id: i32,
    pub manufacturer: String,
    pub variant: String,
    pub icao_code: String,
    pub flown: i32,
    pub aircraft_range: i32,
    pub category: String,
    pub cruise_speed: i32,
    pub date_flown: Option<String>,
    pub takeoff_distance: Option<i32>,
}

#[derive(Insertable, Debug, PartialEq, Eq)]
#[diesel(table_name = aircraft)]
pub struct NewAircraft {
    pub manufacturer: String,
    pub variant: String,
    pub icao_code: String,
    pub flown: i32,
    pub aircraft_range: i32,
    pub category: String,
    pub cruise_speed: i32,
    pub date_flown: Option<String>,
    pub takeoff_distance: Option<i32>,
}
