#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::complexity,
    clippy::perf
)]

fn main() {
    flight_planner::run_app();
}
