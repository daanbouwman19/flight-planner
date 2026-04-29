#[derive(Clone, Copy, Debug)]
pub struct GlobeState {
    pub yaw: f32,
    pub pitch: f32,
    pub zoom: f32,
    pub last_p1: [f32; 3],
    pub last_p2: [f32; 3],
}

impl Default for GlobeState {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            zoom: 1.0,
            last_p1: [0.0; 3],
            last_p2: [0.0; 3],
        }
    }
}
