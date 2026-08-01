//! Screen-space-error driven tile selection.
//!
//! Instead of picking one global LOD and enumerating every tile inside the
//! visible lat/lon bounding box (which explodes combinatorially at high zoom
//! and over-fetches the horizon at nadir resolution), the visible set is found
//! by walking the slippy-map quadtree from the root: a tile is subdivided only
//! while its projected screen size exceeds a threshold and it survives horizon
//! and viewport culling. This naturally yields high detail at the nadir,
//! coarse tiles at the horizon, and a tile count bounded by what is actually
//! on screen.

use eframe::egui::{Pos2, Rect};
use std::collections::VecDeque;
use std::f32::consts::PI;

use super::camera::{Camera, CameraBasis, facing_value_fast, lat_lon_to_world, rotate_fast};

/// Deepest zoom level requested from the imagery provider.
const MAX_LOD: u8 = 18;
/// Native tile texture edge, pixels.
const TILE_PX: f32 = 256.0;
/// A tile subdivides while its projected size exceeds this. Slightly above the
/// texture size so rendered tiles land between ~50% and ~112% texel density.
const SPLIT_SCREEN_PX: f32 = TILE_PX * 1.125;
/// Hard cap on rendered tiles per frame. BFS order means hitting the cap
/// degrades detail uniformly instead of dropping regions.
const MAX_VISIBLE_TILES: usize = 256;
/// Extra angle (radians) beyond the horizon cone before a tile is culled, so
/// limb tiles can alpha-fade out instead of popping.
const HORIZON_CULL_MARGIN: f32 = 0.08;
/// Viewport culling is only applied to tiles smaller than this angular radius;
/// larger tiles curve too much for a screen-space AABB to be meaningful.
const VIEWPORT_CULL_MAX_ANG_RADIUS: f32 = 1.0;
/// Floor for the camera→tile distance used in the screen-size estimate.
const MIN_DEPTH: f32 = 1e-4;

/// A tile selected for rendering this frame.
#[derive(Clone, Copy, Debug)]
pub struct VisibleTile {
    pub z: u8,
    pub x: u32,
    pub y: u32,
    pub lon_min: f32,
    pub lon_max: f32,
    /// Distance from the camera to the tile centre, for back-to-front sorting.
    pub depth: f32,
}

/// Inverse Mercator: tile-Y → latitude (degrees) at a given LOD.
pub fn tile_y_to_lat(y: f32, num_tiles: f32) -> f32 {
    let n = PI - 2.0 * PI * y / num_tiles;
    (180.0 / PI) * n.sinh().atan()
}

/// Walk the quadtree breadth-first and return the tiles to render, sorted
/// back-to-front. `request` is called for every visited (non-culled) node —
/// leaves *and* their ancestors — so fetches arrive coarse-to-fine and every
/// leaf always has a cached ancestor to fall back on while it loads.
pub fn collect_visible_tiles(
    camera: &Camera,
    basis: &CameraBasis,
    viewport: Rect,
    mut request: impl FnMut(u8, u32, u32),
) -> Vec<VisibleTile> {
    let r = 1.0 + camera.altitude;
    let frame = Frame {
        camera,
        basis,
        viewport,
        focal: camera.focal_pixels(viewport.height()),
        // Angle from the camera nadir axis beyond which sphere points are hidden.
        horizon_angle: (1.0 / r).clamp(-1.0, 1.0).acos(),
    };

    let mut leaves: Vec<VisibleTile> = Vec::new();
    let mut queue: VecDeque<(u8, u32, u32)> = VecDeque::new();
    queue.push_back((0, 0, 0));

    while let Some((z, x, y)) = queue.pop_front() {
        let Some(m) = tile_metrics(&frame, z, x, y) else {
            continue;
        };
        request(z, x, y);

        // Emitting a leaf keeps `leaves + queue` constant and culling shrinks
        // it, so gating subdivision (net +3 nodes) on the prospective total
        // keeps the final leaf count at or under the budget. BFS order means
        // running out of budget degrades detail uniformly across the view.
        let within_budget = leaves.len() + queue.len() + 4 <= MAX_VISIBLE_TILES;

        if z < MAX_LOD && within_budget && m.screen_px > SPLIT_SCREEN_PX {
            let (cx, cy) = (x * 2, y * 2);
            queue.push_back((z + 1, cx, cy));
            queue.push_back((z + 1, cx + 1, cy));
            queue.push_back((z + 1, cx, cy + 1));
            queue.push_back((z + 1, cx + 1, cy + 1));
        } else {
            leaves.push(VisibleTile {
                z,
                x,
                y,
                lon_min: m.lon_min,
                lon_max: m.lon_max,
                depth: m.depth,
            });
        }
    }

    leaves.sort_by(|a, b| b.depth.total_cmp(&a.depth));
    leaves
}

