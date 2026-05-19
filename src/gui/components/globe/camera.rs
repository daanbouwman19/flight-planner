use eframe::egui::{Pos2, Rect};
use std::f32::consts::PI;

/// Vertical field of view: 60 degrees.
pub const DEFAULT_FOV_Y: f32 = std::f32::consts::FRAC_PI_3;
/// Camera is just above the surface at this distance (street-level view).
pub const MIN_DISTANCE: f32 = 1.0001;
/// Whole globe with breathing room.
pub const MAX_DISTANCE: f32 = 10.0;
pub const PITCH_LIMIT: f32 = std::f32::consts::FRAC_PI_2 - 0.01;
pub const MAX_LOD: u8 = 18;
pub const TILE_PX: f32 = 256.0;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    pub yaw: f32,
    pub pitch: f32,
    /// Roll around the view axis. Driven by right-drag orbit so the user can
    /// spin the view around the anchor point.
    pub roll: f32,
    /// Camera distance from globe origin along +Z in camera space.
    /// Larger = zoomed out; minimum is just above surface.
    pub distance: f32,
    /// Vertical field of view, radians.
    pub fov_y: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            roll: 0.0,
            distance: 3.0,
            fov_y: DEFAULT_FOV_Y,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LatLonBounds {
    pub lat_min: f32,
    pub lat_max: f32,
    pub lon_min: f32,
    pub lon_max: f32,
    /// True when the visible region straddles the antimeridian.
    pub wraps_antimeridian: bool,
}

impl LatLonBounds {
    pub fn full_sphere() -> Self {
        Self {
            lat_min: -85.0,
            lat_max: 85.0,
            lon_min: -180.0,
            lon_max: 180.0,
            wraps_antimeridian: false,
        }
    }

    pub fn intersects_lon(&self, t_lon_min: f32, t_lon_max: f32) -> bool {
        if self.wraps_antimeridian {
            t_lon_max >= self.lon_min || t_lon_min <= self.lon_max
        } else {
            t_lon_max >= self.lon_min && t_lon_min <= self.lon_max
        }
    }

    pub fn intersects_lat(&self, t_lat_min: f32, t_lat_max: f32) -> bool {
        t_lat_max >= self.lat_min && t_lat_min <= self.lat_max
    }
}

pub fn lat_lon_to_world(lat_deg: f32, lon_deg: f32) -> [f32; 3] {
    let lat = lat_deg.to_radians();
    let lon = lon_deg.to_radians();
    [lat.cos() * lon.sin(), lat.sin(), lat.cos() * lon.cos()]
}

pub fn world_to_lat_lon(p: [f32; 3]) -> (f32, f32) {
    let lat = p[1].clamp(-1.0, 1.0).asin().to_degrees();
    let lon = p[0].atan2(p[2]).to_degrees();
    (lat, lon)
}

impl Camera {
    /// Focal length in pixels: half the viewport height divided by tan(fov_y/2).
    pub fn focal_pixels(&self, viewport_height: f32) -> f32 {
        (viewport_height * 0.5) / (self.fov_y * 0.5).tan()
    }

    /// Full rotation chain: yaw (around Y) → pitch (around X) → roll (around Z).
    pub fn rotate(&self, p: [f32; 3]) -> [f32; 3] {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        let (sr, cr) = self.roll.sin_cos();

        let x1 = p[0] * cy + p[2] * sy;
        let y1 = p[1];
        let z1 = -p[0] * sy + p[2] * cy;

        let x2 = x1;
        let y2 = y1 * cp - z1 * sp;
        let z2 = y1 * sp + z1 * cp;

        [x2 * cr - y2 * sr, x2 * sr + y2 * cr, z2]
    }

    pub fn inverse_rotate(&self, p: [f32; 3]) -> [f32; 3] {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        let (sr, cr) = self.roll.sin_cos();

        // Undo roll.
        let x0 = p[0] * cr + p[1] * sr;
        let y0 = -p[0] * sr + p[1] * cr;
        let z0 = p[2];

        // Undo pitch.
        let x1 = x0;
        let y1 = y0 * cp + z0 * sp;
        let z1 = -y0 * sp + z0 * cp;

        // Undo yaw.
        [x1 * cy - z1 * sy, y1, x1 * sy + z1 * cy]
    }

