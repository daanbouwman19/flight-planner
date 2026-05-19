pub mod camera;
pub mod interaction;
pub mod providers;
pub mod renderer;
pub mod state;
pub mod tile_grid;
pub mod tile_manager;

use eframe::egui::{self, Color32, Vec2};

use camera::{Camera, DEFAULT_FOV_Y, MIN_DISTANCE, MAX_DISTANCE};
use state::GlobeState;
use tile_manager::SharedTileManager;

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
        let tile_manager: SharedTileManager = ui
            .data(|d| d.get_temp(tile_manager_id))
            .unwrap_or_else(|| {
                let manager = SharedTileManager::new(ui.ctx().clone());
                ui.data_mut(|d| d.insert_temp(tile_manager_id, manager.clone()));
                manager
            });

        interaction::update(&mut state, &response, rect);

        let lod = tile_grid::pick_lod(&state.camera, rect);
        let tiles = tile_grid::visible_tiles(&state.camera, rect, lod);

        // Pre-fetch base levels so something is always renderable.
        for (z, x, y) in [(0, 0, 0), (1, 0, 0), (1, 0, 1), (1, 1, 0), (1, 1, 1)] {
            tile_manager.trigger_fetch(z, x, y);
        }

        let painter = ui.painter_at(rect);
        renderer::draw_tiles(&painter, &state.camera, rect, &tiles, lod, &tile_manager);
        renderer::draw_route(&painter, &state.camera, rect, p1, p2);
        renderer::draw_point(&painter, &state.camera, rect, p1, Color32::GREEN, "DEP");
        renderer::draw_point(&painter, &state.camera, rect, p2, Color32::RED, "DEST");
        renderer::draw_globe_outline(&painter, &state.camera, rect);

        let time = ui.input(|i| i.time);
        tile_manager.decay_stats(time);
        let stats = tile_manager.stats();
        renderer::draw_debug_overlay(&painter, rect, &stats, &state.camera, lod);

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

fn initial_state(
    p1: [f32; 3],
    p2: [f32; 3],
    start_lat_lon: (f64, f64),
    end_lat_lon: (f64, f64),
) -> GlobeState {
    let dot = p1[0] * p2[0] + p1[1] * p2[1] + p1[2] * p2[2];
    let theta = dot.clamp(-1.0, 1.0).acos();

    // Convert great-circle distance to camera distance: close routes get low distance
    // (zoomed in), long routes get high distance (whole globe visible).
    let inv_zoom = if theta < 0.01 {
        4.0_f32
    } else {
        (0.9 / (theta / 2.0).sin()).clamp(1.0, 8.0)
    };
    let distance = (1.0 + 1.5 / inv_zoom).clamp(MIN_DISTANCE, MAX_DISTANCE);

    let avg_lat = (start_lat_lon.0 + end_lat_lon.0) / 2.0;
    let avg_lon = (start_lat_lon.1 + end_lat_lon.1) / 2.0;

    GlobeState {
        camera: Camera {
            yaw: -(avg_lon as f32).to_radians(),
            pitch: (avg_lat as f32).to_radians(),
            roll: 0.0,
            distance,
            fov_y: DEFAULT_FOV_Y,
        },
        last_p1: p1,
        last_p2: p2,
        drag: None,
    }
}
