use eframe::egui::{Pos2, Vec2};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DragKind {
    Pan,
    Orbit,
}

#[derive(Clone, Copy, Debug)]
pub struct DragState {
    pub kind: DragKind,
    pub cursor_start: Pos2,
    pub yaw_start: f32,
    pub pitch_start: f32,
    pub offset_start: Vec2,
    pub world_point: [f32; 3],
}

#[derive(Clone, Copy, Debug)]
pub struct GlobeState {
    pub yaw: f32,
    pub pitch: f32,
    pub zoom: f32,
    pub offset: Vec2,
    pub last_p1: [f32; 3],
    pub last_p2: [f32; 3],
    pub drag: Option<DragState>,
}

impl Default for GlobeState {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            zoom: 1.0,
            offset: Vec2::ZERO,
            last_p1: [0.0; 3],
            last_p2: [0.0; 3],
            drag: None,
        }
    }
}