    /// Project a camera-space (post-rotate) point to screen using perspective.
    /// Returns `None` when the point is behind the camera plane.
    pub fn project(&self, p_rotated: [f32; 3], viewport: Rect) -> Option<Pos2> {
        let depth = self.distance - p_rotated[2];
        if depth <= 1e-6 {
            return None;
        }
        let f = self.focal_pixels(viewport.height());
        let c = viewport.center();
        Some(Pos2::new(
            c.x + p_rotated[0] * f / depth,
            c.y - p_rotated[1] * f / depth,
        ))
    }

    pub fn world_to_screen(&self, p: [f32; 3], viewport: Rect) -> Option<Pos2> {
        self.project(self.rotate(p), viewport)
    }

    /// Ray–sphere intersection in camera space. Returns the near-hit point on the
    /// unit sphere (a unit vector), or `None` if the ray through `cursor` misses.
    fn screen_to_camera_sphere(&self, cursor: Pos2, viewport: Rect) -> Option<[f32; 3]> {
        let f = self.focal_pixels(viewport.height());
        let c = viewport.center();
        let ix = (cursor.x - c.x) / f;
        let iy = -(cursor.y - c.y) / f; // y-up in camera space
        let d = self.distance;
        let u = ix * ix + iy * iy + 1.0;
        let disc = d * d - u * (d * d - 1.0);
        if disc < 0.0 {
            return None;
        }
        let t = (d - disc.sqrt()) / u;
        Some([t * ix, t * iy, d - t])
    }

    /// Returns the model-space world point under `cursor`, or `None` if the ray misses.
    pub fn screen_to_world(&self, cursor: Pos2, viewport: Rect) -> Option<[f32; 3]> {
        self.screen_to_camera_sphere(cursor, viewport)
            .map(|p| self.inverse_rotate(p))
    }

    /// Like `screen_to_world` but clamps to the nearest limb point when the cursor
    /// falls outside the visible disc. Used at gesture-start so drags from outside work.
    pub fn screen_to_world_clamped(&self, cursor: Pos2, viewport: Rect) -> [f32; 3] {
        let f = self.focal_pixels(viewport.height());
        let c = viewport.center();
        let mut ix = (cursor.x - c.x) / f;
        let mut iy = -(cursor.y - c.y) / f;
        let d = self.distance;
        // Maximum image-plane radius that still hits the sphere (limb boundary).
        let limb_r2 = (d * d - 1.0).recip();
        let r2 = ix * ix + iy * iy;
        if r2 > limb_r2 {
            let scale = (limb_r2 / r2).sqrt();
            ix *= scale;
            iy *= scale;
        }
        let u = ix * ix + iy * iy + 1.0;
        let disc = (d * d - u * (d * d - 1.0)).max(0.0);
        let t = (d - disc.sqrt()) / u;
        self.inverse_rotate([t * ix, t * iy, d - t])
    }

    /// Adjust yaw/pitch so `world_pt` projects to `target_screen`.
    ///
    /// Uses a ray–sphere hit for the target (perspective-correct), then the same
    /// closed-form yaw/pitch solver as before: undo roll, solve yaw from `t.x`,
    /// solve pitch from `(t.y, t.z)`.
    pub fn rotate_to_pin(&mut self, world_pt: [f32; 3], target_screen: Pos2, viewport: Rect) {
        let Some(cam_pt) = self.screen_to_camera_sphere(target_screen, viewport) else {
            return;
        };
        // Undo roll to recover the target in the (yaw+pitch)-only frame.
        let (sr, cr) = self.roll.sin_cos();
        let tx = cam_pt[0] * cr + cam_pt[1] * sr;
        let ty = -cam_pt[0] * sr + cam_pt[1] * cr;
        let tz = cam_pt[2];

        let r_xz_sq = world_pt[0] * world_pt[0] + world_pt[2] * world_pt[2];
        if r_xz_sq < 1e-10 {
            return;
        }
        let r_xz = r_xz_sq.sqrt();
        if tx.abs() > r_xz {
            return;
        }

        let z1 = (r_xz_sq - tx * tx).max(0.0).sqrt();
        let new_yaw = tx.atan2(z1) - world_pt[0].atan2(world_pt[2]);
        let new_pitch = tz.atan2(ty) - z1.atan2(world_pt[1]);

        if !new_yaw.is_finite() || !new_pitch.is_finite() {
            return;
        }
        if new_pitch.abs() > PITCH_LIMIT {
            return;
        }

        self.yaw = new_yaw;
        self.pitch = new_pitch;
    }

