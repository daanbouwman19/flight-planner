//! Defines the core data models that map directly to database tables.
//!
//! This module contains the struct definitions for `Aircraft`, `Airport`, `History`,
//! and `Runway`. It also re-exports these models for convenient access from other
//! parts of the application and sets up the necessary Diesel ORM relationships
//! between tables.

mod aircraft;
pub mod airport;
mod history;
mod runway;

pub use aircraft::Aircraft;
pub use airport::Airport;
pub use history::{History, NewHistory};
pub use runway::Runway;

use crate::schema::{Airports, Runways};
pub use diesel::prelude::allow_tables_to_appear_in_same_query;
use diesel::prelude::*;

joinable!(Runways -> Airports (AirportID));
allow_tables_to_appear_in_same_query!(Airports, Runways);

pub use aircraft::NewAircraft;
