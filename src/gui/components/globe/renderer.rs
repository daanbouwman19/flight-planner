use eframe::egui::{self, Color32, Painter, Pos2, Rect, Shape, Stroke, Vec2};

use super::camera::{
    Camera, CameraBasis, facing_value_fast, lat_lon_to_world, rotate_fast, tile_y_to_lat,
};
use super::tile_grid::VisibleTile;
use super::tile_manager::{SharedTileManager, TileStats};

const TILE_SUBSTEPS_MAX: usize = 6;
const ATMOSPHERE_ALPHA: u8 = 30;
const GLOBE_OUTLINE_WIDTH: f32 = 2.0;
const ROUTE_STROKE_WIDTH: f32 = 3.0;
const POINT_RADIUS: f32 = 4.0;

/// Substep count scales with LOD: zoomed-out tiles are tiny on screen and need
/// fewer curve samples; close-up tiles need more to round the earth smoothly.
#[inline]
fn substeps_for_lod(lod: u8) -> usize {
    match lod {
        0..=3 => 2,
        4..=6 => 4,
        _ => TILE_SUBSTEPS_MAX,
    }
}

pub fn draw_tiles(
    painter: &Painter,
    camera: &Camera,
    basis: &CameraBasis,
    viewport: Rect,
    tiles: &[VisibleTile],
    lod: u8,
    manager: &SharedTileManager,
) {
    let num_tiles = 1u32 << lod;
    let num_tiles_f = num_tiles as f32;
    let threshold = camera.cull_threshold();
    let substeps = substeps_for_lod(lod);
    let stride = substeps + 1;

    for tile in tiles {
        let Some((texture, uv)) = manager.get_best_tile(lod, tile.x, tile.y) else {
            continue;
        };

        let mut mesh = egui::Mesh::with_texture(texture.id());
        let mut any_behind = false;

        for sy in 0..=substeps {
            for sx in 0..=substeps {
                let f_x = sx as f32 / substeps as f32;
                let f_y = sy as f32 / substeps as f32;

                let lon = tile.lon_min + f_x * (tile.lon_max - tile.lon_min);
                let lat = tile_y_to_lat(tile.y as f32 + f_y, num_tiles_f);

                let w = lat_lon_to_world(lat, lon);
                let alpha = ((facing_value_fast(basis, w) - threshold) * 5.0).clamp(0.0, 1.0);
                let rotated = rotate_fast(basis, w);

                let Some(screen_p) = camera.project(rotated, viewport) else {
                    any_behind = true;
                    break;
                };

                let u = uv[0] + f_x * (uv[2] - uv[0]);
                let v = uv[1] + f_y * (uv[3] - uv[1]);

                mesh.vertices.push(egui::epaint::Vertex {
                    pos: screen_p,
                    uv: Pos2::new(u, v),
                    color: Color32::from_rgba_unmultiplied(255, 255, 255, (alpha * 255.0) as u8),
                });
            }
            if any_behind {
                break;
            }
        }

        if any_behind {
            continue;
        }

        for sy in 0..substeps {
            for sx in 0..substeps {
                let i = sy * stride + sx;
                mesh.indices.extend_from_slice(&[
                    i as u32,
                    (i + 1) as u32,
                    (i + stride) as u32,
                    (i + 1) as u32,
                    (i + stride + 1) as u32,
                    (i + stride) as u32,
                ]);
            }
        }
        painter.add(Shape::mesh(mesh));
    }
}

/// Draw the great-circle route from pre-computed slerp points.
/// `route_points` must be empty when there is no route (theta < threshold).
pub fn draw_route(
    painter: &Painter,
    camera: &Camera,
    basis: &CameraBasis,
    viewport: Rect,
    route_points: &[[f32; 3]],
) {
    if route_points.is_empty() {
        return;
    }

    let threshold = camera.cull_threshold();
    let mut last_p: Option<Pos2> = None;
    let stroke = Stroke::new(ROUTE_STROKE_WIDTH, Color32::from_rgb(255, 200, 0));

    for &p in route_points {
        if facing_value_fast(basis, p) > threshold {
            let rotated = rotate_fast(basis, p);
            if let Some(screen_p) = camera.project(rotated, viewport) {
                if let Some(prev) = last_p {
                    painter.line_segment([prev, screen_p], stroke);
                }
                last_p = Some(screen_p);
            } else {
                last_p = None;
            }
        } else {
            last_p = None;
        }
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

pub fn draw_globe_outline(painter: &Painter, camera: &Camera, viewport: Rect) {
    // The visible limb projects to a circle of radius focal_pixels / sqrt(d²-1).
    let f = camera.focal_pixels(viewport.height());
    let d = 1.0 + camera.altitude;
    let limb_r = f / (d * d - 1.0).max(0.001).sqrt();
    let center = viewport.center();
    painter.circle_stroke(
        center,
        limb_r,
        Stroke::new(GLOBE_OUTLINE_WIDTH, Color32::WHITE),
    );
    painter.circle_stroke(
        center,
        limb_r + 2.0,
        Stroke::new(
            1.0,
            Color32::from_rgba_unmultiplied(100, 200, 255, ATMOSPHERE_ALPHA),
        ),
    );
}

pub fn draw_debug_overlay(
    painter: &Painter,
    viewport: Rect,
    stats: &TileStats,
    camera: &Camera,
    lod: u8,
) {
    let debug_rect = Rect::from_min_size(
        viewport.min + Vec2::new(10.0, 10.0),
        Vec2::new(130.0, 100.0),
    );
    painter.rect_filled(debug_rect, 4.0, Color32::from_black_alpha(150));

    let text = format!(
        "LOD: {}\nDist: {:.3}\nHits: {}\nMiss: {}\nErr: {}\nPend: {}\nCache: {}",
        lod,
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
