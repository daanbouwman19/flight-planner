use crate::schema::history;
use diesel::prelude::*;

#[derive(Queryable, Identifiable, Debug, Clone)]
#[diesel(table_name = history)]
pub struct History {
    pub id: i32,
    pub departure_icao: String,
    pub arrival_icao: String,
    pub aircraft: i32,
    pub date: String,
}

#[derive(Insertable)]
#[diesel(table_name = history)]
pub struct NewHistory {
    pub departure_icao: String,
    pub arrival_icao: String,
    pub aircraft: i32,
    pub date: String,
}
