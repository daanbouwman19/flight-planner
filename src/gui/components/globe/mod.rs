pub mod math;
pub mod providers;
pub mod state;
mod tile_manager;

use eframe::egui::{self, Color32, Painter, Pos2, Shape, Stroke, Vec2};
use std::f32::consts::PI;

use math::{lat_lon_to_vec3, lat_from_y, project, rotate};
use state::GlobeState;
use tile_manager::TileManager;

/// A 3D globe component that visualises a route between two points.
pub struct Globe;

impl Globe {
    /// Renders the 3D globe with a route between two coordinates.
    pub fn render(
        ui: &mut egui::Ui,
        id: egui::Id,
        start_lat_lon: (f64, f64),
        end_lat_lon: (f64, f64),
    ) {
        let available_width = ui.available_width();
        let (rect, response) =
            ui.allocate_exact_size(Vec2::splat(available_width), egui::Sense::drag());

        let p1_vec = lat_lon_to_vec3(start_lat_lon.0 as f32, start_lat_lon.1 as f32);
        let p2_vec = lat_lon_to_vec3(end_lat_lon.0 as f32, end_lat_lon.1 as f32);

        let mut state: GlobeState = ui
            .data(|d| d.get_temp(id))
            .unwrap_or_else(|| Self::initial_state(p1_vec, p2_vec, start_lat_lon, end_lat_lon));

        if state.last_p1 != p1_vec || state.last_p2 != p2_vec {
            state = Self::initial_state(p1_vec, p2_vec, start_lat_lon, end_lat_lon);
        }

        let tile_manager_id = ui.make_persistent_id("tile_manager");
        let tile_manager: TileManager =
            ui.data(|d| d.get_temp(tile_manager_id)).unwrap_or_else(|| {
                let m = TileManager::new(ui.ctx().clone());
                ui.data_mut(|d| d.insert_temp(tile_manager_id, m.clone()));
                m
            });

        Self::handle_interaction(&response, ui, &mut state, id);

        let base_radius = (rect.width() / 2.0) * 0.8;
        let radius = base_radius * state.zoom;
        let center = rect.center();

        // Pre-fetch z=0,1,2 so the 4-level fallback always lands on something.
        tile_manager.trigger_fetch(0, 0, 0);
        for tx in 0..2u32 {
            for ty in 0..2u32 {
                tile_manager.trigger_fetch(1, tx, ty);
            }
        }
        for tx in 0..4u32 {
            for ty in 0..4u32 {
                tile_manager.trigger_fetch(2, tx, ty);
            }
        }

        let tile_z = math::tile_z_for_radius(radius).clamp(2, 8);

        let p1 = lat_lon_to_vec3(start_lat_lon.0 as f32, start_lat_lon.1 as f32);
        let p2 = lat_lon_to_vec3(end_lat_lon.0 as f32, end_lat_lon.1 as f32);

        let painter = ui.painter_at(rect);
        Self::draw_tiles(&painter, center, radius, state, &tile_manager, tile_z);
        Self::draw_route(&painter, center, radius, state, p1, p2);
        Self::draw_point(&painter, center, radius, state, p1, Color32::GREEN, "DEP");
        Self::draw_point(&painter, center, radius, state, p2, Color32::RED, "DEST");
        Self::draw_outline(&painter, center, radius);
        Self::draw_debug_overlay(&painter, rect, tile_z, state, &tile_manager);

        if Self::draw_recenter_button(ui, rect) {
            state = Self::initial_state(p1_vec, p2_vec, start_lat_lon, end_lat_lon);
            ui.data_mut(|d| d.insert_temp(id, state));
        }

        if ui.input(|i| i.time) % 5.0 < 0.1 {
            tile_manager.reset_hit_miss_stats();
        }
    }

    fn handle_interaction(
        response: &egui::Response,
        ui: &egui::Ui,
        state: &mut GlobeState,
        id: egui::Id,
    ) {
        let mut changed = false;

        if response.dragged_by(egui::PointerButton::Primary)
            || response.dragged_by(egui::PointerButton::Secondary)
        {
            let sens = 0.003 / state.zoom.sqrt();
            state.yaw += response.drag_delta().x * sens;
            state.pitch =
                (state.pitch + response.drag_delta().y * sens).clamp(-PI / 2.0, PI / 2.0);
            changed = true;
        }

        let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll_delta != 0.0 {
            state.zoom = (state.zoom * (1.0 + scroll_delta * 0.01)).clamp(1.0, 20.0);
            changed = true;
        }

        if changed {
            ui.data_mut(|d| d.insert_temp(id, *state));
        }
    }

