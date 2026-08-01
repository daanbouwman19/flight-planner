pub mod camera;
pub mod interaction;
pub mod providers;
pub mod quadtree;
pub mod renderer;
pub mod state;
pub mod tile_manager;

use std::sync::Arc;

use eframe::egui::{self, Color32, Vec2};

use camera::{Camera, DEFAULT_FOV_Y, MAX_DISTANCE, MIN_DISTANCE};
use state::{GlobeState, MAX_ALTITUDE, MIN_ALTITUDE};
use tile_manager::SharedTileManager;

/// Maximum slerp samples along the great-circle route (one per degree of arc).
const MAX_ROUTE_STEPS: usize = 100;

pub struct Globe;

impl Globe {
    pub fn render(
        ui: &mut egui::Ui,
        id: egui::Id,
        start_lat_lon: (f64, f64),
        end_lat_lon: (f64, f64),
    ) {
        let available_width = ui.available_width();
        let size = Vec2::splat(available_width);
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::drag());

        let p1 = camera::lat_lon_to_world(start_lat_lon.0 as f32, start_lat_lon.1 as f32);
        let p2 = camera::lat_lon_to_world(end_lat_lon.0 as f32, end_lat_lon.1 as f32);

        let mut state: GlobeState = ui
            .data(|d| d.get_temp(id))
            .unwrap_or_else(|| initial_state(p1, p2, start_lat_lon, end_lat_lon));

        if state.last_p1 != p1 || state.last_p2 != p2 {
            state = initial_state(p1, p2, start_lat_lon, end_lat_lon);
        }

        let tile_manager_id = ui.make_persistent_id("tile_manager");
        let tile_manager: SharedTileManager =
            ui.data(|d| d.get_temp(tile_manager_id)).unwrap_or_else(|| {
                let manager = SharedTileManager::new(ui.ctx().clone());
                ui.data_mut(|d| d.insert_temp(tile_manager_id, manager.clone()));
                manager
            });

        interaction::update(&mut state, &response, rect);

        let camera = &state.camera;
        // Compute the camera basis once for all per-vertex operations this frame.
        let basis = camera.compute_basis();

        // Select tiles by walking the quadtree; every visited node is also
        // queued for fetching so imagery refines coarse-to-fine.
        let tiles = quadtree::collect_visible_tiles(camera, &basis, rect, |z, x, y| {
            tile_manager.request(z, x, y);
        });

        let painter = ui.painter_at(rect);
        renderer::draw_globe_backdrop(&painter, camera, &basis, rect);
        renderer::draw_tiles(&painter, camera, &basis, rect, &tiles, &tile_manager);
        renderer::draw_route(&painter, camera, &basis, rect, &state.route_points);
        renderer::draw_point(&painter, camera, &basis, rect, p1, Color32::GREEN, "DEP");
        renderer::draw_point(&painter, camera, &basis, rect, p2, Color32::RED, "DEST");
        renderer::draw_globe_rim(&painter, camera, &basis, rect);
        renderer::draw_attribution(&painter, rect);

        let stats = tile_manager.stats();
        if stats.pending > 0 {
            renderer::draw_loading_indicator(&painter, rect);
            // Tile arrivals repaint via the egui context; this only guards
            // against a stalled queue leaving the indicator stale.
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(250));
        }

        #[cfg(debug_assertions)]
        {
            tile_manager.decay_stats(ui.input(|i| i.time));
            renderer::draw_debug_overlay(&painter, rect, &stats, camera, &tiles);
        }

        let button_rect =
            egui::Rect::from_min_size(rect.max - Vec2::new(40.0, 40.0), Vec2::new(30.0, 30.0));
        let recenter_clicked = ui
            .put(
                button_rect,
                egui::Button::new(crate::gui::icons::ICON_RECENTER)
                    .fill(Color32::from_black_alpha(150)),
            )
            .on_hover_text("Recenter on route")
            .clicked();
        if recenter_clicked {
            state = initial_state(p1, p2, start_lat_lon, end_lat_lon);
        }

        ui.data_mut(|d| d.insert_temp(id, state));
    }
}

