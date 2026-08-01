use eframe::egui::{self, TextureHandle};
use lru::LruCache;
use std::collections::{HashMap, HashSet};
use std::num::NonZeroUsize;
#[cfg(target_arch = "wasm32")]
use std::sync::Weak;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
#[cfg(not(target_arch = "wasm32"))]
use std::sync::{Condvar, Weak};

use super::providers;

/// LOD >= BASE_LOD tiles are held in an LRU cache of this capacity.
pub const TILE_CACHE_CAP: usize = 512;
pub const PENDING_CAP: usize = 512;
#[cfg(not(target_arch = "wasm32"))]
pub const NUM_WORKERS: usize = 8;
const STATS_DECAY_SECS: f64 = 5.0;
/// Tiles below this LOD are pinned permanently so a coarse fallback always
/// exists; everything else is subject to LRU eviction.
const BASE_LOD: u8 = 3;

pub type TileKey = (u8, u32, u32);

#[derive(Clone, Copy, Debug, Default)]
pub struct TileStats {
    pub hits: usize,
    pub misses: usize,
    pub errors: usize,
    pub pending: usize,
    pub cache_size: usize,
}

/// Work queue shared between `request()` and the native fetch workers.
/// LIFO: the newest requests are served first, so tiles for the current camera
/// position load before stale requests from earlier pan/zoom positions.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
struct FetchQueue {
    state: Mutex<FetchQueueState>,
    ready: Condvar,
}

#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
struct FetchQueueState {
    stack: Vec<TileKey>,
    shutdown: bool,
}

pub struct TileManagerInner {
    /// LRU cache for LOD >= BASE_LOD tiles. Evicts least-recently-used when full.
    tile_cache: Mutex<LruCache<TileKey, TextureHandle>>,
    /// Base LODs are held here permanently and never evicted.
    base_cache: Mutex<HashMap<TileKey, TextureHandle>>,
    /// Keys that have been queued or are currently being fetched.
    pending: Mutex<HashSet<TileKey>>,
    ctx: egui::Context,
    hits: AtomicUsize,
    misses: AtomicUsize,
    errors: AtomicUsize,
    last_decay: Mutex<f64>,
    #[cfg(not(target_arch = "wasm32"))]
    queue: Arc<FetchQueue>,
    #[cfg(not(target_arch = "wasm32"))]
    provider: Arc<dyn providers::TileProvider>,
}

#[cfg(not(target_arch = "wasm32"))]
impl Drop for TileManagerInner {
    fn drop(&mut self) {
        // Wake sleeping workers so their weak upgrade fails and they exit.
        self.queue.state.lock().unwrap().shutdown = true;
        self.queue.ready.notify_all();
    }
}

impl TileManagerInner {
    #[cfg(not(target_arch = "wasm32"))]
    fn new(ctx: egui::Context) -> Arc<Self> {
        let http_client = Arc::new(crate::modules::http::ReqwestClient::new());
        Self::with_provider(
            ctx,
            Arc::new(providers::ArcGisTileProvider::new(http_client)),
        )
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn with_provider(ctx: egui::Context, provider: Arc<dyn providers::TileProvider>) -> Arc<Self> {
        let queue = Arc::new(FetchQueue::default());
        let manager = Arc::new(Self {
            tile_cache: Mutex::new(LruCache::new(NonZeroUsize::new(TILE_CACHE_CAP).unwrap())),
            base_cache: Mutex::new(HashMap::new()),
            pending: Mutex::new(HashSet::new()),
            ctx,
            hits: AtomicUsize::new(0),
            misses: AtomicUsize::new(0),
            errors: AtomicUsize::new(0),
            last_decay: Mutex::new(0.0),
            queue: Arc::clone(&queue),
            provider,
        });

        for worker_idx in 0..NUM_WORKERS {
            let manager_weak = Arc::downgrade(&manager);
            let queue = Arc::clone(&queue);
            std::thread::spawn(move || worker_loop(manager_weak, queue, worker_idx));
        }

        manager
    }

    #[cfg(target_arch = "wasm32")]
    fn new(ctx: egui::Context) -> Arc<Self> {
        Arc::new(Self {
            tile_cache: Mutex::new(LruCache::new(NonZeroUsize::new(TILE_CACHE_CAP).unwrap())),
            base_cache: Mutex::new(HashMap::new()),
            pending: Mutex::new(HashSet::new()),
            ctx,
            hits: AtomicUsize::new(0),
            misses: AtomicUsize::new(0),
            errors: AtomicUsize::new(0),
            last_decay: Mutex::new(0.0),
        })
    }

    fn insert_texture(&self, img: image::DynamicImage, z: u8, x: u32, y: u32) {
        let size = [img.width() as usize, img.height() as usize];
        let pixels = img.to_rgba8();
        let color_image =
            egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_flat_samples().as_slice());
        let tex =
            self.ctx
                .load_texture(format!("tile_{z}_{x}_{y}"), color_image, Default::default());

        if z < BASE_LOD {
            self.base_cache.lock().unwrap().insert((z, x, y), tex);
        } else {
            // LruCache::put auto-evicts the least-recently-used entry when full.
            self.tile_cache.lock().unwrap().put((z, x, y), tex);
        }

        self.ctx.request_repaint();
    }

