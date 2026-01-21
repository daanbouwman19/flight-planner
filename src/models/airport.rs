use crate::schema::Airports;
use diesel::prelude::*;

#[cfg(feature = "gui")]
use rstar::{AABB, RTreeObject};
#[cfg(feature = "gui")]
use std::sync::Arc;

/// Represents an airport record from the database.
///
/// This struct corresponds to the `Airports` table and is used for querying
/// and managing airport data.
#[derive(Queryable, Identifiable, Insertable, Debug, PartialEq, Clone, Default)]
#[diesel(primary_key(ID))]
#[diesel(table_name = Airports)]
#[allow(non_snake_case)]
pub struct Airport {
    /// The unique identifier for the airport.
    pub ID: i32,
    /// The name of the airport.
    pub Name: String,
    /// The ICAO code of the airport.
    pub ICAO: String,
    /// The primary ID, often used in flight simulation data.
    pub PrimaryID: Option<i32>,
    /// The latitude of the airport in decimal degrees.
    pub Latitude: f64,
    /// The longitude of the airport in decimal degrees.
    pub Longtitude: f64,
    /// The elevation of the airport in feet.
    pub Elevation: i32,
    /// The transition altitude in feet.
    pub TransitionAltitude: Option<i32>,
    /// The transition level.
    pub TransitionLevel: Option<i32>,
    /// The speed limit in knots applicable in the airport's vicinity.
    pub SpeedLimit: Option<i32>,
    /// The altitude up to which the speed limit applies, in feet.
    pub SpeedLimitAltitude: Option<i32>,
}

/// A wrapper for `Airport` to make it compatible with `rstar` for spatial indexing.
///
/// This struct holds an `Arc<Airport>` and implements the `RTreeObject` trait,
/// allowing airports to be efficiently stored and queried in an R-tree based on
/// their geographical coordinates.
///
/// We also store `longest_runway_length` here to optimize filtering during spatial queries,
/// avoiding the need for an external HashMap lookup for every candidate in range.
#[cfg(feature = "gui")]
pub struct SpatialAirport {
    /// A shared pointer to the `Airport` data.
    pub airport: Arc<Airport>,
    /// The length of the longest runway in feet.
    pub longest_runway_length: i32,
}

#[cfg(feature = "gui")]
impl RTreeObject for SpatialAirport {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        let point = [self.airport.Latitude, self.airport.Longtitude];
        AABB::from_point(point)
    }
}
