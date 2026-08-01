use std::sync::Arc;

use super::camera::{Camera, MAX_DISTANCE, MIN_DISTANCE};

/// Minimum altitude above the unit-sphere surface (near street-level).
pub const MIN_ALTITUDE: f32 = MIN_DISTANCE - 1.0;
/// Maximum altitude (whole globe with breathing room).
pub const MAX_ALTITUDE: f32 = MAX_DISTANCE - 1.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DragKind {
    Pan,
    Orbit,
}

#[derive(Clone, Copy, Debug)]
pub struct Drag {
    pub kind: DragKind,
    /// World-space point under cursor at pan-drag start.
    pub pan_anchor: Option<[f32; 3]>,
}

#[derive(Clone, Debug)]
pub struct GlobeState {
    pub camera: Camera,
    pub last_p1: [f32; 3],
    pub last_p2: [f32; 3],
    pub drag: Option<Drag>,
    /// Pre-computed great-circle slerp points between `last_p1` and `last_p2`.
    /// Empty when departure and destination coincide.
    pub route_points: Arc<[[f32; 3]]>,
}