    fn is_cached(&self, key: TileKey) -> bool {
        // Peek without updating LRU order.
        if key.0 < BASE_LOD {
            self.base_cache.lock().unwrap().contains_key(&key)
        } else {
            self.tile_cache.lock().unwrap().contains(&key)
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn worker_loop(manager_weak: Weak<TileManagerInner>, queue: Arc<FetchQueue>, worker_idx: usize) {
    loop {
        let key = {
            let mut state = queue.state.lock().unwrap();
            loop {
                if state.shutdown {
                    return;
                }
                if let Some(key) = state.stack.pop() {
                    break key;
                }
                state = queue.ready.wait(state).unwrap();
            }
        };
        let Some(manager) = manager_weak.upgrade() else {
            return;
        };

        // Skip requests superseded while sitting in the queue.
        if !manager.pending.lock().unwrap().contains(&key) {
            continue;
        }

        let (z, x, y) = key;
        let result = manager
            .provider
            .fetch_tile(z, x, y)
            .map_err(|e| {
                log::error!("[tile worker {worker_idx}] fetch error for {z}/{y}/{x}: {e}");
                e
            })
            .and_then(|bytes| {
                image::load_from_memory(&bytes).map_err(|e| {
                    log::error!("[tile worker {worker_idx}] decode error for {z}/{y}/{x}: {e}");
                    e.to_string()
                })
            });

        match result {
            Ok(img) => manager.insert_texture(img, z, x, y),
            Err(_) => {
                manager.errors.fetch_add(1, Ordering::Relaxed);
                // Repaint so any loading indicator reflects the settled state.
                manager.ctx.request_repaint();
            }
        }
        manager.pending.lock().unwrap().remove(&key);
    }
}

#[derive(Clone)]
pub struct SharedTileManager(Arc<TileManagerInner>);

impl SharedTileManager {
    pub fn new(ctx: egui::Context) -> Self {
        let manager = Self(TileManagerInner::new(ctx));
        // Warm the permanent base layers once so a coarse fallback exists for
        // any camera position from the first frames on.
        for z in 0..BASE_LOD {
            let n = 1u32 << z;
            for x in 0..n {
                for y in 0..n {
                    manager.request(z, x, y);
                }
            }
        }
        manager
    }

    /// Test constructor with a custom provider and no base prefetch.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn with_provider(ctx: egui::Context, provider: Arc<dyn providers::TileProvider>) -> Self {
        Self(TileManagerInner::with_provider(ctx, provider))
    }

    /// Queue `(z, x, y)` for fetching unless it is already cached or queued.
    pub fn request(&self, z: u8, x: u32, y: u32) {
        let key = (z, x, y);
        if self.0.is_cached(key) {
            return;
        }

        {
            let mut pending = self.0.pending.lock().unwrap();
            if pending.contains(&key) {
                return;
            }
            // Queue full: stale requests from earlier camera positions are
            // blocking new visible tiles. Drop them all; anything still needed
            // is re-requested by the next frame's traversal.
            if pending.len() >= PENDING_CAP {
                pending.clear();
                #[cfg(not(target_arch = "wasm32"))]
                self.0.queue.state.lock().unwrap().stack.clear();
            }
            pending.insert(key);
            #[cfg(not(target_arch = "wasm32"))]
            self.0.queue.state.lock().unwrap().stack.push(key);
        }

        #[cfg(not(target_arch = "wasm32"))]
        self.0.queue.ready.notify_one();

        #[cfg(target_arch = "wasm32")]
        {
            let weak = Arc::downgrade(&self.0);
            wasm_bindgen_futures::spawn_local(async move {
                fetch_tile_wasm(weak, z, x, y).await;
            });
        }
    }

    /// Returns the best available texture for `(z, x, y)` — either the exact
    /// tile or the deepest cached ancestor along with the UV sub-rect to
    /// sample from it. Read-only: fetching is driven by `request`.
    pub fn best_texture(&self, z: u8, x: u32, y: u32) -> Option<(TextureHandle, [f32; 4])> {
        let mut cur_z = z;
        let mut cur_x = x;
        let mut cur_y = y;

        loop {
            let z_diff = z - cur_z;
            let pow_diff = (1u32 << z_diff) as f32;
            let dx = (x % (1u32 << z_diff)) as f32;
            let dy = (y % (1u32 << z_diff)) as f32;
            let uv = [
                dx / pow_diff,
                dy / pow_diff,
                (dx + 1.0) / pow_diff,
                (dy + 1.0) / pow_diff,
            ];

            let tex = if cur_z < BASE_LOD {
                self.0
                    .base_cache
                    .lock()
                    .unwrap()
                    .get(&(cur_z, cur_x, cur_y))
                    .cloned()
            } else {
                self.0
                    .tile_cache
                    .lock()
                    .unwrap()
                    .get(&(cur_z, cur_x, cur_y))
                    .cloned()
            };

            if let Some(tex) = tex {
                if cur_z == z {
                    self.0.hits.fetch_add(1, Ordering::Relaxed);
                } else {
                    self.0.misses.fetch_add(1, Ordering::Relaxed);
                }
                return Some((tex, uv));
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
            cache_size: self.0.tile_cache.lock().unwrap().len()
                + self.0.base_cache.lock().unwrap().len(),
        }
    }

    /// Reset hit/miss counters at most once every `STATS_DECAY_SECS` so the
    /// debug overlay shows recent rates instead of monotonic totals.
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
async fn fetch_tile_wasm(manager_weak: Weak<TileManagerInner>, z: u8, x: u32, y: u32) {
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
        Ok(img) => manager.insert_texture(img, z, x, y),
        Err(_) => {
            manager.errors.fetch_add(1, Ordering::Relaxed);
            manager.ctx.request_repaint();
        }
    }
    manager.pending.lock().unwrap().remove(&(z, x, y));
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use std::time::Duration;

    struct MockProvider {
        fetches: AtomicUsize,
    }

    impl MockProvider {
        fn new() -> Arc<Self> {
            Arc::new(Self {
                fetches: AtomicUsize::new(0),
            })
        }
    }

    impl providers::TileProvider for MockProvider {
        fn fetch_tile(&self, _z: u8, _x: u32, _y: u32) -> Result<Vec<u8>, String> {
            self.fetches.fetch_add(1, Ordering::SeqCst);
            let img = image::RgbaImage::from_pixel(4, 4, image::Rgba([10, 20, 30, 255]));
            let mut buf = std::io::Cursor::new(Vec::new());
            image::DynamicImage::ImageRgba8(img)
                .write_to(&mut buf, image::ImageFormat::Png)
                .map_err(|e| e.to_string())?;
            Ok(buf.into_inner())
        }
    }

    fn wait_until(mut condition: impl FnMut() -> bool) {
        for _ in 0..500 {
            if condition() {
                return;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        panic!("timed out waiting for tile worker");
    }

    #[test]
    fn request_fetches_once_and_caches() {
        let provider = MockProvider::new();
        let manager = SharedTileManager::with_provider(egui::Context::default(), provider.clone());

        manager.request(5, 1, 2);
        wait_until(|| manager.best_texture(5, 1, 2).is_some());

        let (_, uv) = manager.best_texture(5, 1, 2).unwrap();
        assert_eq!(
            uv,
            [0.0, 0.0, 1.0, 1.0],
            "exact hit must map the full texture"
        );

        // A second request for a cached tile must not hit the network again.
        manager.request(5, 1, 2);
        std::thread::sleep(Duration::from_millis(50));
        assert_eq!(provider.fetches.load(Ordering::SeqCst), 1);
        assert_eq!(manager.stats().pending, 0);
    }

    #[test]
    fn missing_tile_falls_back_to_ancestor_with_cropped_uv() {
        let provider = MockProvider::new();
        let manager = SharedTileManager::with_provider(egui::Context::default(), provider);

        manager.request(3, 0, 0);
        wait_until(|| manager.best_texture(3, 0, 0).is_some());

        // (4, 1, 1) is not cached; its parent (3, 0, 0) is. The fallback must
        // crop to the south-east quadrant of the parent texture.
        let (_, uv) = manager.best_texture(4, 1, 1).expect("ancestor fallback");
        assert_eq!(uv, [0.5, 0.5, 1.0, 1.0]);
    }
}
