use crate::schema::Runways;
use diesel::prelude::*;

#[derive(Associations, Queryable, Identifiable, PartialEq, Debug, Clone)]
#[diesel(primary_key(ID))]
#[diesel(belongs_to(super::Airport, foreign_key = AirportID))]
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
