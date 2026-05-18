use eframe::egui::{self, TextureHandle};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
#[cfg(target_arch = "wasm32")]
use std::sync::Weak;

type TileKey = (u8, u32, u32);
type TileCache = HashMap<TileKey, (TextureHandle, usize)>;

struct TileManagerInner {
    cache: Mutex<TileCache>,
    pending: Mutex<HashSet<TileKey>>,
    #[cfg(target_arch = "wasm32")]
    ctx: egui::Context,
    hits: AtomicUsize,
    misses: AtomicUsize,
    errors: AtomicUsize,
    access_counter: AtomicUsize,
    #[cfg(not(target_arch = "wasm32"))]
    request_tx: std::sync::mpsc::Sender<TileKey>,
    #[cfg(not(target_arch = "wasm32"))]
    provider: Arc<dyn super::providers::TileProvider>,
}

impl TileManagerInner {
    fn new(ctx: egui::Context) -> Arc<Self> {
        Self::new_impl(ctx)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn new_impl(ctx: egui::Context) -> Arc<Self> {
        let (tx, rx) = std::sync::mpsc::channel::<TileKey>();
        let rx = Arc::new(Mutex::new(rx));
        let http_client = Arc::new(crate::modules::http::ReqwestClient::new());
        let provider: Arc<dyn super::providers::TileProvider> =
            Arc::new(super::providers::ArcGisTileProvider::new(http_client));

        let manager = Arc::new(Self {
            cache: Mutex::new(HashMap::new()),
            pending: Mutex::new(HashSet::new()),
            hits: AtomicUsize::new(0),
            misses: AtomicUsize::new(0),
            errors: AtomicUsize::new(0),
            access_counter: AtomicUsize::new(0),
            request_tx: tx,
            provider,
        });

        for i in 0..8 {
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
    fn new_impl(ctx: egui::Context) -> Arc<Self> {
        Arc::new(Self {
            cache: Mutex::new(HashMap::new()),
            pending: Mutex::new(HashSet::new()),
            ctx,
            hits: AtomicUsize::new(0),
            misses: AtomicUsize::new(0),
            errors: AtomicUsize::new(0),
            access_counter: AtomicUsize::new(0),
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
        if cache.len() >= 512 {
            let oldest_key = cache
                .iter()
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

#[cfg(target_arch = "wasm32")]
async fn fetch_tile_wasm(manager_weak: Weak<TileManagerInner>, ctx: egui::Context, z: u8, x: u32, y: u32) {
    let url = super::providers::tile_url(z, x, y);

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

/// Shared, clone-able handle to the tile cache and fetch workers.
#[derive(Clone)]
pub struct TileManager(Arc<TileManagerInner>);

impl TileManager {
    pub fn new(ctx: egui::Context) -> Self {
        Self(TileManagerInner::new(ctx))
    }

    pub fn trigger_fetch(&self, z: u8, x: u32, y: u32) {
        let key = (z, x, y);
        if self.0.cache.lock().unwrap().contains_key(&key) {
            return;
        }

        let mut pending = self.0.pending.lock().unwrap();
        if pending.contains(&key) || pending.len() >= 512 {
            return;
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

    /// Returns the best available tile at or below `z`, falling back at most 4
    /// zoom levels. Prevents showing single-pixel slivers of very low-res tiles.
    pub fn get_best_tile(&self, z: u8, x: u32, y: u32) -> Option<(TextureHandle, [f32; 4])> {
        self.trigger_fetch(z, x, y);

        let min_z = z.saturating_sub(4);
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
                let pow_diff = (1 << z_diff) as f32;
                let dx = (x % (1 << z_diff)) as f32;
                let dy = (y % (1 << z_diff)) as f32;

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

            if cur_z == 0 || cur_z <= min_z {
                break;
            }
            cur_z -= 1;
            cur_x /= 2;
            cur_y /= 2;
        }

        None
    }

    pub fn hits(&self) -> usize {
        self.0.hits.load(Ordering::Relaxed)
    }

    pub fn misses(&self) -> usize {
        self.0.misses.load(Ordering::Relaxed)
    }

    pub fn errors(&self) -> usize {
        self.0.errors.load(Ordering::Relaxed)
    }

    pub fn pending_count(&self) -> usize {
        self.0.pending.lock().unwrap().len()
    }

    pub fn cache_size(&self) -> usize {
        self.0.cache.lock().unwrap().len()
    }

    pub fn reset_hit_miss_stats(&self) {
        self.0.hits.store(0, Ordering::Relaxed);
        self.0.misses.store(0, Ordering::Relaxed);
    }
}
