//! Defines the data structures used specifically for the GUI.
//!
//! This module contains view-model-like structs that are tailored for display
//! in the user interface. It separates the core application data models from
//! the data structures needed for rendering, ensuring a clean architecture.
//!
//! The `list_items` submodule contains structs for individual list items, while
//! `table_items` provides a unified enum for handling different item types in
//! a generic table.

pub mod list_items;
pub mod table_items;

pub use list_items::*;
pub use table_items::TableItem;