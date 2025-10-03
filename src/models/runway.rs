use crate::schema::Runways;
use diesel::prelude::*;

/// Represents a runway record from the database.
///
/// This struct corresponds to the `Runways` table and is associated with an `Airport`.
#[derive(Associations, Queryable, Identifiable, Insertable, PartialEq, Debug, Clone)]
#[diesel(primary_key(ID))]
#[diesel(belongs_to(super::Airport, foreign_key = AirportID))]
#[diesel(table_name = Runways)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[allow(non_snake_case)]
pub struct Runway {
    /// The unique identifier for the runway.
    pub ID: i32,
    /// The ID of the airport this runway belongs to.
    pub AirportID: i32,
    /// The identifier of the runway (e.g., "09L" or "27R").
    pub Ident: String,
    /// The true heading of the runway in degrees.
    pub TrueHeading: f64,
    /// The length of the runway in feet.
    pub Length: i32,
    /// The width of the runway in feet.
    pub Width: i32,
    /// The surface material of the runway (e.g., "ASPH" for asphalt).
    pub Surface: String,
    /// The latitude of the runway's start point in decimal degrees.
    pub Latitude: f64,
    /// The longitude of the runway's start point in decimal degrees.
    pub Longtitude: f64,
    /// The elevation of the runway in feet.
    pub Elevation: i32,
}
