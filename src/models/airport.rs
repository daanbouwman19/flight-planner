use crate::schema::Airports;
use diesel::prelude::*;

#[derive(Queryable, Identifiable, Debug, PartialEq, Clone, Default)]
#[diesel(primary_key(ID))]
#[diesel(table_name = Airports)]
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
