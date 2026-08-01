use eframe::egui::{Pos2, Rect};
use std::ops::{Add, Mul, Neg, Sub};

/// Vertical field of view: 60 degrees.
pub const DEFAULT_FOV_Y: f32 = std::f32::consts::FRAC_PI_3;
/// Camera is just above the surface at this distance (street-level view).
pub const MIN_DISTANCE: f32 = 1.0001;
/// Whole globe with breathing room.
pub const MAX_DISTANCE: f32 = 10.0;

// ---------------------------------------------------------------------------
// Minimal 3-D vector type shared within the globe module.
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Vec3(pub(super) f32, pub(super) f32, pub(super) f32);

impl Vec3 {
    #[inline]
    pub(super) fn dot(self, o: Vec3) -> f32 {
        self.0 * o.0 + self.1 * o.1 + self.2 * o.2
    }

    #[inline]
    pub(super) fn cross(self, o: Vec3) -> Vec3 {
        Vec3(
            self.1 * o.2 - self.2 * o.1,
            self.2 * o.0 - self.0 * o.2,
            self.0 * o.1 - self.1 * o.0,
        )
    }

    #[inline]
    pub(super) fn normalize(self) -> Vec3 {
        let len = self.dot(self).sqrt();
        if len < 1e-10 {
            self
        } else {
            self * (1.0 / len)
        }
    }
}

impl From<[f32; 3]> for Vec3 {
    #[inline]
    fn from(a: [f32; 3]) -> Vec3 {
        Vec3(a[0], a[1], a[2])
    }
}

impl From<Vec3> for [f32; 3] {
    #[inline]
    fn from(v: Vec3) -> [f32; 3] {
        [v.0, v.1, v.2]
    }
}

impl Add for Vec3 {
    type Output = Vec3;
    #[inline]
    fn add(self, o: Vec3) -> Vec3 {
        Vec3(self.0 + o.0, self.1 + o.1, self.2 + o.2)
    }
}

impl Sub for Vec3 {
    type Output = Vec3;
    #[inline]
    fn sub(self, o: Vec3) -> Vec3 {
        Vec3(self.0 - o.0, self.1 - o.1, self.2 - o.2)
    }
}

impl Mul<f32> for Vec3 {
    type Output = Vec3;
    #[inline]
    fn mul(self, s: f32) -> Vec3 {
        Vec3(self.0 * s, self.1 * s, self.2 * s)
    }
}

impl Neg for Vec3 {
    type Output = Vec3;
    #[inline]
    fn neg(self) -> Vec3 {
        Vec3(-self.0, -self.1, -self.2)
    }
}

// ---------------------------------------------------------------------------
// Pre-computed camera basis — amortizes basis() across all per-vertex calls.
// ---------------------------------------------------------------------------

/// Cached camera basis vectors, valid for one camera state.
/// Compute once per frame with `Camera::compute_basis()`, then pass to
/// `rotate_fast` / `facing_value_fast` to avoid redundant sin/cos per vertex.
pub struct CameraBasis {
    right: Vec3,
    up: Vec3,
    look: Vec3,
    position: Vec3,
    /// position / (1 + altitude) — facing_value_fast reduces to a single dot product.
    facing_unit: Vec3,
}

impl CameraBasis {
    /// Unit vector from the globe centre toward the camera (P/|P|).
    pub(super) fn facing_unit(&self) -> Vec3 {
        self.facing_unit
    }
}

/// Rotate a world point into camera space using a pre-computed basis.
#[inline]
pub fn rotate_fast(basis: &CameraBasis, w: [f32; 3]) -> [f32; 3] {
    let d = Vec3::from(w) - basis.position;
    [d.dot(basis.right), d.dot(basis.up), d.dot(basis.look)]
}

/// Back-face culling value using a pre-computed basis (single dot product).
#[inline]
pub fn facing_value_fast(basis: &CameraBasis, w: [f32; 3]) -> f32 {
    Vec3::from(w).dot(basis.facing_unit)
}

