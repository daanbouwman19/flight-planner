//! The main entry point for the Flight Planner application.

#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::complexity,
    clippy::perf
)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/// The main function that starts the application.
///
/// This function calls `flight_planner::run_app()`, which initializes and runs
/// the application, handling both GUI and CLI modes.
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    flight_planner::run_app();
}

// WASM entry point is defined in lib.rs via #[wasm_bindgen(start)]
#[cfg(target_arch = "wasm32")]
fn main() {}
