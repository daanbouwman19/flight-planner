//! Contains the core business logic of the application.
//!
//! This module is organized by domain, with each submodule handling a specific
//! area of functionality, such as aircraft management, airport operations, or
//! route generation. These modules are designed to be independent of the user
//! interface and can be used by both the GUI and CLI.

pub mod aircraft;
pub mod airport;
pub mod data_operations;
pub mod history;
#[cfg(feature = "gui")]
pub mod routes;
pub mod runway;
