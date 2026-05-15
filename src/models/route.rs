use crate::models::Airport;
use serde::{Deserialize, Serialize};

/// A serialisable route returned by the `/api/routes` endpoint.
///
/// The server generates routes using its local `RouteGenerator` and serialises
/// them as this DTO.  The WASM frontend deserialises them and turns them into
/// `ListItemRoute` values without ever needing to hold the full airport/runway
/// database locally.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RouteResponse {
    pub departure: Airport,
    pub destination: Airport,
    pub aircraft_id: i32,
    pub distance_nm: f64,
    pub departure_runway_ft: i32,
    pub destination_runway_ft: i32,
}

/// Server-side conversion from the internally-generated list item.
#[cfg(any(feature = "gui", feature = "web"))]
impl From<&crate::gui::data::ListItemRoute> for RouteResponse {
    fn from(r: &crate::gui::data::ListItemRoute) -> Self {
        Self {
            departure: (*r.departure).clone(),
            destination: (*r.destination).clone(),
            aircraft_id: r.aircraft.id,
            distance_nm: r.route_length,
            departure_runway_ft: r.departure_runway_length,
            destination_runway_ft: r.destination_runway_length,
        }
    }
}
