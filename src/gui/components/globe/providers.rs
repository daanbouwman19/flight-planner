#[cfg(not(target_arch = "wasm32"))]
use crate::modules::http::HttpClient;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;

pub fn tile_url(z: u8, x: u32, y: u32) -> String {
    format!(
        "https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{}/{}/{}",
        z, y, x
    )
}

#[cfg(not(target_arch = "wasm32"))]
pub trait TileProvider: Send + Sync {
    fn fetch_tile(&self, z: u8, x: u32, y: u32) -> Result<Vec<u8>, String>;
}

#[cfg(not(target_arch = "wasm32"))]
pub struct ArcGisTileProvider {
    client: Arc<dyn HttpClient>,
}

#[cfg(not(target_arch = "wasm32"))]
impl ArcGisTileProvider {
    pub fn new(client: Arc<dyn HttpClient>) -> Self {
        Self { client }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl TileProvider for ArcGisTileProvider {
    fn fetch_tile(&self, z: u8, x: u32, y: u32) -> Result<Vec<u8>, String> {
        let url = tile_url(z, x, y);
        let (bytes, status) = self.client.get_bytes(&url, None)?;
        if !(200..300).contains(&status) {
            return Err(format!("HTTP error fetching tile: {}", status));
        }
        Ok(bytes)
    }
}
