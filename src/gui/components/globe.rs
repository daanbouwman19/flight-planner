use eframe::egui::{self, Color32, Painter, Pos2, Shape, Stroke, TextureHandle, Vec2};
use std::collections::{HashMap, HashSet};
use std::f32::consts::PI;
use std::sync::{Arc, Mutex};

/// A 3D globe component that visualizes a route between two points.
pub struct Globe;

#[derive(Clone, Copy, Debug)]
struct GlobeState {
    yaw: f32,
    pitch: f32,
    zoom: f32,
    last_p1: [f32; 3],
    last_p2: [f32; 3],
}

use std::sync::atomic::{AtomicUsize, Ordering};

struct TileManagerInner {
    cache: Mutex<HashMap<(u8, u32, u32), TextureHandle>>,
    pending: Mutex<HashSet<(u8, u32, u32)>>,
    request_tx: std::sync::mpsc::Sender<(u8, u32, u32)>,

    // Metrics for debugging
    hits: AtomicUsize,
    misses: AtomicUsize,
    errors: AtomicUsize,

    client: reqwest::blocking::Client,
    access_order: Mutex<std::collections::VecDeque<(u8, u32, u32)>>,
}

impl TileManagerInner {
    fn new(ctx: egui::Context) -> Arc<Self> {
        let (tx, rx) = std::sync::mpsc::channel::<(u8, u32, u32)>();
        let rx = Arc::new(Mutex::new(rx));
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap_or_default();

        let manager = Arc::new(Self {
            cache: Mutex::new(HashMap::new()),
            pending: Mutex::new(HashSet::new()),
            access_order: Mutex::new(std::collections::VecDeque::new()),
            request_tx: tx,
            hits: AtomicUsize::new(0),
            misses: AtomicUsize::new(0),
            errors: AtomicUsize::new(0),
            client,
        });

        for i in 0..8 {
            let manager_clone = Arc::downgrade(&manager);
            let rx_clone = rx.clone();
            let ctx_clone = ctx.clone();
            std::thread::spawn(move || {
                loop {
                    let Ok((z, x, y)) = rx_clone.lock().unwrap().recv() else {
                        break;
                    };
                    let Some(manager) = manager_clone.upgrade() else {
                        break;
                    };

                    let url = format!(
                        "https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{}/{}/{}",
                        z, y, x
                    );

                    let result = manager
                        .client
                        .get(&url)
                        .send()
                        .and_then(|r| r.bytes())
                        .map_err(|e| {
                            log::error!(
                                "[Worker {i}] Network error fetching tile {z}/{y}/{x}: {e}"
                            );
                            Box::new(e) as Box<dyn std::error::Error + Send + Sync>
                        })
                        .and_then(|b| {
                            image::load_from_memory(&b).map_err(|e| {
                                log::error!("[Worker {i}] Decode error for tile {z}/{y}/{x}: {e}");
                                Box::new(e) as Box<dyn std::error::Error + Send + Sync>
                            })
                        });

                    match result {
                        Ok(image) => {
                            let size = [image.width() as usize, image.height() as usize];
                            let pixels = image.to_rgba8();
                            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                                size,
                                pixels.as_flat_samples().as_slice(),
                            );
                            let tex = ctx_clone.load_texture(
                                format!("tile_{}_{}_{}", z, x, y),
                                color_image,
                                Default::default(),
                            );
                            let mut cache = manager.cache.lock().unwrap();
                            let mut order = manager.access_order.lock().unwrap();

                            // Evict oldest if full (max 1000 tiles)
                            while cache.len() >= 1000 {
                                if let Some(oldest_key) = order.pop_front() {
                                    cache.remove(&oldest_key);
                                } else {
                                    break;
                                }
                            }

                            cache.insert((z, x, y), tex);
                            order.push_back((z, x, y));

                            ctx_clone.request_repaint();
                        }
                        Err(_) => {
                            manager.errors.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                    manager.pending.lock().unwrap().remove(&(z, x, y));
                }
            });
        }

        manager
    }
}

#[derive(Clone)]
struct SharedTileManager(Arc<TileManagerInner>);

impl SharedTileManager {
    fn trigger_fetch(&self, z: u8, x: u32, y: u32) {
        let key = (z, x, y);
        if self.0.cache.lock().unwrap().contains_key(&key) {
            return;
        }

        let mut pending = self.0.pending.lock().unwrap();
        if !pending.contains(&key) && pending.len() < 512 {
            pending.insert(key);
            let _ = self.0.request_tx.send(key);
        }
    }

