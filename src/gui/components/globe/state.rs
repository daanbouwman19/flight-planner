use super::camera::Camera;
use eframe::egui::Pos2;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DragKind {
    Pan,
    Orbit,
}

#[derive(Clone, Copy, Debug)]
pub struct Drag {
    pub kind: DragKind,
    pub cursor_start: Pos2,
    pub yaw_start: f32,
    pub pitch_start: f32,
    pub roll_start: f32,
    /// Model-space (pre-rotation) world point under cursor at drag-start.
    /// Set for both `Pan` (used for cursor-pinned rotation) and `Orbit` (used as
    /// the anchor that the view spins/tilts around).
    pub world_pt: Option<[f32; 3]>,
}

#[derive(Clone, Copy, Debug)]
pub struct GlobeState {
    pub camera: Camera,
    pub last_p1: [f32; 3],
    pub last_p2: [f32; 3],
    pub drag: Option<Drag>,
}

impl Default for GlobeState {
    fn default() -> Self {
        Self {
            camera: Camera::default(),
            last_p1: [0.0; 3],
            last_p2: [0.0; 3],
            drag: None,
        }
    }
}