// ---------------------------------------------------------------------------
// Camera
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    /// Latitude of the screen-centre nadir, degrees.
    pub center_lat: f32,
    /// Longitude of the screen-centre nadir, degrees.
    pub center_lon: f32,
    /// Camera height above the unit-sphere surface.
    pub altitude: f32,
    /// Bearing: 0 = north up, positive = clockwise, radians.
    pub bearing: f32,
    /// Tilt: 0 = top-down, max ~PI/2.5, radians.
    pub tilt: f32,
    /// Vertical field of view, radians.
    pub fov_y: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            center_lat: 0.0,
            center_lon: 0.0,
            altitude: 2.0,
            bearing: 0.0,
            tilt: 0.0,
            fov_y: DEFAULT_FOV_Y,
        }
    }
}

impl Camera {
    /// Compute the camera basis vectors.
    ///
    /// Returns `(right, up, look, position)` all in world space:
    /// - `right`: screen-right direction (tilt-independent)
    /// - `up`: screen-up direction
    /// - `look`: into-scene direction (unit vector)
    /// - `position`: camera world position, |P| = 1 + altitude
    fn basis(&self) -> (Vec3, Vec3, Vec3, Vec3) {
        let lat = self.center_lat.to_radians();
        let lon = self.center_lon.to_radians();
        let (slat, clat) = lat.sin_cos();
        let (slon, clon) = lon.sin_cos();

        // Nadir: unit vector pointing from origin toward center_lat/lon on sphere.
        let n = Vec3(clat * slon, slat, clat * clon);

        // North and east tangents at the nadir.
        let north_n = Vec3(-slat * slon, clat, -slat * clon);
        let east_n = Vec3(clon, 0.0, -slon);

        let (sb, cb) = self.bearing.sin_cos();
        // Screen-up direction at tilt=0 (bearing-rotated north).
        let up_base = north_n * cb + east_n * sb;

        // Screen-right is tilt-independent.
        let right = up_base.cross(n).normalize();

        let (st, ct) = self.tilt.sin_cos();

        // Into-scene look direction and tilted screen-up.
        let look = (n * -ct + up_base * st).normalize();
        let up = (up_base * ct + n * st).normalize();

        // Camera position on sphere of radius r = 1 + altitude.
        // t = camera-to-nadir distance, derived from |P| = r with P on the look ray.
        let r = 1.0 + self.altitude;
        let t = -ct + (r * r - st * st).max(0.0).sqrt();
        let position = n * (1.0 + t * ct) + up_base * (-t * st);

        (right, up, look, position)
    }

    /// Compute and cache the camera basis vectors for this frame.
    /// Pass the result to `rotate_fast` / `facing_value_fast` to avoid
    /// recomputing sin/cos once per vertex.
    pub fn compute_basis(&self) -> CameraBasis {
        let (right, up, look, position) = self.basis();
        let r = 1.0 + self.altitude;
        CameraBasis {
            right,
            up,
            look,
            position,
            facing_unit: position * (1.0 / r),
        }
    }

    /// Focal length in pixels: half the viewport height divided by tan(fov_y/2).
    pub fn focal_pixels(&self, viewport_height: f32) -> f32 {
        (viewport_height * 0.5) / (self.fov_y * 0.5).tan()
    }

    /// Project a world point `w` into camera space relative to the camera position `P`.
    /// Returns `[x_cam, y_cam, depth]` where depth is along the look direction.
    pub fn rotate(&self, w: [f32; 3]) -> [f32; 3] {
        let (right, up, look, position) = self.basis();
        let d = Vec3::from(w) - position;
        [d.dot(right), d.dot(up), d.dot(look)]
    }

    /// Project a camera-space point (post-rotate) to screen using perspective.
    /// Returns `None` when the point is behind the camera plane.
    pub fn project(&self, cam_pt: [f32; 3], viewport: Rect) -> Option<Pos2> {
        let depth = cam_pt[2];
        if depth <= 1e-6 {
            return None;
        }
        let f = self.focal_pixels(viewport.height());
        let c = viewport.center();
        Some(Pos2::new(
            c.x + f * cam_pt[0] / depth,
            c.y - f * cam_pt[1] / depth,
        ))
    }

