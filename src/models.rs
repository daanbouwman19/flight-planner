use crate::schema::*;
use diesel::prelude::*;

#[derive(Queryable, Debug, PartialEq, Clone, Insertable, Identifiable, AsChangeset)]
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

#[derive(Queryable, Identifiable, Insertable, Debug, Clone)]
#[diesel(table_name = history)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct History {
    pub id: i32,
    pub departure_icao: String,
    pub arrival_icao: String,
    pub aircraft: i32,
    pub date: String,
}

#[derive(Queryable, Identifiable, Debug, PartialEq, Clone, Insertable, Default)]
#[diesel(primary_key(ID))]
#[diesel(table_name = Airports)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[allow(non_snake_case)]
pub struct Airport {
    pub ID: i32,
    pub Name: String,
    pub ICAO: String,
    pub PrimaryID: Option<i32>,
    pub Latitude: f64,
    pub Longtitude: f64,
    pub Elevation: i32,
    pub TransitionAltitude: Option<i32>,
    pub TransitionLevel: Option<i32>,
    pub SpeedLimit: Option<i32>,
    pub SpeedLimitAltitude: Option<i32>,
}

#[derive(Associations, Queryable, Identifiable, PartialEq, Debug, Insertable, Clone)]
#[diesel(primary_key(ID))]
#[diesel(belongs_to(Airport, foreign_key = AirportID))]
#[diesel(table_name = Runways)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[allow(non_snake_case)]
pub struct Runway {
    pub ID: i32,
    pub AirportID: i32,
    pub Ident: String,
    pub TrueHeading: f64,
    pub Length: i32,
    pub Width: i32,
    pub Surface: String,
    pub Latitude: f64,
    pub Longtitude: f64,
    pub Elevation: i32,
}

joinable!(Runways -> Airports (AirportID));
allow_tables_to_appear_in_same_query!(Airports, Runways);
