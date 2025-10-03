//! The main module for the application's graphical user interface (GUI).
//!
//! This module organizes the UI into several sub-modules:
//! - `components`: Reusable UI widgets.
//! - `data`: Data structures specific to the GUI.
//! - `events`: The event enum that drives UI interactions.
//! - `services`: Business logic services tailored for the GUI.
//! - `state`: The centralized state management for the UI.
//! - `ui`: The main `Gui` struct and the top-level rendering logic.

pub mod components;
pub mod data;
pub mod events;
pub mod services;
pub mod state;
pub mod ui;