    fn draw_outline(painter: &Painter, center: Pos2, radius: f32) {
        painter.circle_stroke(center, radius, Stroke::new(2.0, Color32::WHITE));
        painter.circle_stroke(
            center,
            radius + 2.0,
            Stroke::new(1.0, Color32::from_rgba_unmultiplied(100, 200, 255, 30)),
        );
    }

    fn draw_debug_overlay(
        painter: &Painter,
        rect: egui::Rect,
        tile_z: u8,
        state: GlobeState,
        manager: &TileManager,
    ) {
        let debug_rect =
            egui::Rect::from_min_size(rect.min + Vec2::new(10.0, 10.0), Vec2::new(120.0, 100.0));
        painter.rect_filled(debug_rect, 4.0, Color32::from_black_alpha(150));
        painter.text(
            debug_rect.min + Vec2::new(5.0, 5.0),
            egui::Align2::LEFT_TOP,
            format!(
                "LOD: {}\nZoom: {:.2}\nHits: {}\nMiss: {}\nErr: {}\nPend: {}\nCache: {}",
                tile_z,
                state.zoom,
                manager.hits(),
                manager.misses(),
                manager.errors(),
                manager.pending_count(),
                manager.cache_size()
            ),
            egui::FontId::monospace(10.0),
            Color32::WHITE,
        );
    }

    fn draw_recenter_button(ui: &mut egui::Ui, rect: egui::Rect) -> bool {
        let button_rect =
            egui::Rect::from_min_size(rect.max - Vec2::new(40.0, 40.0), Vec2::new(30.0, 30.0));
        ui.put(
            button_rect,
            egui::Button::new(crate::gui::icons::ICON_RECENTER)
                .fill(Color32::from_black_alpha(150)),
        )
        .on_hover_text("Recenter on route")
        .clicked()
    }

    fn draw_tiles(
        painter: &Painter,
        center: Pos2,
        radius: f32,
        state: GlobeState,
        manager: &TileManager,
        z: u8,
    ) {
        let num_tiles = 1i32 << z;
        let mut tiles = Vec::new();

        let center_lon = -state.yaw.to_degrees();
        let center_lat = state.pitch.to_degrees();
        let deg_range = 180.0 / state.zoom;

        let lon_start = center_lon - deg_range;
        let lon_end = center_lon + deg_range;
        let lat_start = (center_lat - deg_range).clamp(-85.0, 85.0);
        let lat_end = (center_lat + deg_range).clamp(-85.0, 85.0);

        let tx_start = (((lon_start + 180.0) / 360.0) * num_tiles as f32).floor() as i32;
        let mut tx_end = (((lon_end + 180.0) / 360.0) * num_tiles as f32).ceil() as i32;
        if tx_end - tx_start >= num_tiles {
            tx_end = tx_start + num_tiles - 1;
        }

        let lat_to_y = |lat: f32| {
            let lat_rad = lat.to_radians();
            let y = (1.0 - (lat_rad.tan() + 1.0 / lat_rad.cos()).ln() / PI) / 2.0;
            (y * num_tiles as f32) as i32
        };

        let mut y1 = lat_to_y(lat_start);
        let mut y2 = lat_to_y(lat_end);
        if y1 > y2 {
            std::mem::swap(&mut y1, &mut y2);
        }
        let ty_start = y1.max(0).min(num_tiles - 1);
        let ty_end = y2.max(0).min(num_tiles - 1);

        for ty in ty_start..=ty_end {
            for tx_raw in tx_start..=tx_end {
                let tx = ((tx_raw % num_tiles) + num_tiles) % num_tiles;
                let tx = tx as u32;
                let ty = ty as u32;

                let lon_min = (tx as f32 / num_tiles as f32) * 360.0 - 180.0;
                let lon_max = ((tx + 1) as f32 / num_tiles as f32) * 360.0 - 180.0;
                let lat_max = lat_from_y(ty as f32, num_tiles as u32);
                let lat_min = lat_from_y((ty + 1) as f32, num_tiles as u32);

                let p_mid =
                    lat_lon_to_vec3((lat_min + lat_max) / 2.0, (lon_min + lon_max) / 2.0);
                let rotated_mid = rotate(p_mid, state.yaw, state.pitch);
                if rotated_mid[2] < -0.8 {
                    continue;
                }

                tiles.push((rotated_mid[2], tx, ty, lon_min, lon_max));
            }
        }

        tiles.sort_by(|a, b| a.0.total_cmp(&b.0));

        for (_, tx, ty, lon_min, lon_max) in tiles {
            if let Some((texture, uv_range)) = manager.get_best_tile(z, tx, ty) {
                let mut mesh = egui::Mesh::with_texture(texture.id());
                let substeps = 6;

                for sy in 0..=substeps {
                    for sx in 0..=substeps {
                        let f_x = sx as f32 / substeps as f32;
                        let f_y = sy as f32 / substeps as f32;

                        let lon = lon_min + f_x * (lon_max - lon_min);
                        let lat = lat_from_y(ty as f32 + f_y, num_tiles as u32);
                        let p = lat_lon_to_vec3(lat, lon);
                        let rotated = rotate(p, state.yaw, state.pitch);

                        let alpha = if rotated[2] < 0.0 {
                            (1.0 + rotated[2] * 5.0).clamp(0.0, 1.0)
                        } else {
                            1.0
                        };

                        let u = uv_range[0] + f_x * (uv_range[2] - uv_range[0]);
                        let v = uv_range[1] + f_y * (uv_range[3] - uv_range[1]);

                        mesh.vertices.push(egui::epaint::Vertex {
                            pos: project(rotated, center, radius),
                            uv: Pos2::new(u, v),
                            color: Color32::from_rgba_unmultiplied(
                                255,
                                255,
                                255,
                                (alpha * 255.0) as u8,
                            ),
                        });
                    }
                }

                for sy in 0..substeps {
                    for sx in 0..substeps {
                        let i = sy * (substeps + 1) + sx;
                        mesh.indices.extend_from_slice(&[
                            i as u32,
                            (i + 1) as u32,
                            (i + substeps + 1) as u32,
                            (i + 1) as u32,
                            (i + substeps + 2) as u32,
                            (i + substeps + 1) as u32,
                        ]);
                    }
                }

                painter.add(Shape::mesh(mesh));
            }
        }
    }