/// Per-frame values shared by every node visit during one traversal.
struct Frame<'a> {
    camera: &'a Camera,
    basis: &'a CameraBasis,
    viewport: Rect,
    focal: f32,
    horizon_angle: f32,
}

struct TileMetrics {
    screen_px: f32,
    depth: f32,
    lon_min: f32,
    lon_max: f32,
}

/// Cull `(z, x, y)` against the horizon cone and the viewport, and estimate
/// its projected screen size. Returns `None` when the tile cannot be visible.
fn tile_metrics(frame: &Frame, z: u8, x: u32, y: u32) -> Option<TileMetrics> {
    let num_tiles = (1u32 << z) as f32;
    let lon_min = (x as f32 / num_tiles) * 360.0 - 180.0;
    let lon_max = ((x + 1) as f32 / num_tiles) * 360.0 - 180.0;
    let lat_n = tile_y_to_lat(y as f32, num_tiles);
    let lat_s = tile_y_to_lat((y + 1) as f32, num_tiles);
    let lat_c = (lat_n + lat_s) * 0.5;
    let lon_c = (lon_min + lon_max) * 0.5;

    // World-space angular extents of the tile (radians).
    let ew_arc = (lon_max - lon_min).to_radians() * lat_c.to_radians().cos();
    let ns_arc = (lat_n - lat_s).to_radians();

    // Conservative angular radius: the flat half-diagonal underestimates
    // spherical distance for near-hemisphere tiles, so inflate and cap.
    let ang_radius = ((ew_arc * ew_arc + ns_arc * ns_arc).sqrt() * 0.625).min(PI);

    // Horizon cull: the tile is hidden when even its nearest edge lies beyond
    // the horizon cone (plus a margin for the limb fade).
    let center = lat_lon_to_world(lat_c, lon_c);
    let cos_center = facing_value_fast(frame.basis, center).clamp(-1.0, 1.0);
    if cos_center.acos() - ang_radius > frame.horizon_angle + HORIZON_CULL_MARGIN {
        return None;
    }

    let cam = rotate_fast(frame.basis, center);
    let dist = (cam[0] * cam[0] + cam[1] * cam[1] + cam[2] * cam[2]).sqrt();
    // World-space extent across the tile's larger axis.
    let chord = 2.0 * (ew_arc.max(ns_arc).min(PI) * 0.5).sin();
    // Nearest possible camera→tile distance; the nadir at `altitude` is a
    // global lower bound for the whole sphere.
    let near = (dist - chord * 0.5)
        .max(frame.camera.altitude)
        .max(MIN_DEPTH);
    let screen_px = chord * frame.focal / near;

    if ang_radius < VIEWPORT_CULL_MAX_ANG_RADIUS
        && !projects_into_viewport(
            frame,
            screen_px,
            [lat_n, lat_c, lat_s],
            [lon_min, lon_c, lon_max],
        )
    {
        return None;
    }

    Some(TileMetrics {
        screen_px,
        depth: dist,
        lon_min,
        lon_max,
    })
}

