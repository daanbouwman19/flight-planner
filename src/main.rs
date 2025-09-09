#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::complexity,
    clippy::perf
)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    flight_planner::run_app();
}