    fn draw_route(
        painter: &Painter,
        center: Pos2,
        radius: f32,
        state: GlobeState,
        p1: [f32; 3],
        p2: [f32; 3],
    ) {
        let dot = p1[0] * p2[0] + p1[1] * p2[1] + p1[2] * p2[2];
        let theta = dot.clamp(-1.0, 1.0).acos();
        if theta < 0.001 {
            return;
        }

        let steps = (theta.to_degrees() as usize).clamp(10, 100);
        let mut last_p: Option<Pos2> = None;
        let stroke = Stroke::new(3.0, Color32::from_rgb(255, 200, 0));

        for i in 0..=steps {
            let f = i as f32 / steps as f32;
            let a = ((1.0 - f) * theta).sin() / theta.sin();
            let b = (f * theta).sin() / theta.sin();
            let p = [
                a * p1[0] + b * p2[0],
                a * p1[1] + b * p2[1],
                a * p1[2] + b * p2[2],
            ];
            let rotated = rotate(p, state.yaw, state.pitch);
            if rotated[2] > -0.05 {
                let screen_p = project(rotated, center, radius);
                if let Some(prev) = last_p {
                    painter.line_segment([prev, screen_p], stroke);
                }
                last_p = if rotated[2] > 0.0 { Some(screen_p) } else { None };
            } else {
                last_p = None;
            }
        }
    }

    fn draw_point(
        painter: &Painter,
        center: Pos2,
        radius: f32,
        state: GlobeState,
        p: [f32; 3],
        color: Color32,
        label: &str,
    ) {
        let rotated = rotate(p, state.yaw, state.pitch);
        if rotated[2] > 0.0 {
            let screen_p = project(rotated, center, radius);
            painter.circle_filled(screen_p, 4.0, color);
            painter.circle_stroke(screen_p, 4.0, Stroke::new(1.0, Color32::WHITE));
            painter.text(
                screen_p + Vec2::new(6.0, -6.0),
                egui::Align2::LEFT_BOTTOM,
                label,
                egui::FontId::proportional(12.0),
                Color32::WHITE,
            );
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
        let zoom = if theta < 0.01 {
            4.0
        } else {
            (0.9 / (theta / 2.0).sin()).clamp(1.0, 8.0)
        };
        let avg_lat = (start_lat_lon.0 + end_lat_lon.0) / 2.0;
        let avg_lon = (start_lat_lon.1 + end_lat_lon.1) / 2.0;
        GlobeState {
            yaw: -(avg_lon as f32).to_radians(),
            pitch: (avg_lat as f32).to_radians(),
            zoom,
            last_p1: p1,
            last_p2: p2,
        }
    }
}