    /// Approximate lat/lon bounds visible from the current camera. Samples the
    /// projected globe-limb rings (which are always on the sphere) plus the viewport
    /// border (needed when the globe extends beyond the viewport at high zoom).
    pub fn visible_lat_lon_bounds(&self, viewport: Rect) -> LatLonBounds {
        let center = viewport.center();
        let f = self.focal_pixels(viewport.height());
        let d = self.distance;
        // Screen-space radius of the globe's visible limb.
        let limb_r = f / (d * d - 1.0).max(1e-4).sqrt();

        let mut points: Vec<Pos2> = Vec::with_capacity(41);

        let viewport_half = viewport.size().min_elem() * 0.5;
        if limb_r < viewport_half {
            // Globe fits inside the viewport (zoomed out): viewport-border samples all
            // miss the sphere, so sample near the limb circle to span the hemisphere.
            for i in 0..16u32 {
                let angle = i as f32 * std::f32::consts::TAU / 16.0;
                let r = 0.95 * limb_r;
                points.push(Pos2::new(
                    center.x + r * angle.cos(),
                    center.y + r * angle.sin(),
                ));
            }
            for i in 0..8u32 {
                let angle = i as f32 * std::f32::consts::TAU / 8.0;
                let r = 0.50 * limb_r;
                points.push(Pos2::new(
                    center.x + r * angle.cos(),
                    center.y + r * angle.sin(),
                ));
            }
        }
        points.push(center);

        // Viewport border (16 samples): covers tiles that extend past the limb
        // when the globe is larger than the viewport (close zoom).
        const SAMPLES_PER_EDGE: usize = 4;
        for i in 0..SAMPLES_PER_EDGE {
            let t = i as f32 / SAMPLES_PER_EDGE as f32;
            points.push(Pos2::new(
                viewport.min.x + t * viewport.width(),
                viewport.min.y,
            ));
            points.push(Pos2::new(
                viewport.max.x,
                viewport.min.y + t * viewport.height(),
            ));
            points.push(Pos2::new(
                viewport.max.x - t * viewport.width(),
                viewport.max.y,
            ));
            points.push(Pos2::new(
                viewport.min.x,
                viewport.max.y - t * viewport.height(),
            ));
        }

        let mut lats: Vec<f32> = Vec::with_capacity(points.len());
        let mut lons: Vec<f32> = Vec::with_capacity(points.len());

        for p in &points {
            if let Some(w) = self.screen_to_world(*p, viewport) {
                let (lat, lon) = world_to_lat_lon(w);
                lats.push(lat);
                lons.push(lon);
            }
        }

        if lats.is_empty() {
            return LatLonBounds::full_sphere();
        }

        let lat_min = lats
            .iter()
            .copied()
            .fold(f32::INFINITY, f32::min)
            .max(-85.0);
        let lat_max = lats
            .iter()
            .copied()
            .fold(f32::NEG_INFINITY, f32::max)
            .min(85.0);

        let mut lon_min = f32::INFINITY;
        let mut lon_max = f32::NEG_INFINITY;
        for &l in &lons {
            lon_min = lon_min.min(l);
            lon_max = lon_max.max(l);
        }

        let span_naive = lon_max - lon_min;
        let (final_lon_min, final_lon_max, wraps) = if span_naive > 180.0 {
            let mut east = f32::INFINITY;
            let mut west = f32::NEG_INFINITY;
            for &l in &lons {
                if l > 0.0 && l < east {
                    east = l;
                }
                if l < 0.0 && l > west {
                    west = l;
                }
            }
            if east.is_finite() && west.is_finite() {
                (east, west, true)
            } else {
                (-180.0, 180.0, false)
            }
        } else {
            (lon_min, lon_max, false)
        };

        LatLonBounds {
            lat_min,
            lat_max,
            lon_min: final_lon_min,
            lon_max: final_lon_max,
            wraps_antimeridian: wraps,
        }
    }
}

