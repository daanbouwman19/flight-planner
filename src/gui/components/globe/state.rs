use super::camera::Camera;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DragKind {
    Pan,
    Orbit,
}

#[derive(Clone, Copy, Debug)]
pub struct Drag {
    pub kind: DragKind,
    /// Model-space world point under cursor at drag-start (used to pin pan).
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
