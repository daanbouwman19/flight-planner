use eframe::egui::{Pos2, Rect};
use std::f32::consts::PI;

/// Vertical field of view: 60 degrees.
pub const DEFAULT_FOV_Y: f32 = std::f32::consts::FRAC_PI_3;
/// Camera is just above the surface at this distance (street-level view).
pub const MIN_DISTANCE: f32 = 1.0001;
/// Whole globe with breathing room.
pub const MAX_DISTANCE: f32 = 10.0;
pub const MAX_LOD: u8 = 18;
pub const TILE_PX: f32 = 256.0;

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = dot(v, v).sqrt();
    if len < 1e-10 {
        v
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

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
    fn basis(&self) -> ([f32; 3], [f32; 3], [f32; 3], [f32; 3]) {
        let lat = self.center_lat.to_radians();
        let lon = self.center_lon.to_radians();
        let (slat, clat) = lat.sin_cos();
        let (slon, clon) = lon.sin_cos();

        // Nadir: unit vector pointing from origin toward center_lat/lon on sphere.
        let n = [clat * slon, slat, clat * clon];

        // North tangent at nadir.
        let north_n = [-slat * slon, clat, -slat * clon];
        // East tangent at nadir.
        let east_n = [clon, 0.0, -slon];

        let (sb, cb) = self.bearing.sin_cos();
        // Screen-up direction at tilt=0 (bearing-rotated north).
        let up_base = add(scale(north_n, cb), scale(east_n, sb));

        // Screen-right is tilt-independent.
        let right = normalize(cross(up_base, n));

        let (st, ct) = self.tilt.sin_cos();

        // Into-scene look direction.
        let look = add(scale(n, -ct), scale(up_base, st));
        let look = normalize(look);

        // Screen-up tilted.
        let up = add(scale(up_base, ct), scale(n, st));
        let up = normalize(up);

        // Camera position on sphere of radius r = 1 + altitude.
        let r = 1.0 + self.altitude;
        // t = camera-to-nadir distance (along the tilted look-ray from P to nadir N).
        // |P| = r, nadir is at distance 1 from origin.
        // P = r*N_tilt_offset, derived from: P = N*(1 + t*ct) - t*st*up_base where |P|=r.
        let t = -ct + (r * r - st * st).max(0.0).sqrt();
        let position = add(scale(n, 1.0 + t * ct), scale(up_base, -t * st));

        (right, up, look, position)
    }

    /// Focal length in pixels: half the viewport height divided by tan(fov_y/2).
    pub fn focal_pixels(&self, viewport_height: f32) -> f32 {
        (viewport_height * 0.5) / (self.fov_y * 0.5).tan()
    }

    /// Project a world point `w` into camera space relative to the camera position `P`.
    /// Returns `[x_cam, y_cam, depth]` where depth is along the look direction.
    pub fn rotate(&self, w: [f32; 3]) -> [f32; 3] {
        let (right, up, look, position) = self.basis();
        let d = sub(w, position);
        [dot(d, right), dot(d, up), dot(d, look)]
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
        dot(w, position) / r
    }

    /// Cull threshold: tiles/points with facing_value below this are back-facing.
    pub fn cull_threshold(&self) -> f32 {
        1.0 / (1.0 + self.altitude) - 0.3
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
        let dir = add(add(scale(right, ix), scale(up, iy)), look);

        let a = dot(dir, dir);
        let b = dot(position, dir);
        let c_val = dot(position, position) - 1.0;

        let disc = b * b - a * c_val;
        if disc < 0.0 {
            return None;
        }
        let t = (-b - disc.sqrt()) / a;
        if t < 0.0 {
            return None;
        }
        Some(add(position, scale(dir, t)))
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
        let dir = add(add(scale(right, ix), scale(up, iy)), look);

        let a = dot(dir, dir);
        let b = dot(position, dir);
        let c_val = dot(position, position) - 1.0;

        let disc = (b * b - a * c_val).max(0.0);
        let t = (-b - disc.sqrt()) / a;
        add(position, scale(dir, t))
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

            // Partial derivative w.r.t. center_lat.
            let mut cam_lat = *self;
            cam_lat.center_lat += EPS;
            let Some(s_lat) = cam_lat.world_to_screen(world_pt, viewport) else {
                return;
            };
            let dlat = (s_lat - cur) / EPS;

            // Partial derivative w.r.t. center_lon.
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

    /// Approximate lat/lon bounds visible from the current camera.
    pub fn visible_lat_lon_bounds(&self, viewport: Rect) -> LatLonBounds {
        let center = viewport.center();
        let f = self.focal_pixels(viewport.height());
        let r = 1.0 + self.altitude;
        // Approximate limb screen radius (exact for tilt=0).
        let limb_r = f * (r * r - 1.0).max(1e-4).sqrt() / r;

        let mut points: Vec<Pos2> = Vec::with_capacity(41);

        let viewport_half = viewport.size().min_elem() * 0.5;
        if limb_r < viewport_half {
            // Globe fits inside the viewport: sample near the limb circle.
            for i in 0..16u32 {
                let angle = i as f32 * std::f32::consts::TAU / 16.0;
                let rr = 0.95 * limb_r;
                points.push(Pos2::new(
                    center.x + rr * angle.cos(),
                    center.y + rr * angle.sin(),
                ));
            }
            for i in 0..8u32 {
                let angle = i as f32 * std::f32::consts::TAU / 8.0;
                let rr = 0.50 * limb_r;
                points.push(Pos2::new(
                    center.x + rr * angle.cos(),
                    center.y + rr * angle.sin(),
                ));
            }
        }
        points.push(center);

        // Viewport border samples.
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
        let screen = camera.world_to_screen(nadir, viewport).expect("nadir should be visible");
        let center = viewport.center();
        assert!(
            (screen - center).length() < 0.5,
            "nadir should project to screen center: got {screen:?}, expected {center:?}",
        );

        // An off-center point should project somewhere else.
        let off = lat_lon_to_world(30.0, 30.0);
        let off_screen = camera.world_to_screen(off, viewport).expect("off-center point should be visible");
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
        // Use a point near the center so it's guaranteed to hit.
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
        let north = lat_lon_to_world(10.0, 0.0); // point north of equator

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

        // After bearing rotation, the north point should be at a different screen position.
        if let (Some(p0), Some(p1)) = (s0, s1) {
            let diff = (p0 - p1).length();
            assert!(
                diff > 1.0,
                "bearing rotation should move screen position of north point: diff={diff}",
            );
        }
    }

    #[test]
    fn pick_lod_increases_as_altitude_approaches_zero() {
        let viewport = Rect::from_min_size(Pos2::ZERO, Vec2::splat(500.0));
        let lod_far = super::super::tile_grid::pick_lod(
            &Camera {
                altitude: 4.0,
                ..Camera::default()
            },
            viewport,
        );
        let lod_near = super::super::tile_grid::pick_lod(
            &Camera {
                altitude: 0.1,
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