    pub fn world_to_screen(&self, w: [f32; 3], viewport: Rect) -> Option<Pos2> {
        self.project(self.rotate(w), viewport)
    }

    /// Back-face culling value: dot(w, P/|P|).
    /// For tilt=0 this equals the old `rotated[2]` — same values at nadir (1.0) and limb (1/r).
    pub fn facing_value(&self, w: [f32; 3]) -> f32 {
        let r = 1.0 + self.altitude;
        let (_, _, _, position) = self.basis();
        Vec3::from(w).dot(position) / r
    }

    /// Cull threshold: tiles/points with facing_value below this are back-facing.
    pub fn cull_threshold(&self) -> f32 {
        // CULLING_FADE_MARGIN widens the kept set just past the geometric horizon so
        // limb tiles can fade out smoothly rather than popping off abruptly.
        const CULLING_FADE_MARGIN: f32 = 0.3;
        1.0 / (1.0 + self.altitude) - CULLING_FADE_MARGIN
    }

    /// Ray–sphere intersection from the camera position.
    /// Returns the world-space hit point on the unit sphere, or `None` if the ray misses.
    pub fn screen_to_world(&self, cursor: Pos2, viewport: Rect) -> Option<[f32; 3]> {
        let f = self.focal_pixels(viewport.height());
        let c = viewport.center();
        let ix = (cursor.x - c.x) / f;
        let iy = -(cursor.y - c.y) / f;

        let (right, up, look, position) = self.basis();

        // Ray direction in world space: ix*right + iy*up + look (unnormalized).
        let dir = right * ix + up * iy + look;

        let a = dir.dot(dir);
        let b = position.dot(dir);
        let c_val = position.dot(position) - 1.0;

        let disc = b * b - a * c_val;
        if disc < 0.0 {
            return None;
        }
        let t = (-b - disc.sqrt()) / a;
        if t < 0.0 {
            return None;
        }
        Some((position + dir * t).into())
    }

    /// Like `screen_to_world` but clamps the image-plane coordinates to the limb
    /// boundary so drags starting outside the globe disc still work.
    pub fn screen_to_world_clamped(&self, cursor: Pos2, viewport: Rect) -> [f32; 3] {
        let f = self.focal_pixels(viewport.height());
        let c = viewport.center();
        let mut ix = (cursor.x - c.x) / f;
        let mut iy = -(cursor.y - c.y) / f;

        let r = 1.0 + self.altitude;
        // Limb boundary for tilt=0: ix²+iy²+1 <= r²/(r²-1), equivalently ix²+iy² <= 1/(r²-1).
        let limb_r2 = 1.0 / (r * r - 1.0).max(1e-6);
        let r2 = ix * ix + iy * iy;
        if r2 > limb_r2 {
            let s = (limb_r2 / r2).sqrt();
            ix *= s;
            iy *= s;
        }

        let (right, up, look, position) = self.basis();
        let dir = right * ix + up * iy + look;

        let a = dir.dot(dir);
        let b = position.dot(dir);
        let c_val = position.dot(position) - 1.0;

        let disc = (b * b - a * c_val).max(0.0);
        let t = (-b - disc.sqrt()) / a;
        (position + dir * t).into()
    }

