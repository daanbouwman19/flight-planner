mod aircraft;
mod airport;
mod history;
mod runway;

pub use aircraft::{Aircraft, NewAircraft};
pub use airport::Airport;
pub use history::{History, NewHistory};
pub use runway::Runway;

use crate::schema::*;
pub use diesel::prelude::allow_tables_to_appear_in_same_query;
use diesel::prelude::*;

joinable!(Runways -> Airports (AirportID));
allow_tables_to_appear_in_same_query!(Airports, Runways);
