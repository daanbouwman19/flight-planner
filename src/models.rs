use diesel::prelude::*;

#[derive(Queryable, Selectable, Debug, PartialEq, Identifiable, Clone, Insertable)]
#[diesel(table_name = crate::schema::aircraft)]
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
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::schema::history)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct History {
    pub id: i32,
    pub departure_icao: String,
    pub arrival_icao: String,
    pub aircraft: i32,
    pub date: String,
}
