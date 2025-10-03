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
fn main() {
    flight_planner::run_app();
}