/// Conservative viewport test: project a 3×3 sample grid of the tile and check
/// whether its screen AABB (padded for edge curvature) touches the viewport.
/// Any sample behind the camera keeps the tile.
fn projects_into_viewport(frame: &Frame, screen_px: f32, lats: [f32; 3], lons: [f32; 3]) -> bool {
    let mut min = Pos2::new(f32::INFINITY, f32::INFINITY);
    let mut max = Pos2::new(f32::NEG_INFINITY, f32::NEG_INFINITY);
    for lat in lats {
        for lon in lons {
            let cam = rotate_fast(frame.basis, lat_lon_to_world(lat, lon));
            let Some(p) = frame.camera.project(cam, frame.viewport) else {
                return true;
            };
            min = min.min(p);
            max = max.max(p);
        }
    }
    // Padding for the curved bulge of tile edges between sample points.
    let margin = screen_px * 0.35 + 24.0;
    Rect::from_min_max(min, max)
        .expand(margin)
        .intersects(frame.viewport)
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::Vec2;

    fn viewport() -> Rect {
        Rect::from_min_size(Pos2::ZERO, Vec2::splat(800.0))
    }

    fn collect(camera: &Camera) -> (Vec<VisibleTile>, Vec<(u8, u32, u32)>) {
        let basis = camera.compute_basis();
        let mut requested = Vec::new();
        let leaves = collect_visible_tiles(camera, &basis, viewport(), |z, x, y| {
            requested.push((z, x, y));
        });
        (leaves, requested)
    }

    #[test]
    fn produces_tiles_and_respects_budget() {
        for altitude in [0.0002, 0.01, 0.2, 1.0, 4.0, 9.0] {
            for tilt in [0.0, 0.6, 1.2] {
                let camera = Camera {
                    center_lat: 45.0,
                    center_lon: -60.0,
                    altitude,
                    tilt,
                    ..Camera::default()
                };
                let (leaves, _) = collect(&camera);
                assert!(
                    !leaves.is_empty(),
                    "no tiles at altitude={altitude}, tilt={tilt}"
                );
                assert!(
                    leaves.len() <= MAX_VISIBLE_TILES,
                    "budget exceeded at altitude={altitude}, tilt={tilt}: {}",
                    leaves.len()
                );
            }
        }
    }

    #[test]
    fn zooming_in_increases_detail() {
        let far = Camera {
            altitude: 2.0,
            ..Camera::default()
        };
        let near = Camera {
            altitude: 0.05,
            ..Camera::default()
        };
        let max_z = |leaves: &[VisibleTile]| leaves.iter().map(|t| t.z).max().unwrap();
        let (far_leaves, _) = collect(&far);
        let (near_leaves, _) = collect(&near);
        assert!(
            max_z(&near_leaves) > max_z(&far_leaves),
            "expected deeper subdivision when closer: near={}, far={}",
            max_z(&near_leaves),
            max_z(&far_leaves)
        );
    }

    #[test]
    fn leaves_are_unique() {
        let camera = Camera {
            altitude: 0.3,
            tilt: 0.8,
            ..Camera::default()
        };
        let (leaves, _) = collect(&camera);
        let mut keys: Vec<_> = leaves.iter().map(|t| (t.z, t.x, t.y)).collect();
        keys.sort_unstable();
        let before = keys.len();
        keys.dedup();
        assert_eq!(before, keys.len(), "duplicate leaves in visible set");
    }

    #[test]
    fn ancestors_of_every_leaf_are_requested() {
        let camera = Camera {
            center_lat: 30.0,
            center_lon: 10.0,
            altitude: 0.1,
            ..Camera::default()
        };
        let (leaves, requested) = collect(&camera);
        let requested: std::collections::HashSet<_> = requested.into_iter().collect();
        for leaf in &leaves {
            let (mut z, mut x, mut y) = (leaf.z, leaf.x, leaf.y);
            while z > 0 {
                z -= 1;
                x /= 2;
                y /= 2;
                assert!(
                    requested.contains(&(z, x, y)),
                    "ancestor {z}/{x}/{y} of leaf {}/{}/{} was not requested",
                    leaf.z,
                    leaf.x,
                    leaf.y
                );
            }
        }
    }

    #[test]
    fn nadir_is_covered() {
        let camera = Camera {
            center_lat: 20.0,
            center_lon: 30.0,
            altitude: 0.5,
            ..Camera::default()
        };
        let (leaves, _) = collect(&camera);
        let covered = leaves.iter().any(|t| {
            let num_tiles = (1u32 << t.z) as f32;
            let lat_n = tile_y_to_lat(t.y as f32, num_tiles);
            let lat_s = tile_y_to_lat((t.y + 1) as f32, num_tiles);
            (lat_s..=lat_n).contains(&camera.center_lat)
                && (t.lon_min..=t.lon_max).contains(&camera.center_lon)
        });
        assert!(covered, "no leaf covers the camera nadir");
    }
}
