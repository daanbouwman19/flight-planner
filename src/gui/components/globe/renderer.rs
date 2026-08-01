use eframe::egui::{self, Color32, Painter, Pos2, Rect, Shape, Stroke, Vec2};

use super::camera::{Camera, CameraBasis, Vec3, facing_value_fast, lat_lon_to_world, rotate_fast};
use super::providers;
use super::quadtree::{VisibleTile, tile_y_to_lat};
use super::tile_manager::{SharedTileManager, TileStats};

const GLOBE_OUTLINE_WIDTH: f32 = 1.5;
const ROUTE_STROKE_WIDTH: f32 = 3.0;
const POINT_RADIUS: f32 = 4.0;
/// Vertex alpha ramps from 0 to 1 over this rate as the facing value rises
/// above the cull threshold, fading tiles out toward the limb.
const LIMB_FADE_RATE: f32 = 5.0;
/// Samples along the projected limb polyline.
const LIMB_SAMPLES: usize = 64;
/// Fill behind the tiles: hides hairline cracks between LOD levels and gives
/// the globe a silhouette before any imagery has loaded.
const BACKDROP_FILL: Color32 = Color32::from_rgb(8, 20, 38);
/// Light-blue haze along the limb: (100, 200, 255) at ~30% alpha, premultiplied.
const ATMOSPHERE_GLOW: Color32 = Color32::from_rgba_premultiplied(31, 63, 80, 80);

/// Tessellation density per tile. Coarse-LOD tiles span huge arcs and need
/// many segments to round the sphere; deep tiles are nearly flat.
fn substeps_for(z: u8) -> usize {
    match z {
        0 => 16,
        1 => 12,
        2 => 8,
        3 => 6,
        4 => 5,
        _ => 4,
    }
}

pub fn draw_tiles(
    painter: &Painter,
    camera: &Camera,
    basis: &CameraBasis,
    viewport: Rect,
    tiles: &[VisibleTile],
    manager: &SharedTileManager,
) {
    let threshold = camera.cull_threshold();

    for tile in tiles {
        let Some((texture, uv)) = manager.best_texture(tile.z, tile.x, tile.y) else {
            continue;
        };

        let substeps = substeps_for(tile.z);
        let stride = substeps + 1;
        let num_tiles_f = (1u32 << tile.z) as f32;

        let mut mesh = egui::Mesh::with_texture(texture.id());
        // Mesh index of each grid vertex; None when it is behind the camera.
        let mut grid: Vec<Option<u32>> = Vec::with_capacity(stride * stride);

        for sy in 0..=substeps {
            let f_y = sy as f32 / substeps as f32;
            let lat = tile_y_to_lat(tile.y as f32 + f_y, num_tiles_f);
            for sx in 0..=substeps {
                let f_x = sx as f32 / substeps as f32;
                let lon = tile.lon_min + f_x * (tile.lon_max - tile.lon_min);

                let w = lat_lon_to_world(lat, lon);
                let rotated = rotate_fast(basis, w);
                let Some(screen_p) = camera.project(rotated, viewport) else {
                    grid.push(None);
                    continue;
                };

                let alpha =
                    ((facing_value_fast(basis, w) - threshold) * LIMB_FADE_RATE).clamp(0.0, 1.0);
                let u = uv[0] + f_x * (uv[2] - uv[0]);
                let v = uv[1] + f_y * (uv[3] - uv[1]);

                grid.push(Some(mesh.vertices.len() as u32));
                mesh.vertices.push(egui::epaint::Vertex {
                    pos: screen_p,
                    uv: Pos2::new(u, v),
                    color: Color32::from_rgba_unmultiplied(255, 255, 255, (alpha * 255.0) as u8),
                });
            }
        }

        // Emit only cells whose four corners are in front of the camera, so a
        // tile partially behind the near plane still renders its visible part.
        for sy in 0..substeps {
            for sx in 0..substeps {
                let i = sy * stride + sx;
                if let (Some(a), Some(b), Some(c), Some(d)) =
                    (grid[i], grid[i + 1], grid[i + stride], grid[i + stride + 1])
                {
                    mesh.indices.extend_from_slice(&[a, b, c, b, d, c]);
                }
            }
        }

        if !mesh.indices.is_empty() {
            painter.add(Shape::mesh(mesh));
        }
    }
}

/// Draw the great-circle route from pre-computed slerp points as polyline
/// runs, split wherever the route dips behind the horizon.
pub fn draw_route(
    painter: &Painter,
    camera: &Camera,
    basis: &CameraBasis,
    viewport: Rect,
    route_points: &[[f32; 3]],
) {
    if route_points.len() < 2 {
        return;
    }

    let threshold = camera.cull_threshold();
    let stroke = Stroke::new(ROUTE_STROKE_WIDTH, Color32::from_rgb(255, 200, 0));
    let mut run: Vec<Pos2> = Vec::new();

    for &p in route_points {
        let screen = (facing_value_fast(basis, p) > threshold)
            .then(|| camera.project(rotate_fast(basis, p), viewport))
            .flatten();
        match screen {
            Some(s) => run.push(s),
            None => flush_polyline(painter, &mut run, stroke),
        }
    }
    flush_polyline(painter, &mut run, stroke);
}

fn flush_polyline(painter: &Painter, run: &mut Vec<Pos2>, stroke: Stroke) {
    if run.len() >= 2 {
        painter.add(Shape::line(std::mem::take(run), stroke));
    } else {
        run.clear();
    }
}

