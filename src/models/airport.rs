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

/// A wrapper around `Arc<Airport>` that stores pre-calculated trigonometric values.
///
/// This struct is used to optimize distance calculations (Haversine formula) in tight loops
/// by avoiding repetitive `to_radians()` and `cos()` calls.
#[cfg(feature = "gui")]
#[derive(Clone, Debug)]
pub struct CachedAirport {
    /// A shared pointer to the `Airport` data.
    pub inner: Arc<Airport>,
    /// Latitude in radians (f32).
    pub lat_rad: f32,
    /// Longitude in radians (f32).
    pub lon_rad: f32,
    /// Sine of the latitude (f32).
    pub sin_lat: f32,
    /// Cosine of the latitude (f32).
    pub cos_lat: f32,
    /// Sine of the longitude (f32).
    pub sin_lon: f32,
    /// Cosine of the longitude (f32).
    pub cos_lon: f32,
    /// The length of the longest runway in feet.
    pub longest_runway_length: i32,
    /// Pre-formatted display name "Name (ICAO)". Wrapped in Arc for cheap cloning.
    pub display_name: Arc<String>,
}

#[cfg(feature = "gui")]
impl CachedAirport {
    /// Creates a new `CachedAirport` from an `Arc<Airport>`.
    pub fn new(airport: Arc<Airport>, longest_runway_length: i32) -> Self {
        let lat_rad = (airport.Latitude as f32).to_radians();
        let lon_rad = (airport.Longtitude as f32).to_radians();
        let (sin_lat, cos_lat) = lat_rad.sin_cos();
        let (sin_lon, cos_lon) = lon_rad.sin_cos();
        let display_name = Arc::new(format!("{} ({})", airport.Name, airport.ICAO));
        Self {
            inner: airport,
            lat_rad,
            lon_rad,
            sin_lat,
            cos_lat,
            sin_lon,
            cos_lon,
            longest_runway_length,
            display_name,
        }
    }
}

/// A wrapper for `Airport` to make it compatible with `rstar` for spatial indexing.
///
/// This struct holds a `CachedAirport` and implements the `RTreeObject` trait,
/// allowing airports to be efficiently stored and queried in an R-tree based on
/// their geographical coordinates.
#[cfg(feature = "gui")]
pub struct SpatialAirport {
    /// The cached airport data.
    pub airport: CachedAirport,
}

#[cfg(feature = "gui")]
impl RTreeObject for SpatialAirport {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        let point = [self.airport.inner.Latitude, self.airport.inner.Longtitude];
        AABB::from_point(point)
    }
}