/// Slerp between `p1` and `p2` with roughly one sample per degree of arc.
/// Returns an empty slice when the points coincide.
fn compute_route_points(p1: [f32; 3], p2: [f32; 3]) -> Arc<[[f32; 3]]> {
    let dot = p1[0] * p2[0] + p1[1] * p2[1] + p1[2] * p2[2];
    let theta = dot.clamp(-1.0, 1.0).acos();
    if theta < 0.001 {
        return Vec::new().into();
    }
    let steps = (theta.to_degrees() as usize).clamp(10, MAX_ROUTE_STEPS);
    let sin_theta = theta.sin();
    let mut points = Vec::with_capacity(steps + 1);
    for i in 0..=steps {
        let f = i as f32 / steps as f32;
        let a = ((1.0 - f) * theta).sin() / sin_theta;
        let b = (f * theta).sin() / sin_theta;
        points.push([
            a * p1[0] + b * p2[0],
            a * p1[1] + b * p2[1],
            a * p1[2] + b * p2[2],
        ]);
    }
    points.into()
}

fn initial_state(
    p1: [f32; 3],
    p2: [f32; 3],
    start_lat_lon: (f64, f64),
    end_lat_lon: (f64, f64),
) -> GlobeState {
    let dot = p1[0] * p2[0] + p1[1] * p2[1] + p1[2] * p2[2];
    let theta = dot.clamp(-1.0, 1.0).acos();

    let inv_zoom = if theta < 0.01 {
        4.0_f32
    } else {
        (0.9 / (theta / 2.0).sin()).clamp(1.0, 8.0)
    };
    // INITIAL_ZOOM_FACTOR controls how far out the camera starts relative to the route arc.
    const INITIAL_ZOOM_FACTOR: f32 = 1.5;
    let distance = (1.0 + INITIAL_ZOOM_FACTOR / inv_zoom).clamp(MIN_DISTANCE, MAX_DISTANCE);

    let avg_lat = ((start_lat_lon.0 + end_lat_lon.0) / 2.0) as f32;
    let avg_lon = ((start_lat_lon.1 + end_lat_lon.1) / 2.0) as f32;

    GlobeState {
        camera: Camera {
            center_lat: avg_lat,
            center_lon: avg_lon,
            altitude: (distance - 1.0).clamp(MIN_ALTITUDE, MAX_ALTITUDE),
            bearing: 0.0,
            tilt: 0.0,
            fov_y: DEFAULT_FOV_Y,
        },
        last_p1: p1,
        last_p2: p2,
        drag: None,
        route_points: compute_route_points(p1, p2),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use camera::lat_lon_to_world;

    #[test]
    fn route_points_span_endpoints() {
        let p1 = lat_lon_to_world(0.0, 0.0);
        let p2 = lat_lon_to_world(0.0, 90.0);
        let points = compute_route_points(p1, p2);
        // 90 degrees of arc → 90 steps → 91 samples.
        assert_eq!(points.len(), 91);
        let first = points.first().unwrap();
        let last = points.last().unwrap();
        for i in 0..3 {
            assert!((first[i] - p1[i]).abs() < 1e-5, "start point mismatch");
            assert!((last[i] - p2[i]).abs() < 1e-5, "end point mismatch");
        }
    }

    #[test]
    fn route_is_empty_for_coincident_points() {
        let p = lat_lon_to_world(45.0, 45.0);
        assert!(compute_route_points(p, p).is_empty());
    }

    #[test]
    fn route_stays_on_unit_sphere() {
        let p1 = lat_lon_to_world(52.3, 4.8); // Amsterdam-ish
        let p2 = lat_lon_to_world(-33.9, 151.2); // Sydney-ish
        for p in compute_route_points(p1, p2).iter() {
            let len = (p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt();
            assert!(
                (len - 1.0).abs() < 1e-4,
                "slerp point off the sphere: {len}"
            );
        }
    }
}