    fn get_best_tile(&self, z: u8, x: u32, y: u32) -> Option<(TextureHandle, [f32; 4])> {
        self.trigger_fetch(z, x, y);

        let mut cur_z = z;
        let mut cur_x = x;
        let mut cur_y = y;

        loop {
            if let Some(tex) = self.0.cache.lock().unwrap().get(&(cur_z, cur_x, cur_y)) {
                if cur_z == z {
                    self.0.hits.fetch_add(1, Ordering::Relaxed);
                } else {
                    self.0.misses.fetch_add(1, Ordering::Relaxed);
                }

                // Update access order for LRU
                let mut order = self.0.access_order.lock().unwrap();
                if let Some(pos) = order.iter().position(|&k| k == (cur_z, cur_x, cur_y)) {
                    let key = order.remove(pos).unwrap();
                    order.push_back(key);
                }

                let z_diff = z - cur_z;
                let pow_diff = (1 << z_diff) as f32;
                let dx = (x % (1 << z_diff)) as f32;
                let dy = (y % (1 << z_diff)) as f32;

                // UV range [u_min, v_min, u_max, v_max]
                return Some((
                    tex.clone(),
                    [
                        dx / pow_diff,
                        dy / pow_diff,
                        (dx + 1.0) / pow_diff,
                        (dy + 1.0) / pow_diff,
                    ],
                ));
            }

            if cur_z == 0 {
                break;
            }
            cur_z -= 1;
            cur_x /= 2;
            cur_y /= 2;
        }

        None
    }
}

impl Default for GlobeState {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            zoom: 1.0,
            last_p1: [0.0; 3],
            last_p2: [0.0; 3],
        }
    }
}

impl Globe {
    /// Renders the 3D globe with a route between two coordinates.
    ///
    /// # Arguments
    ///
    /// * `ui` - The egui::Ui to render into.
    /// * `start_lat_lon` - (latitude, longitude) of the departure point in degrees.
    /// * `end_lat_lon` - (latitude, longitude) of the destination point in degrees.
    pub fn render(ui: &mut egui::Ui, start_lat_lon: (f64, f64), end_lat_lon: (f64, f64)) {
        let available_width = ui.available_width();
        let size = Vec2::splat(available_width);
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::drag());

        let id = ui.make_persistent_id("globe_state");
        let p1_vec = Self::lat_lon_to_vec3(start_lat_lon.0 as f32, start_lat_lon.1 as f32);
        let p2_vec = Self::lat_lon_to_vec3(end_lat_lon.0 as f32, end_lat_lon.1 as f32);

        let mut state: GlobeState = ui
            .data(|d| d.get_temp(id))
            .unwrap_or_else(|| Self::initial_state(p1_vec, p2_vec, start_lat_lon, end_lat_lon));

        // Detect route change and reset view
        if state.last_p1 != p1_vec || state.last_p2 != p2_vec {
            state = Self::initial_state(p1_vec, p2_vec, start_lat_lon, end_lat_lon);
        }

        // Shared tile manager
        let tile_manager_id = ui.make_persistent_id("tile_manager");
        let tile_manager: SharedTileManager =
            ui.data(|d| d.get_temp(tile_manager_id)).unwrap_or_else(|| {
                let manager = SharedTileManager(TileManagerInner::new(ui.ctx().clone()));
                ui.data_mut(|d| d.insert_temp(tile_manager_id, manager.clone()));
                manager
            });

        // Handle interaction
        if response.dragged() {
            state.yaw += response.drag_delta().x * 0.003;
            // Inverted Y axis fix: use + for pitch adjustment
            state.pitch =
                (state.pitch + response.drag_delta().y * 0.003).clamp(-PI / 2.0, PI / 2.0);
            ui.data_mut(|d| d.insert_temp(id, state));
        }

        let scroll_delta = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll_delta != 0.0 {
            state.zoom = (state.zoom * (1.0 + scroll_delta * 0.008)).clamp(0.5, 15.0);
            ui.data_mut(|d| d.insert_temp(id, state));
        }