    /// Adjust `center_lat` and `center_lon` so `world_pt` projects to `target_screen`.
    /// Uses Newton's method (4 iterations). Does not change tilt, bearing, altitude, or fov_y.
    pub fn pan_to(&mut self, world_pt: [f32; 3], target_screen: Pos2, viewport: Rect) {
        const EPS: f32 = 0.005; // degrees
        for _ in 0..4 {
            let Some(cur) = self.world_to_screen(world_pt, viewport) else {
                return;
            };
            let err = cur - target_screen;
            if err.length() < 0.5 {
                break;
            }

            // Partial derivatives w.r.t. center_lat and center_lon.
            let mut cam_lat = *self;
            cam_lat.center_lat += EPS;
            let Some(s_lat) = cam_lat.world_to_screen(world_pt, viewport) else {
                return;
            };
            let dlat = (s_lat - cur) / EPS;

            let mut cam_lon = *self;
            cam_lon.center_lon += EPS;
            let Some(s_lon) = cam_lon.world_to_screen(world_pt, viewport) else {
                return;
            };
            let dlon = (s_lon - cur) / EPS;

            // Solve 2×2 linear system: dlat*d_lat + dlon*d_lon = -err
            let det = dlat.x * dlon.y - dlat.y * dlon.x;
            if det.abs() < 1e-10 {
                break;
            }
            let neg_err = -err;
            let d_lat_deg = (dlon.y * neg_err.x - dlon.x * neg_err.y) / det;
            let d_lon_deg = (dlat.x * neg_err.y - dlat.y * neg_err.x) / det;

            self.center_lat = (self.center_lat + d_lat_deg).clamp(-85.0, 85.0);
            self.center_lon += d_lon_deg;
        }
    }
}

// ---------------------------------------------------------------------------
// Supporting free functions
// ---------------------------------------------------------------------------

