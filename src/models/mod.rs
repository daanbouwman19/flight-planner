//! Defines the core data models that map directly to database tables.
//!
//! This module contains the struct definitions for `Aircraft`, `Airport`, `History`,
//! and `Runway`. It also re-exports these models for convenient access from other
//! parts of the application and sets up the necessary Diesel ORM relationships
//! between tables.

mod aircraft;
pub mod airport;
mod history;
pub mod route;
pub mod runway;
pub mod setting;
pub mod statistics;
#[cfg(any(feature = "gui", feature = "web"))]
pub mod weather;

pub use aircraft::Aircraft;
pub use airport::Airport;
pub use history::{History, NewHistory};
pub use route::RouteResponse;
pub use runway::Runway;
pub use statistics::FlightStatistics;

#[cfg(not(target_arch = "wasm32"))]
use crate::schema::{Airports, Runways};
#[cfg(not(target_arch = "wasm32"))]
pub use diesel::prelude::allow_tables_to_appear_in_same_query;
#[cfg(not(target_arch = "wasm32"))]
use diesel::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
joinable!(Runways -> Airports (AirportID));
#[cfg(not(target_arch = "wasm32"))]
allow_tables_to_appear_in_same_query!(Airports, Runways);

pub use aircraft::NewAircraft;
