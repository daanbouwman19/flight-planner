use crate::schema::*;
use diesel::prelude::*;

#[derive(Queryable, Debug, PartialEq, Clone, Identifiable, AsChangeset)]
#[diesel(table_name = aircraft)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
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
