use super::camera::{Camera, DEFAULT_FOV_Y, MAX_DISTANCE, MIN_DISTANCE};

/// Minimum altitude above the unit-sphere surface (near street-level).
pub const MIN_ALTITUDE: f32 = MIN_DISTANCE - 1.0;
/// Maximum altitude (whole globe with breathing room).
pub const MAX_ALTITUDE: f32 = MAX_DISTANCE - 1.0;

/// Maximum slerp steps for a route (theta ≤ 100°).
pub const MAX_ROUTE_STEPS: usize = 100;

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

#[derive(Clone, Copy, Debug)]
pub struct GlobeState {
    pub camera: Camera,
    pub last_p1: [f32; 3],
    pub last_p2: [f32; 3],
    pub drag: Option<Drag>,
    /// Pre-computed great-circle slerp points between last_p1 and last_p2.
    /// Valid indices: `0..=route_steps`. `route_steps == 0` means no route.
    pub route_points: [[f32; 3]; MAX_ROUTE_STEPS + 1],
    pub route_steps: usize,
}

impl Default for GlobeState {
    fn default() -> Self {
        Self {
            camera: Camera {
                center_lat: 0.0,
                center_lon: 0.0,
                altitude: 2.0,
                bearing: 0.0,
                tilt: 0.0,
                fov_y: DEFAULT_FOV_Y,
            },
            last_p1: [0.0; 3],
            last_p2: [0.0; 3],
            drag: None,
            route_points: [[0.0; 3]; MAX_ROUTE_STEPS + 1],
            route_steps: 0,
        }
    }
}
