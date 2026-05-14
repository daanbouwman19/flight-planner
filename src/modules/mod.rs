//! Contains the core business logic of the application.
//!
//! This module is organized by domain, with each submodule handling a specific
//! area of functionality, such as aircraft management, airport operations, or
//! route generation. These modules are designed to be independent of the user
//! interface and can be used by both the GUI and CLI.

#[cfg(not(target_arch = "wasm32"))]
pub mod aircraft;
#[cfg(any(feature = "gui", feature = "web"))]
pub mod airport;
#[cfg(any(feature = "gui", feature = "web"))]
pub mod data_operations;
#[cfg(not(target_arch = "wasm32"))]
pub mod history;
#[cfg(all(feature = "gui", not(target_arch = "wasm32")))]
pub mod http;
#[cfg(any(feature = "gui", feature = "web"))]
pub mod routes;
#[cfg(not(target_arch = "wasm32"))]
pub mod runway;