/// 3x3 row-major rotation matrix for rotation by `angle` (radians) around unit
/// `axis`. Rodrigues' formula.
pub fn axis_angle_matrix(axis: [f32; 3], angle: f32) -> [[f32; 3]; 3] {
    let (s, c) = angle.sin_cos();
    let t = 1.0 - c;
    let (x, y, z) = (axis[0], axis[1], axis[2]);
    [
        [t * x * x + c, t * x * y - s * z, t * x * z + s * y],
        [t * x * y + s * z, t * y * y + c, t * y * z - s * x],
        [t * x * z - s * y, t * y * z + s * x, t * z * z + c],
    ]
}

pub fn mat_mul(a: [[f32; 3]; 3], b: [[f32; 3]; 3]) -> [[f32; 3]; 3] {
    let mut out = [[0.0_f32; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            out[i][j] = a[i][0] * b[0][j] + a[i][1] * b[1][j] + a[i][2] * b[2][j];
        }
    }
    out
}

pub fn mat_apply(m: [[f32; 3]; 3], v: [f32; 3]) -> [f32; 3] {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}

impl Camera {
    /// Build the 3x3 rotation matrix `R_z(roll) · R_x(pitch) · R_y(yaw)` matching `rotate()`.
    pub fn rotation_matrix(&self) -> [[f32; 3]; 3] {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        let (sr, cr) = self.roll.sin_cos();
        let m = [
            [cy, 0.0, sy],
            [sp * sy, cp, -sp * cy],
            [-cp * sy, sp, cp * cy],
        ];
        [
            [
                cr * m[0][0] - sr * m[1][0],
                cr * m[0][1] - sr * m[1][1],
                cr * m[0][2] - sr * m[1][2],
            ],
            [
                sr * m[0][0] + cr * m[1][0],
                sr * m[0][1] + cr * m[1][1],
                sr * m[0][2] + cr * m[1][2],
            ],
            [m[2][0], m[2][1], m[2][2]],
        ]
    }

    /// Decompose `M = R_z(roll) · R_x(pitch) · R_y(yaw)` back to `(yaw, pitch, roll)`.
    pub fn set_from_matrix(&mut self, m: [[f32; 3]; 3]) {
        let sp = m[2][1].clamp(-1.0, 1.0);
        let pitch = sp.asin().clamp(-PITCH_LIMIT, PITCH_LIMIT);
        let cp = pitch.cos();
        if cp.abs() < 1e-6 {
            return;
        }
        let yaw = (-m[2][0]).atan2(m[2][2]);
        let roll = (-m[0][1]).atan2(m[1][1]);
        if yaw.is_finite() && roll.is_finite() {
            self.yaw = yaw;
            self.pitch = pitch;
            self.roll = roll;
        }
    }
}

/// Inverse Mercator: tile-Y → latitude (degrees) at a given LOD.
pub fn tile_y_to_lat(y: f32, num_tiles: f32) -> f32 {
    let n = PI - 2.0 * PI * y / num_tiles;
    (180.0 / PI) * n.sinh().atan()
}

