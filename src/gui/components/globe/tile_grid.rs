use eframe::egui::Rect;

use super::camera::{Camera, MAX_LOD, TILE_PX, lat_lon_to_world, lat_to_tile_y, tile_y_to_lat};

#[derive(Clone, Copy, Debug)]
pub struct VisibleTile {
    pub x: u32,
    pub y: u32,
    pub lon_min: f32,
    pub lon_max: f32,
    pub lat_min: f32,
    pub lat_max: f32,
    /// Rotated z of the tile center — used by the caller to sort back-to-front.
    pub center_rotated_z: f32,
}

/// Pick the LOD whose texel density best matches the front of the sphere
/// (the closest visible point, depth ≈ altitude). Saturates at `MAX_LOD`.
pub fn pick_lod(camera: &Camera, viewport: Rect) -> u8 {
    let f = camera.focal_pixels(viewport.height());
    let depth_front = camera.altitude.max(1e-5);
    let circumference_px = 2.0 * std::f32::consts::PI * f / depth_front;
    let ideal = (circumference_px / TILE_PX).log2();
    if !ideal.is_finite() {
        return 0;
    }
    ideal.round().clamp(0.0, MAX_LOD as f32) as u8
}

/// All tiles at `lod` that intersect the viewport's lat/lon bounds and aren't
/// fully back-facing. Returns them sorted back-to-front by rotated-z.
pub fn visible_tiles(camera: &Camera, viewport: Rect, lod: u8) -> Vec<VisibleTile> {
    let bounds = camera.visible_lat_lon_bounds(viewport);
    let num_tiles = 1u32 << lod;
    let num_tiles_f = num_tiles as f32;

    let y_min_f = lat_to_tile_y(bounds.lat_max, num_tiles_f).floor();
    let y_max_f = lat_to_tile_y(bounds.lat_min, num_tiles_f).ceil();
    let ty_start = (y_min_f as i64).clamp(0, num_tiles as i64 - 1) as u32;
    let ty_end = (y_max_f as i64).clamp(0, num_tiles as i64 - 1) as u32;

    let lon_to_x = |lon: f32| ((lon + 180.0) / 360.0) * num_tiles_f;

    let x_ranges: Vec<(u32, u32)> = if bounds.wraps_antimeridian {
        let east_start = lon_to_x(bounds.lon_min).floor() as i64;
        let east_end = num_tiles as i64 - 1;
        let west_start = 0i64;
        let west_end = lon_to_x(bounds.lon_max).ceil() as i64;
        vec![
            (
                east_start.clamp(0, num_tiles as i64 - 1) as u32,
                east_end as u32,
            ),
            (
                west_start as u32,
                west_end.clamp(0, num_tiles as i64 - 1) as u32,
            ),
        ]
    } else {
        let x_start = lon_to_x(bounds.lon_min).floor() as i64;
        let x_end = lon_to_x(bounds.lon_max).ceil() as i64;
        vec![(
            x_start.clamp(0, num_tiles as i64 - 1) as u32,
            x_end.clamp(0, num_tiles as i64 - 1) as u32,
        )]
    };

    let threshold = camera.cull_threshold();
    let mut out: Vec<VisibleTile> = Vec::new();

    for (x_lo, x_hi) in x_ranges {
        for ty in ty_start..=ty_end {
            for tx in x_lo..=x_hi {
                let lon_min = (tx as f32 / num_tiles_f) * 360.0 - 180.0;
                let lon_max = ((tx + 1) as f32 / num_tiles_f) * 360.0 - 180.0;
                let lat_max = tile_y_to_lat(ty as f32, num_tiles_f);
                let lat_min = tile_y_to_lat((ty + 1) as f32, num_tiles_f);

                let center = lat_lon_to_world((lat_min + lat_max) * 0.5, (lon_min + lon_max) * 0.5);
                let fv = camera.facing_value(center);
                if fv < threshold {
                    continue;
                }

                out.push(VisibleTile {
                    x: tx,
                    y: ty,
                    lon_min,
                    lon_max,
                    lat_min,
                    lat_max,
                    center_rotated_z: fv,
                });
            }
        }
    }

    out.sort_by(|a, b| a.center_rotated_z.total_cmp(&b.center_rotated_z));
    out
}