pub fn lat_lon_to_world(lat_deg: f32, lon_deg: f32) -> [f32; 3] {
    let lat = lat_deg.to_radians();
    let lon = lon_deg.to_radians();
    [lat.cos() * lon.sin(), lat.sin(), lat.cos() * lon.cos()]
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::Vec2;

    fn test_viewport() -> Rect {
        Rect::from_center_size(Pos2::new(400.0, 400.0), Vec2::splat(800.0))
    }

    fn test_camera() -> Camera {
        Camera {
            center_lat: 0.0,
            center_lon: 0.0,
            altitude: 2.0,
            bearing: 0.0,
            tilt: 0.0,
            fov_y: DEFAULT_FOV_Y,
        }
    }

    /// With tilt=0, the nadir (center_lat/lon point) must project to the viewport center.
    #[test]
    fn tilt_zero_matches_untilted() {
        let camera = test_camera();
        let viewport = test_viewport();
        let nadir = lat_lon_to_world(0.0, 0.0);
        let screen = camera
            .world_to_screen(nadir, viewport)
            .expect("nadir should be visible");
        let center = viewport.center();
        assert!(
            (screen - center).length() < 0.5,
            "nadir should project to screen center: got {screen:?}, expected {center:?}",
        );

        // An off-center point should project somewhere else.
        let off = lat_lon_to_world(30.0, 30.0);
        let off_screen = camera
            .world_to_screen(off, viewport)
            .expect("off-center point should be visible");
        assert!(
            (off_screen - center).length() > 10.0,
            "off-center point should not be at center",
        );
    }

    /// pan_to should bring a world point to the target screen position.
    #[test]
    fn pan_to_pins_world_point() {
        let viewport = test_viewport();

        // Test at tilt=0.
        let mut camera = test_camera();
        let world_pt = lat_lon_to_world(30.0, 20.0);
        let target = Pos2::new(450.0, 350.0);
        camera.pan_to(world_pt, target, viewport);
        let projected = camera
            .world_to_screen(world_pt, viewport)
            .expect("world_pt should be visible after pan");
        assert!(
            (projected - target).length() < 1.0,
            "pan_to tilt=0: projected {projected:?}, target {target:?}",
        );

        // Test at tilt=0.4.
        let mut camera2 = Camera {
            tilt: 0.4,
            ..test_camera()
        };
        let world_pt2 = lat_lon_to_world(10.0, 15.0);
        let target2 = Pos2::new(420.0, 380.0);
        camera2.pan_to(world_pt2, target2, viewport);
        let projected2 = camera2
            .world_to_screen(world_pt2, viewport)
            .expect("world_pt should be visible after pan (tilt=0.4)");
        assert!(
            (projected2 - target2).length() < 1.0,
            "pan_to tilt=0.4: projected {projected2:?}, target {target2:?}",
        );
    }

    /// screen_to_world followed by world_to_screen should recover the original screen position.
    #[test]
    fn screen_to_world_roundtrip() {
        let camera = Camera {
            center_lat: 10.0,
            center_lon: 20.0,
            altitude: 2.0,
            bearing: 0.2,
            tilt: 0.3,
            fov_y: DEFAULT_FOV_Y,
        };
        let viewport = test_viewport();
        let screen = Pos2::new(410.0, 390.0);
        let world = camera
            .screen_to_world(screen, viewport)
            .expect("center-ish point should hit sphere");
        let back = camera
            .world_to_screen(world, viewport)
            .expect("re-projected world point should be visible");
        assert!(
            (back - screen).length() < 1e-2,
            "round-trip failed: orig {screen:?}, recovered {back:?}",
        );
    }

    /// With tilt>0 toward north (bearing=0), a north point should appear closer to screen
    /// center than with tilt=0 — the camera has rotated to look more toward the horizon.
    #[test]
    fn tilt_vertical_drag_moves_tilt() {
        let viewport = test_viewport();
        let north_pt = lat_lon_to_world(40.0, 0.0);

        let cam0 = test_camera();
        let cam_tilted = Camera {
            tilt: 0.5,
            ..test_camera()
        };

        let s0 = cam0
            .world_to_screen(north_pt, viewport)
            .expect("north point visible at tilt=0");
        let s1 = cam_tilted
            .world_to_screen(north_pt, viewport)
            .expect("north point visible at tilt=0.5");

        let center = viewport.center();
        let dist0 = (s0 - center).length();
        let dist1 = (s1 - center).length();
        assert!(
            dist1 < dist0,
            "tilting north should bring north point closer to screen center: dist_before={dist0:.1}, dist_after={dist1:.1}",
        );
    }

    /// At tilt=0, facing_value at the nadir should equal ~1.0 (camera points directly at nadir).
    #[test]
    fn facing_value_at_nadir() {
        let camera = test_camera();
        let nadir = lat_lon_to_world(0.0, 0.0);
        let fv = camera.facing_value(nadir);
        assert!(
            (fv - 1.0).abs() < 1e-4,
            "facing_value at nadir (tilt=0) should be ~1.0, got {fv}",
        );
    }

    /// With tilt>0, bearing rotation should change which direction "up" is on screen.
    #[test]
    fn bearing_changes_screen_up() {
        let viewport = test_viewport();
        let north = lat_lon_to_world(10.0, 0.0);

        let cam0 = Camera {
            tilt: 0.4,
            bearing: 0.0,
            ..test_camera()
        };
        let cam_rot = Camera {
            tilt: 0.4,
            bearing: 0.5,
            ..test_camera()
        };

        let s0 = cam0.world_to_screen(north, viewport);
        let s1 = cam_rot.world_to_screen(north, viewport);

        if let (Some(p0), Some(p1)) = (s0, s1) {
            let diff = (p0 - p1).length();
            assert!(
                diff > 1.0,
                "bearing rotation should move screen position of north point: diff={diff}",
            );
        }
    }

    /// rotate_fast and facing_value_fast should match the non-fast versions.
    #[test]
    fn fast_variants_match_slow() {
        let camera = Camera {
            center_lat: 15.0,
            center_lon: -30.0,
            altitude: 1.5,
            bearing: 0.3,
            tilt: 0.2,
            fov_y: DEFAULT_FOV_Y,
        };
        let basis = camera.compute_basis();
        let pts = [
            lat_lon_to_world(0.0, 0.0),
            lat_lon_to_world(15.0, -30.0),
            lat_lon_to_world(-45.0, 90.0),
        ];
        for w in pts {
            let r_slow = camera.rotate(w);
            let r_fast = rotate_fast(&basis, w);
            for i in 0..3 {
                assert!(
                    (r_slow[i] - r_fast[i]).abs() < 1e-5,
                    "rotate_fast mismatch at index {i}: slow={}, fast={}",
                    r_slow[i],
                    r_fast[i]
                );
            }
            let fv_slow = camera.facing_value(w);
            let fv_fast = facing_value_fast(&basis, w);
            assert!(
                (fv_slow - fv_fast).abs() < 1e-5,
                "facing_value_fast mismatch: slow={fv_slow}, fast={fv_fast}",
            );
        }
    }
}
