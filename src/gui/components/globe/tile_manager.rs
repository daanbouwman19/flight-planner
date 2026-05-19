use eframe::egui::{self, TextureHandle};
use std::collections::{HashMap, HashSet};
#[cfg(target_arch = "wasm32")]
use std::sync::Weak;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use super::providers;

pub const TILE_CACHE_CAP: usize = 512;
pub const PENDING_CAP: usize = 512;
#[cfg(not(target_arch = "wasm32"))]
pub const NUM_WORKERS: usize = 8;
const STATS_DECAY_SECS: f64 = 5.0;

pub type TileKey = (u8, u32, u32);
pub type TileCache = HashMap<TileKey, (TextureHandle, usize)>;

#[derive(Clone, Copy, Debug, Default)]
pub struct TileStats {
    pub hits: usize,
    pub misses: usize,
    pub errors: usize,
    pub pending: usize,
    pub cache_size: usize,
}

pub struct TileManagerInner {
    cache: Mutex<TileCache>,
    pending: Mutex<HashSet<TileKey>>,
    #[cfg(target_arch = "wasm32")]
    ctx: egui::Context,
    hits: AtomicUsize,
    misses: AtomicUsize,
    errors: AtomicUsize,
    access_counter: AtomicUsize,
    last_decay: Mutex<f64>,
    #[cfg(not(target_arch = "wasm32"))]
    request_tx: std::sync::mpsc::Sender<TileKey>,
    #[cfg(not(target_arch = "wasm32"))]
    provider: Arc<dyn providers::TileProvider>,
}

impl TileManagerInner {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(ctx: egui::Context) -> Arc<Self> {
        let (tx, rx) = std::sync::mpsc::channel::<TileKey>();
        let rx = Arc::new(Mutex::new(rx));
        let http_client = Arc::new(crate::modules::http::ReqwestClient::new());
        let provider: Arc<dyn providers::TileProvider> =
            Arc::new(providers::ArcGisTileProvider::new(http_client));

        let manager = Arc::new(Self {
            cache: Mutex::new(HashMap::new()),
            pending: Mutex::new(HashSet::new()),
            hits: AtomicUsize::new(0),
            misses: AtomicUsize::new(0),
            errors: AtomicUsize::new(0),
            access_counter: AtomicUsize::new(0),
            last_decay: Mutex::new(0.0),
            request_tx: tx,
            provider,
        });