        let painter = ui.painter_at(rect);
        let center = rect.center();
        let radius = (rect.width() / 2.0) * 0.8 * state.zoom;

        // Pre-fetch base levels if not present
        tile_manager.trigger_fetch(0, 0, 0);
        tile_manager.trigger_fetch(1, 0, 0);
        tile_manager.trigger_fetch(1, 0, 1);
        tile_manager.trigger_fetch(1, 1, 0);
        tile_manager.trigger_fetch(1, 1, 1);

        // Determine zoom level for tiles
        let tile_z = if state.zoom > 8.0 {
            6
        } else if state.zoom > 4.0 {
            5
        } else if state.zoom > 2.0 {
            4
        } else {
            3
        };

        // Draw satellite tiles
        Self::draw_tiles(&painter, center, radius, state, &tile_manager, tile_z);

        // Convert lat/lon to unit vectors
        let p1 = Self::lat_lon_to_vec3(start_lat_lon.0 as f32, start_lat_lon.1 as f32);
        let p2 = Self::lat_lon_to_vec3(end_lat_lon.0 as f32, end_lat_lon.1 as f32);

        // Draw route line (great circle)
        Self::draw_route(&painter, center, radius, state, p1, p2);

        // Draw points
        Self::draw_point(&painter, center, radius, state, p1, Color32::GREEN, "DEP");
        Self::draw_point(&painter, center, radius, state, p2, Color32::RED, "DEST");

        // Draw globe outline
        painter.circle_stroke(center, radius, Stroke::new(2.0, Color32::WHITE));

        // Add a subtle atmosphere glow
        painter.circle_stroke(
            center,
            radius + 2.0,
            Stroke::new(1.0, Color32::from_rgba_unmultiplied(100, 200, 255, 30)),
        );

        // --- DEBUG OVERLAY ---
        let debug_rect =
            egui::Rect::from_min_size(rect.min + Vec2::new(10.0, 10.0), Vec2::new(120.0, 100.0));
        painter.rect_filled(debug_rect, 4.0, Color32::from_black_alpha(150));

        let hits = tile_manager.0.hits.load(Ordering::Relaxed);
        let misses = tile_manager.0.misses.load(Ordering::Relaxed);
        let errors = tile_manager.0.errors.load(Ordering::Relaxed);
        let pending = tile_manager.0.pending.lock().unwrap().len();
        let cache_size = tile_manager.0.cache.lock().unwrap().len();

        let debug_text = format!(
            "LOD: {}\nZoom: {:.2}\nHits: {}\nMiss: {}\nErr: {}\nPend: {}\nCache: {}",
            tile_z, state.zoom, hits, misses, errors, pending, cache_size
        );

        painter.text(
            debug_rect.min + Vec2::new(5.0, 5.0),
            egui::Align2::LEFT_TOP,
            debug_text,
            egui::FontId::monospace(10.0),
            Color32::WHITE,
        );

