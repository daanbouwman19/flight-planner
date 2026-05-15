use serde::{Deserialize, Serialize};

/// Enriched history record returned by `GET /api/history`.
///
/// The server resolves aircraft + airport names and computes distance so the
/// frontend doesn't need a local copy of the airport database.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HistoryItemResponse {
    pub id: i32,
    pub departure_icao: String,
    pub departure_name: String,
    pub arrival_icao: String,
    pub arrival_name: String,
    pub aircraft_id: i32,
    pub aircraft_name: String,
    pub date: String,
    pub distance_nm: i32,
}

/// Paginated response for `GET /api/history?limit=N&offset=N`.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HistoryPageResponse {
    pub items: Vec<HistoryItemResponse>,
    pub has_more: bool,
}