        for i in 0..NUM_WORKERS {
            let manager_weak = Arc::downgrade(&manager);
            let rx_clone = Arc::clone(&rx);
            let ctx_clone = ctx.clone();
            std::thread::spawn(move || {
                loop {
                    let Ok((z, x, y)) = rx_clone.lock().unwrap().recv() else {
                        break;
                    };
                    let Some(manager) = manager_weak.upgrade() else {
                        break;
                    };

                    // Skip tiles evicted from pending while sitting in the channel.
                    if !manager.pending.lock().unwrap().contains(&(z, x, y)) {
                        continue;
                    }

                    let result = manager
                        .provider
                        .fetch_tile(z, x, y)
                        .map_err(|e| {
                            log::error!(
                                "[Worker {i}] Network error fetching tile {z}/{y}/{x}: {e}"
                            );
                            e
                        })
                        .and_then(|b| {
                            image::load_from_memory(&b).map_err(|e| {
                                log::error!("[Worker {i}] Decode error for tile {z}/{y}/{x}: {e}");
                                e.to_string()
                            })
                        });

                    match result {
                        Ok(img) => Self::insert_texture(&manager, &ctx_clone, img, z, x, y),
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

    #[cfg(target_arch = "wasm32")]
    pub fn new(ctx: egui::Context) -> Arc<Self> {
        Arc::new(Self {
            cache: Mutex::new(HashMap::new()),
            pending: Mutex::new(HashSet::new()),
            ctx,
            hits: AtomicUsize::new(0),
            misses: AtomicUsize::new(0),
            errors: AtomicUsize::new(0),
            access_counter: AtomicUsize::new(0),
            last_decay: Mutex::new(0.0),
        })
    }

    fn insert_texture(
        manager: &Arc<Self>,
        ctx: &egui::Context,
        img: image::DynamicImage,
        z: u8,
        x: u32,
        y: u32,
    ) {
        let size = [img.width() as usize, img.height() as usize];
        let pixels = img.to_rgba8();
        let color_image =
            egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_flat_samples().as_slice());
        let tex = ctx.load_texture(format!("tile_{z}_{x}_{y}"), color_image, Default::default());
        let mut cache = manager.cache.lock().unwrap();
        if cache.len() >= TILE_CACHE_CAP {
            // Never evict base LODs — they serve as the always-available fallback.
            let oldest_key = cache
                .iter()
                .filter(|((lod, _, _), _)| *lod >= 3)
                .min_by_key(|(_, (_, access))| *access)
                .map(|(&k, _)| k);
            if let Some(key) = oldest_key {
                cache.remove(&key);
            }
        }
        let access = manager.access_counter.fetch_add(1, Ordering::Relaxed);
        cache.insert((z, x, y), (tex, access));
        ctx.request_repaint();
    }
}

#[derive(Clone)]
pub struct SharedTileManager(pub Arc<TileManagerInner>);

impl SharedTileManager {
    pub fn new(ctx: egui::Context) -> Self {
        Self(TileManagerInner::new(ctx))
    }

    pub fn trigger_fetch(&self, z: u8, x: u32, y: u32) {
        let key = (z, x, y);
        if self.0.cache.lock().unwrap().contains_key(&key) {
            return;
        }

        let mut pending = self.0.pending.lock().unwrap();
        if pending.contains(&key) {
            return;
        }
        // Queue full: stale requests from a previous camera position are blocking
        // new visible tiles. Clear them so current tiles can be fetched immediately.
        if pending.len() >= PENDING_CAP {
            pending.clear();
        }
        pending.insert(key);
        drop(pending);

        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = self.0.request_tx.send(key);
        }

        #[cfg(target_arch = "wasm32")]
        {
            let weak = Arc::downgrade(&self.0);
            let ctx = self.0.ctx.clone();
            wasm_bindgen_futures::spawn_local(async move {
                fetch_tile_wasm(weak, ctx, z, x, y).await;
            });
        }
    }

    /// Returns the best available texture for `(z, x, y)` — either the exact tile
    /// or the deepest cached ancestor along with the UV sub-rect to sample from it.
    pub fn get_best_tile(&self, z: u8, x: u32, y: u32) -> Option<(TextureHandle, [f32; 4])> {
        self.trigger_fetch(z, x, y);

        let mut cur_z = z;
        let mut cur_x = x;
        let mut cur_y = y;

        loop {
            let mut cache = self.0.cache.lock().unwrap();
            if let Some((tex, access)) = cache.get_mut(&(cur_z, cur_x, cur_y)) {
                if cur_z == z {
                    self.0.hits.fetch_add(1, Ordering::Relaxed);
                } else {
                    self.0.misses.fetch_add(1, Ordering::Relaxed);
                }

                *access = self.0.access_counter.fetch_add(1, Ordering::Relaxed);

                let z_diff = z - cur_z;
                let pow_diff = (1u32 << z_diff) as f32;
                let dx = (x % (1u32 << z_diff)) as f32;
                let dy = (y % (1u32 << z_diff)) as f32;

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

    pub fn stats(&self) -> TileStats {
        TileStats {
            hits: self.0.hits.load(Ordering::Relaxed),
            misses: self.0.misses.load(Ordering::Relaxed),
            errors: self.0.errors.load(Ordering::Relaxed),
            pending: self.0.pending.lock().unwrap().len(),
            cache_size: self.0.cache.lock().unwrap().len(),
        }
    }

    /// Reset hit/miss counters at most once every `STATS_DECAY_SECS`. Replaces the
    /// `if time % 5.0 < 0.1` hack — the modulo only fired on frames that happened to
    /// land in a 0.1 s window, so during sustained loads the counters could grow
    /// monotonically.
    pub fn decay_stats(&self, time_secs: f64) {
        let mut last = self.0.last_decay.lock().unwrap();
        if time_secs - *last >= STATS_DECAY_SECS {
            self.0.hits.store(0, Ordering::Relaxed);
            self.0.misses.store(0, Ordering::Relaxed);
            *last = time_secs;
        }
    }
}

#[cfg(target_arch = "wasm32")]
async fn fetch_tile_wasm(
    manager_weak: Weak<TileManagerInner>,
    ctx: egui::Context,
    z: u8,
    x: u32,
    y: u32,
) {
    let url = providers::tile_url(z, x, y);

    let result: Result<image::DynamicImage, String> = async {
        let resp = reqwest::get(&url).await.map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("HTTP {}", resp.status()));
        }
        let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
        image::load_from_memory(&bytes).map_err(|e| {
            log::error!("Decode error for tile {z}/{y}/{x}: {e}");
            e.to_string()
        })
    }
    .await;

    let Some(manager) = manager_weak.upgrade() else {
        return;
    };

    match result {
        Ok(img) => TileManagerInner::insert_texture(&manager, &ctx, img, z, x, y),
        Err(_) => {
            manager.errors.fetch_add(1, Ordering::Relaxed);
        }
    }
    manager.pending.lock().unwrap().remove(&(z, x, y));
}