pub fn draw_point(
    painter: &Painter,
    camera: &Camera,
    basis: &CameraBasis,
    viewport: Rect,
    p: [f32; 3],
    color: Color32,
    label: &str,
) {
    let threshold = camera.cull_threshold();
    if facing_value_fast(basis, p) <= threshold {
        return;
    }
    let rotated = rotate_fast(basis, p);
    let Some(screen_p) = camera.project(rotated, viewport) else {
        return;
    };
    painter.circle_filled(screen_p, POINT_RADIUS, color);
    painter.circle_stroke(screen_p, POINT_RADIUS, Stroke::new(1.0, Color32::WHITE));
    painter.text(
        screen_p + Vec2::new(6.0, -6.0),
        egui::Align2::LEFT_BOTTOM,
        label,
        egui::FontId::proportional(12.0),
        Color32::WHITE,
    );
}

/// Project the true limb — the circle of sphere points at grazing angle from
/// the camera — into screen space. Exact under any bearing and tilt, unlike a
/// screen-space circle around the viewport centre. Points behind the camera
/// (possible at high tilt when zoomed in) are skipped.
fn limb_points(camera: &Camera, basis: &CameraBasis, viewport: Rect) -> Vec<Pos2> {
    let r = 1.0 + camera.altitude;
    let axis = basis.facing_unit();
    // The limb lies on a circle of radius rho around the nadir axis, at
    // height 1/r from the globe centre.
    let rho = (1.0 - 1.0 / (r * r)).max(0.0).sqrt();
    let ring_center = axis * (1.0 / r);
    let helper = if axis.0.abs() < 0.9 {
        Vec3(1.0, 0.0, 0.0)
    } else {
        Vec3(0.0, 1.0, 0.0)
    };
    let e1 = axis.cross(helper).normalize();
    let e2 = axis.cross(e1);

    let mut points = Vec::with_capacity(LIMB_SAMPLES);
    for i in 0..LIMB_SAMPLES {
        let t = i as f32 * std::f32::consts::TAU / LIMB_SAMPLES as f32;
        let (sin_t, cos_t) = t.sin_cos();
        let w: [f32; 3] = (ring_center + e1 * (rho * cos_t) + e2 * (rho * sin_t)).into();
        if let Some(p) = camera.project(rotate_fast(basis, w), viewport) {
            points.push(p);
        }
    }
    points
}

/// Filled disc up to the limb, drawn before the tiles.
pub fn draw_globe_backdrop(
    painter: &Painter,
    camera: &Camera,
    basis: &CameraBasis,
    viewport: Rect,
) {
    let points = limb_points(camera, basis, viewport);
    if points.len() >= 3 {
        painter.add(Shape::convex_polygon(points, BACKDROP_FILL, Stroke::NONE));
    }
}

/// Atmosphere glow and rim stroke along the limb, drawn after the tiles.
pub fn draw_globe_rim(painter: &Painter, camera: &Camera, basis: &CameraBasis, viewport: Rect) {
    let points = limb_points(camera, basis, viewport);
    if points.len() >= 3 {
        painter.add(Shape::closed_line(
            points.clone(),
            Stroke::new(5.0, ATMOSPHERE_GLOW),
        ));
        painter.add(Shape::closed_line(
            points,
            Stroke::new(GLOBE_OUTLINE_WIDTH, Color32::WHITE),
        ));
    }
}

/// Imagery attribution, bottom-left corner.
pub fn draw_attribution(painter: &Painter, viewport: Rect) {
    let galley = painter.layout_no_wrap(
        providers::ATTRIBUTION.to_owned(),
        egui::FontId::proportional(10.0),
        Color32::from_gray(220),
    );
    let pos = viewport.left_bottom() + Vec2::new(6.0, -6.0 - galley.size().y);
    painter.rect_filled(
        Rect::from_min_size(pos, galley.size()).expand(2.0),
        2.0,
        Color32::from_black_alpha(120),
    );
    painter.galley(pos, galley, Color32::from_gray(220));
}

/// Subtle indicator while tile fetches are in flight, top-right corner.
pub fn draw_loading_indicator(painter: &Painter, viewport: Rect) {
    let galley = painter.layout_no_wrap(
        "Loading imagery…".to_owned(),
        egui::FontId::proportional(11.0),
        Color32::from_gray(220),
    );
    let pos = viewport.right_top() + Vec2::new(-8.0 - galley.size().x, 8.0);
    painter.rect_filled(
        Rect::from_min_size(pos, galley.size()).expand(3.0),
        3.0,
        Color32::from_black_alpha(120),
    );
    painter.galley(pos, galley, Color32::from_gray(220));
}

pub fn draw_debug_overlay(
    painter: &Painter,
    viewport: Rect,
    stats: &TileStats,
    camera: &Camera,
    tiles: &[VisibleTile],
) {
    let debug_rect = Rect::from_min_size(
        viewport.min + Vec2::new(10.0, 10.0),
        Vec2::new(130.0, 112.0),
    );
    painter.rect_filled(debug_rect, 4.0, Color32::from_black_alpha(150));

    let max_z = tiles.iter().map(|t| t.z).max().unwrap_or(0);
    let text = format!(
        "Tiles: {} (z≤{})\nDist: {:.3}\nHits: {}\nMiss: {}\nErr: {}\nPend: {}\nCache: {}",
        tiles.len(),
        max_z,
        1.0 + camera.altitude,
        stats.hits,
        stats.misses,
        stats.errors,
        stats.pending,
        stats.cache_size,
    );
    painter.text(
        debug_rect.min + Vec2::new(5.0, 5.0),
        egui::Align2::LEFT_TOP,
        text,
        egui::FontId::monospace(10.0),
        Color32::WHITE,
    );
}
