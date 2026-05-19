use eframe::egui::{self, Color32, Painter, Pos2, Rect, Shape, Stroke, Vec2};

use super::camera::{Camera, lat_lon_to_world, tile_y_to_lat};
use super::tile_grid::{VisibleTile, cull_threshold};
use super::tile_manager::{SharedTileManager, TileStats};

const TILE_SUBSTEPS: usize = 6;
const ATMOSPHERE_ALPHA: u8 = 30;
const GLOBE_OUTLINE_WIDTH: f32 = 2.0;
const ROUTE_STROKE_WIDTH: f32 = 3.0;
const POINT_RADIUS: f32 = 4.0;

pub fn draw_tiles(
    painter: &Painter,
    camera: &Camera,
    viewport: Rect,
    tiles: &[VisibleTile],
    lod: u8,
    manager: &SharedTileManager,
) {
    let num_tiles = 1u32 << lod;
    let num_tiles_f = num_tiles as f32;
    let threshold = cull_threshold(camera.distance);

    for tile in tiles {
        let Some((texture, uv)) = manager.get_best_tile(lod, tile.x, tile.y) else {
            continue;
        };

        let mut mesh = egui::Mesh::with_texture(texture.id());
        let stride = TILE_SUBSTEPS + 1;
        let mut any_behind = false;

        for sy in 0..=TILE_SUBSTEPS {
            for sx in 0..=TILE_SUBSTEPS {
                let f_x = sx as f32 / TILE_SUBSTEPS as f32;
                let f_y = sy as f32 / TILE_SUBSTEPS as f32;

                let lon = tile.lon_min + f_x * (tile.lon_max - tile.lon_min);
                let lat = tile_y_to_lat(tile.y as f32 + f_y, num_tiles_f);

                let rotated = camera.rotate(lat_lon_to_world(lat, lon));
                let alpha = ((rotated[2] - threshold) * 5.0).clamp(0.0, 1.0);

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

        for sy in 0..TILE_SUBSTEPS {
            for sx in 0..TILE_SUBSTEPS {
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

pub fn draw_route(painter: &Painter, camera: &Camera, viewport: Rect, p1: [f32; 3], p2: [f32; 3]) {
    let dot = p1[0] * p2[0] + p1[1] * p2[1] + p1[2] * p2[2];
    let theta = dot.clamp(-1.0, 1.0).acos();
    if theta < 0.001 {
        return;
    }

    let threshold = cull_threshold(camera.distance);
    let steps = (theta.to_degrees() as usize).clamp(10, 100);
    let mut last_p: Option<Pos2> = None;
    let stroke = Stroke::new(ROUTE_STROKE_WIDTH, Color32::from_rgb(255, 200, 0));
    let sin_theta = theta.sin();

    for i in 0..=steps {
        let f = i as f32 / steps as f32;
        let a = ((1.0 - f) * theta).sin() / sin_theta;
        let b = (f * theta).sin() / sin_theta;
        let p = [
            a * p1[0] + b * p2[0],
            a * p1[1] + b * p2[1],
            a * p1[2] + b * p2[2],
        ];

        let rotated = camera.rotate(p);
        if rotated[2] > threshold {
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
    viewport: Rect,
    p: [f32; 3],
    color: Color32,
    label: &str,
) {
    let threshold = cull_threshold(camera.distance);
    let rotated = camera.rotate(p);
    if rotated[2] <= threshold {
        return;
    }
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
    let d = camera.distance;
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
        camera.distance,
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