/// Forward Mercator: latitude (degrees) → tile-Y at a given LOD.
pub fn lat_to_tile_y(lat_deg: f32, num_tiles: f32) -> f32 {
    let lat = lat_deg.clamp(-85.0511, 85.0511).to_radians();
    let y = (1.0 - (lat.tan() + 1.0 / lat.cos()).ln() / PI) / 2.0;
    y * num_tiles
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::Vec2;

    fn test_viewport() -> Rect {
        Rect::from_center_size(Pos2::new(100.0, 100.0), Vec2::splat(200.0))
    }

    fn test_camera() -> Camera {
        Camera {
            yaw: 0.0,
            pitch: 0.0,
            roll: 0.0,
            distance: 2.0,
            fov_y: DEFAULT_FOV_Y,
        }
    }

    fn assert_pinned(camera: &Camera, world_pt: [f32; 3], target: Pos2, viewport: Rect) {
        let projected = camera
            .world_to_screen(world_pt, viewport)
            .expect("world_pt should be visible");
        let delta = (projected - target).length();
        assert!(
            delta < 0.5,
            "pin failed: projected {projected:?}, expected {target:?}, delta {delta}",
        );
    }

    #[test]
    fn rotate_to_pin_drag_down_from_center() {
        let mut camera = test_camera();
        let viewport = test_viewport();
        let world_pt = camera.screen_to_world(viewport.center(), viewport).unwrap();
        let target = Pos2::new(100.0, 110.0);
        camera.rotate_to_pin(world_pt, target, viewport);
        assert_pinned(&camera, world_pt, target, viewport);
    }

    #[test]
    fn rotate_to_pin_drag_horizontal_from_center() {
        let mut camera = test_camera();
        let viewport = test_viewport();
        let world_pt = camera.screen_to_world(viewport.center(), viewport).unwrap();
        let target = Pos2::new(115.0, 100.0);
        camera.rotate_to_pin(world_pt, target, viewport);
        assert_pinned(&camera, world_pt, target, viewport);
    }

    #[test]
    fn rotate_to_pin_off_center_point_to_center() {
        let mut camera = test_camera();
        let viewport = test_viewport();
        let world_pt = lat_lon_to_world(30.0, 0.0);
        let start_screen = camera.world_to_screen(world_pt, viewport).unwrap();
        let target = viewport.center();
        assert!((start_screen - target).length() > 1.0);
        camera.rotate_to_pin(world_pt, target, viewport);
        assert_pinned(&camera, world_pt, target, viewport);
    }

    #[test]
    fn rotate_to_pin_idempotent_when_already_pinned() {
        let mut camera = Camera {
            yaw: 0.3,
            pitch: -0.1,
            roll: 0.0,
            distance: 2.0,
            fov_y: DEFAULT_FOV_Y,
        };
        let viewport = test_viewport();
        let world_pt = lat_lon_to_world(10.0, 25.0);
        let target = camera.world_to_screen(world_pt, viewport).unwrap();
        let (yaw_before, pitch_before) = (camera.yaw, camera.pitch);
        camera.rotate_to_pin(world_pt, target, viewport);
        assert!((camera.yaw - yaw_before).abs() < 1e-3);
        assert!((camera.pitch - pitch_before).abs() < 1e-3);
        assert_pinned(&camera, world_pt, target, viewport);
    }

    #[test]
    fn rotate_to_pin_unreachable_target_leaves_camera_unchanged() {
        let mut camera = test_camera();
        let viewport = test_viewport();
        let world_pt = lat_lon_to_world(89.9, 0.0); // near north pole, tiny R_xz
        let target = Pos2::new(180.0, 100.0); // 80 px right of center — inside limb
        let (yaw_before, pitch_before) = (camera.yaw, camera.pitch);
        camera.rotate_to_pin(world_pt, target, viewport);
        assert_eq!(camera.yaw, yaw_before);
        assert_eq!(camera.pitch, pitch_before);
    }

    #[test]
    fn rotate_to_pin_preserves_roll() {
        let mut camera = Camera {
            yaw: 0.2,
            pitch: 0.1,
            roll: 0.4,
            distance: 2.0,
            fov_y: DEFAULT_FOV_Y,
        };
        let viewport = test_viewport();
        let world_pt = camera.screen_to_world(viewport.center(), viewport).unwrap();
        let target = Pos2::new(112.0, 93.0);
        camera.rotate_to_pin(world_pt, target, viewport);
        assert!((camera.roll - 0.4).abs() < 1e-5);
        assert_pinned(&camera, world_pt, target, viewport);
    }

    #[test]
    fn screen_to_world_inverts_world_to_screen() {
        let camera = Camera {
            yaw: 0.3,
            pitch: 0.2,
            roll: 0.1,
            distance: 2.0,
            fov_y: DEFAULT_FOV_Y,
        };
        let viewport = test_viewport();
        let world_pt = lat_lon_to_world(20.0, 15.0);
        let screen = camera.world_to_screen(world_pt, viewport).unwrap();
        let recovered = camera.screen_to_world(screen, viewport).unwrap();
        for i in 0..3 {
            assert!(
                (world_pt[i] - recovered[i]).abs() < 1e-4,
                "round-trip failed at [{i}]: {} vs {}",
                world_pt[i],
                recovered[i],
            );
        }
    }

    #[test]
    fn ray_sphere_miss_returns_none() {
        let camera = test_camera();
        let viewport = test_viewport();
        // Far outside the limb (limb is at ~100 px radius with d=2, fov=60°).
        let far_outside = Pos2::new(100.0 + 150.0, 100.0);
        assert!(camera.screen_to_world(far_outside, viewport).is_none());
    }

    #[test]
    fn matrix_roundtrip_preserves_camera() {
        for &(yaw, pitch, roll) in &[
            (0.0f32, 0.0, 0.0),
            (0.5, 0.3, -0.2),
            (-1.2, 0.7, 1.1),
            (3.0, -1.3, 0.5),
        ] {
            let original = Camera {
                yaw,
                pitch,
                roll,
                distance: 2.0,
                fov_y: DEFAULT_FOV_Y,
            };
            let m = original.rotation_matrix();
            let mut recovered = Camera::default();
            recovered.set_from_matrix(m);
            let m2 = recovered.rotation_matrix();
            for r in 0..3 {
                for c in 0..3 {
                    assert!(
                        (m[r][c] - m2[r][c]).abs() < 1e-4,
                        "matrix roundtrip diverged at [{r}][{c}]: {} vs {}",
                        m[r][c],
                        m2[r][c],
                    );
                }
            }
        }
    }

    #[test]
    fn rotation_matrix_matches_rotate() {
        let camera = Camera {
            yaw: 0.7,
            pitch: -0.3,
            roll: 0.5,
            distance: 2.0,
            fov_y: DEFAULT_FOV_Y,
        };
        let m = camera.rotation_matrix();
        for p in &[
            [1.0_f32, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            lat_lon_to_world(40.0, 10.0),
        ] {
            let from_rotate = camera.rotate(*p);
            let from_matrix = mat_apply(m, *p);
            for i in 0..3 {
                assert!(
                    (from_rotate[i] - from_matrix[i]).abs() < 1e-5,
                    "rotate vs matrix disagree at {i}: {} vs {}",
                    from_rotate[i],
                    from_matrix[i],
                );
            }
        }
    }

    #[test]
    fn axis_angle_keeps_axis_fixed() {
        let axis = [0.6, 0.8, 0.0];
        let m = axis_angle_matrix(axis, 1.2);
        let rotated = mat_apply(m, axis);
        for i in 0..3 {
            assert!((rotated[i] - axis[i]).abs() < 1e-5);
        }
    }

    #[test]
    fn orbit_spin_preserves_anchor_screen_position() {
        let camera0 = Camera {
            yaw: 0.4,
            pitch: 0.2,
            roll: 0.1,
            distance: 2.0,
            fov_y: DEFAULT_FOV_Y,
        };
        let viewport = test_viewport();
        let anchor = lat_lon_to_world(15.0, -22.0);
        let original_screen = camera0.world_to_screen(anchor, viewport).unwrap();

        let spin_axis = camera0.rotate(anchor);
        for &theta in &[0.3_f32, -0.8, 1.5, -2.4] {
            let r_spin = axis_angle_matrix(spin_axis, theta);
            let new_matrix = mat_mul(r_spin, camera0.rotation_matrix());
            let mut camera = Camera::default();
            camera.set_from_matrix(new_matrix);
            camera.distance = camera0.distance;
            camera.fov_y = camera0.fov_y;
            let new_screen = camera.world_to_screen(anchor, viewport).unwrap();
            let delta = (new_screen - original_screen).length();
            assert!(
                delta < 0.5,
                "spin at θ={theta} moved anchor by {delta} (was {original_screen:?}, now {new_screen:?})",
            );
        }
    }

    #[test]
    fn pick_lod_increases_as_distance_approaches_one() {
        let viewport = Rect::from_min_size(Pos2::ZERO, Vec2::splat(500.0));
        let lod_far = super::super::tile_grid::pick_lod(
            &Camera {
                distance: 5.0,
                ..Camera::default()
            },
            viewport,
        );
        let lod_near = super::super::tile_grid::pick_lod(
            &Camera {
                distance: 1.1,
                ..Camera::default()
            },
            viewport,
        );
        assert!(
            lod_near > lod_far,
            "expected LOD to increase closer to surface: near={lod_near}, far={lod_far}",
        );
    }
}