        // Reset counters periodically to see "real-time" traffic
        if ui.input(|i| i.time) % 5.0 < 0.1 {
            tile_manager.0.hits.store(0, Ordering::Relaxed);
            tile_manager.0.misses.store(0, Ordering::Relaxed);
        }
    }

    fn lat_lon_to_vec3(lat: f32, lon: f32) -> [f32; 3] {
        let lat_rad = lat.to_radians();
        let lon_rad = lon.to_radians();
        [
            lat_rad.cos() * lon_rad.sin(),
            lat_rad.sin(),
            lat_rad.cos() * lon_rad.cos(),
        ]
    }

    fn rotate(p: [f32; 3], state: GlobeState) -> [f32; 3] {
        // Rotate around Y axis (yaw)
        let x1 = p[0] * state.yaw.cos() + p[2] * state.yaw.sin();
        let y1 = p[1];
        let z1 = -p[0] * state.yaw.sin() + p[2] * state.yaw.cos();

        // Rotate around X axis (pitch)
        let x2 = x1;
        let y2 = y1 * state.pitch.cos() - z1 * state.pitch.sin();
        let z2 = y1 * state.pitch.sin() + z1 * state.pitch.cos();

        [x2, y2, z2]
    }

    fn project(p: [f32; 3], center: Pos2, radius: f32) -> Pos2 {
        Pos2::new(center.x + p[0] * radius, center.y - p[1] * radius)
    }

    fn draw_tiles(
        painter: &Painter,
        center: Pos2,
        radius: f32,
        state: GlobeState,
        manager: &SharedTileManager,
        z: u8,
    ) {
        let num_tiles = 1 << z;
        let mut tiles = Vec::new();

        // Viewport optimization: Only iterate over tiles likely to be visible
        let center_lon = -state.yaw.to_degrees();
        let center_lat = state.pitch.to_degrees();

        let deg_range = 180.0 / state.zoom;

        let lon_start = center_lon - deg_range;
        let lon_end = center_lon + deg_range;
        let lat_start = (center_lat - deg_range).clamp(-85.0, 85.0);
        let lat_end = (center_lat + deg_range).clamp(-85.0, 85.0);

        let tx_start = (((lon_start + 180.0) / 360.0) * num_tiles as f32).floor() as i32;
        let mut tx_end = (((lon_end + 180.0) / 360.0) * num_tiles as f32).ceil() as i32;

        // Cap the range to avoid redundant iterations at low zoom
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
                let lat_from_y = |y: f32| {
                    let n = PI - 2.0 * PI * y / num_tiles as f32;
                    (180.0 / PI) * n.sinh().atan()
                };
                let lat_max = lat_from_y(ty as f32);
                let lat_min = lat_from_y((ty + 1) as f32);

                let p_mid =
                    Self::lat_lon_to_vec3((lat_min + lat_max) / 2.0, (lon_min + lon_max) / 2.0);
                let rotated_mid = Self::rotate(p_mid, state);
                if rotated_mid[2] < -0.8 {
                    continue;
                }

                tiles.push((rotated_mid[2], tx, ty, lon_min, lon_max, lat_min, lat_max));
            }
        }

        // Sort by Z (back to front) - total_cmp is safer for floats
        tiles.sort_by(|a, b| a.0.total_cmp(&b.0));

        for (_, tx, ty, lon_min, lon_max, _lat_min, _lat_max) in tiles {
            let lat_from_y = |y: f32| {
                let n = PI - 2.0 * PI * y / num_tiles as f32;
                (180.0 / PI) * n.sinh().atan()
            };

            if let Some((texture, uv_range)) = manager.get_best_tile(z, tx, ty) {
                let mut mesh = egui::Mesh::with_texture(texture.id());
                let substeps = 6;
                for sy in 0..=substeps {
                    for sx in 0..=substeps {
                        let f_x = sx as f32 / substeps as f32;
                        let f_y = sy as f32 / substeps as f32;

                        let lon = lon_min + f_x * (lon_max - lon_min);
                        let lat = lat_from_y(ty as f32 + f_y);

                        let p = Self::lat_lon_to_vec3(lat, lon);
                        let rotated = Self::rotate(p, state);

                        // Per-vertex horizon culling for smoothness
                        let alpha = if rotated[2] < 0.0 {
                            (1.0 + rotated[2] * 5.0).clamp(0.0, 1.0)
                        } else {
                            1.0
                        };

                        let screen_p = Self::project(rotated, center, radius);

                        // Map local f_x, f_y to global uv_range
                        let u = uv_range[0] + f_x * (uv_range[2] - uv_range[0]);
                        let v = uv_range[1] + f_y * (uv_range[3] - uv_range[1]);

                        mesh.vertices.push(egui::epaint::Vertex {
                            pos: screen_p,
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
                            (i + (substeps + 1)) as u32,
                            (i + 1) as u32,
                            (i + (substeps + 2)) as u32,
                            (i + (substeps + 1)) as u32,
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

            let rotated = Self::rotate(p, state);

            // Horizon clipping for smooth lines
            if rotated[2] > -0.05 {
                let screen_p = Self::project(rotated, center, radius);
                if let Some(prev) = last_p {
                    // Only draw if we didn't just cross from far-back to near-front
                    painter.line_segment([prev, screen_p], stroke);
                }
                last_p = if rotated[2] > 0.0 {
                    Some(screen_p)
                } else {
                    None
                };
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
        let rotated = Self::rotate(p, state);
        if rotated[2] > 0.0 {
            let screen_p = Self::project(rotated, center, radius);
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
