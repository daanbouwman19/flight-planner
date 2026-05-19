use super::camera::{Camera, DEFAULT_FOV_Y, MIN_DISTANCE, MAX_DISTANCE};

/// Minimum altitude above the unit-sphere surface (near street-level).
pub const MIN_ALTITUDE: f32 = MIN_DISTANCE - 1.0;
/// Maximum altitude (whole globe with breathing room).
pub const MAX_ALTITUDE: f32 = MAX_DISTANCE - 1.0;

/// User-visible map state: where the camera looks, how far, what bearing.
/// This is the source of truth for interaction; rendering converts to [`Camera`].
///
/// Invariants: `center_lat` ∈ [−85, 85] degrees; `altitude` ∈ [MIN_ALTITUDE, MAX_ALTITUDE].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MapView {
    /// Latitude of the point the camera looks at (screen centre), degrees.
    pub center_lat: f32,
    /// Longitude of the point the camera looks at (screen centre), degrees.
    pub center_lon: f32,
    /// Camera height above the unit-sphere surface.
    pub altitude: f32,
    /// Bearing: which direction is "up" on screen. 0 = north, radians.
    pub bearing: f32,
    /// Vertical field of view, radians.
    pub fov_y: f32,
}

impl Default for MapView {
    fn default() -> Self {
        Self {
            center_lat: 0.0,
            center_lon: 0.0,
            altitude: 2.0,
            bearing: 0.0,
            fov_y: DEFAULT_FOV_Y,
        }
    }
}

impl MapView {
    /// Build the rendering camera from this map view.
    ///
    /// Derivation: `inverse_rotate([0,0,1])` = nadir in world space.
    /// With yaw = −lon_rad, pitch = lat_rad, roll = bearing, that nadir equals
    /// `lat_lon_to_world(center_lat, center_lon)`. Roll does not affect the nadir
    /// (it is rotation around Z in camera space), so bearing is purely cosmetic.
    pub fn to_camera(&self) -> Camera {
        Camera {
            yaw: -self.center_lon.to_radians(),
            pitch: self.center_lat.to_radians(),
            roll: self.bearing,
            distance: 1.0 + self.altitude,
            fov_y: self.fov_y,
        }
    }

    /// Update `center_lat` and `center_lon` from a camera that was derived from
    /// this view and then mutated by `rotate_to_pin`. Bearing and altitude are
    /// left unchanged; the caller sets those independently.
    ///
    /// `rotate_to_pin` only modifies `yaw` and `pitch`, so the roll (= bearing)
    /// round-trips without loss.
    pub fn sync_center_from_camera(&mut self, camera: &Camera) {
        self.center_lat = camera.pitch.to_degrees();
        // Normalize to [−180, 180).
        let raw = (-camera.yaw).to_degrees();
        self.center_lon = (raw + 180.0).rem_euclid(360.0) - 180.0;
    }
}

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
    pub map_view: MapView,
    pub last_p1: [f32; 3],
    pub last_p2: [f32; 3],
    pub drag: Option<Drag>,
}

impl Default for GlobeState {
    fn default() -> Self {
        Self {
            map_view: MapView::default(),
            last_p1: [0.0; 3],
            last_p2: [0.0; 3],
            drag: None,
        }
    }
}
