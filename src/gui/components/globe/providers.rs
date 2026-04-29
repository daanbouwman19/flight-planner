use crate::modules::http::HttpClient;
use std::sync::Arc;

/// A trait for providing map tiles.
pub trait TileProvider: Send + Sync {
    /// Fetches a tile for the given z, x, y coordinates and returns the raw image bytes.
    fn fetch_tile(&self, z: u8, x: u32, y: u32) -> Result<Vec<u8>, String>;
}

/// An implementation of `TileProvider` that fetches tiles from the ArcGIS REST API.
pub struct ArcGisTileProvider {
    client: Arc<dyn HttpClient>,
}

impl ArcGisTileProvider {
    pub fn new(client: Arc<dyn HttpClient>) -> Self {
        Self { client }
    }
}

impl TileProvider for ArcGisTileProvider {
    fn fetch_tile(&self, z: u8, x: u32, y: u32) -> Result<Vec<u8>, String> {
        let url = format!(
            "https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{}/{}/{}",
            z, y, x
        );

        let (bytes, status) = self.client.get_bytes(&url, None)?;

        if status < 200 || status >= 300 {
            return Err(format!("HTTP error fetching tile: {}", status));
        }

        Ok(bytes)
    }
}
