use crate::gui::services::{AppService, WeatherService};
use crate::models::{Aircraft, Airport, Runway};
use crate::traits::HistoryOperations;
use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderValue, StatusCode},
    response::IntoResponse,
    routing::{get, post, put},
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tokio::sync::Mutex;
use tower_http::{cors::{AllowOrigin, CorsLayer}, services::ServeDir};

/// Shared application state for all request handlers.
pub struct AppState {
    pub app_service: Mutex<AppService>,
    pub weather_service: Mutex<WeatherService>,
    /// Runway data pre-grouped by airport ID at startup (static data, never changes).
    pub cached_runways: HashMap<i32, Vec<Runway>>,
}

impl AppState {
    pub fn new(mut app_service: AppService, weather_service: WeatherService) -> Self {
        let cached_runways = app_service
            .database_pool()
            .get_runways()
            .unwrap_or_default()
            .into_iter()
            .fold(HashMap::<i32, Vec<Runway>>::new(), |mut map, r| {
                map.entry(r.AirportID).or_default().push(r);
                map
            });
        Self {
            app_service: Mutex::new(app_service),
            weather_service: Mutex::new(weather_service),
            cached_runways,
        }
    }
}

type SharedState = Arc<AppState>;

// --- DTOs ---

#[derive(Serialize)]
struct AircraftResponse {
    id: i32,
    manufacturer: String,
    variant: String,
    icao_code: String,
    aircraft_range: i32,
    takeoff_distance: Option<i32>,
    cruise_speed: i32,
    category: String,
    flown: i32,
    date_flown: Option<String>,
}

impl From<&Aircraft> for AircraftResponse {
    fn from(a: &Aircraft) -> Self {
        Self {
            id: a.id,
            manufacturer: a.manufacturer.clone(),
            variant: a.variant.clone(),
            icao_code: a.icao_code.clone(),
            aircraft_range: a.aircraft_range,
            takeoff_distance: a.takeoff_distance,
            cruise_speed: a.cruise_speed,
            category: a.category.clone(),
            flown: a.flown,
            date_flown: a.date_flown.clone(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct AddHistoryRequest {
    pub aircraft_id: i32,
    pub departure_icao: String,
    pub arrival_icao: String,
}

#[derive(Serialize, Deserialize)]
pub struct SettingRequest {
    pub key: String,
    pub value: String,
}

// --- Handlers ---

async fn get_aircraft(State(state): State<SharedState>) -> impl IntoResponse {
    let svc = state.app_service.lock().await;
    let aircraft: Vec<AircraftResponse> =
        svc.aircraft().iter().map(|a| a.as_ref().into()).collect();
    Json(aircraft)
}

async fn toggle_flown(State(state): State<SharedState>, Path(id): Path<i32>) -> impl IntoResponse {
    let mut svc = state.app_service.lock().await;
    match svc.toggle_aircraft_flown_status(id) {
        Ok(()) => StatusCode::OK.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn reset_flown(State(state): State<SharedState>) -> impl IntoResponse {
    let mut svc = state.app_service.lock().await;
    match svc.mark_all_aircraft_as_not_flown() {
        Ok(()) => StatusCode::OK.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn get_airports(State(state): State<SharedState>) -> impl IntoResponse {
    let svc = state.app_service.lock().await;
    let airports: Vec<Airport> = svc.airports().iter().map(|a| a.as_ref().clone()).collect();
    Json(airports)
}

async fn get_runways(State(state): State<SharedState>) -> impl IntoResponse {
    let by_airport: HashMap<String, Vec<Runway>> = state
        .cached_runways
        .iter()
        .map(|(k, v)| (k.to_string(), v.clone()))
        .collect();
    Json(by_airport)
}

async fn get_history(State(state): State<SharedState>) -> impl IntoResponse {
    let mut svc = state.app_service.lock().await;
    let pool = svc.database_pool();
    match pool.get_history() {
        Ok(history) => Json(history).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn add_history(
    State(state): State<SharedState>,
    Json(req): Json<AddHistoryRequest>,
) -> impl IntoResponse {
    let mut svc = state.app_service.lock().await;

    let aircraft = svc
        .aircraft()
        .iter()
        .find(|a| a.id == req.aircraft_id)
        .cloned();
    let departure = svc.get_airport_by_icao(&req.departure_icao);
    let arrival = svc.get_airport_by_icao(&req.arrival_icao);

    match (aircraft, departure, arrival) {
        (Some(a), Some(dep), Some(arr)) => match svc.add_history_entry(&a, &dep, &arr) {
            Ok(()) => StatusCode::CREATED.into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        },
        _ => (StatusCode::BAD_REQUEST, "Aircraft or airport not found").into_response(),
    }
}

async fn get_weather(
    State(state): State<SharedState>,
    Path(icao): Path<String>,
) -> impl IntoResponse {
    let svc = state.weather_service.lock().await;
    match svc.fetch_metar(&icao) {
        Ok(metar) => Json(metar).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn get_settings(State(state): State<SharedState>) -> impl IntoResponse {
    let mut svc = state.app_service.lock().await;
    let mut settings = HashMap::<String, String>::new();
    if let Ok(Some(v)) = svc.get_setting("api_key") {
        settings.insert("api_key".to_string(), v);
    }
    Json(settings)
}

async fn save_setting(
    State(state): State<SharedState>,
    Json(req): Json<SettingRequest>,
) -> StatusCode {
    let is_api_key_and_ok: Result<bool, String> = {
        let mut svc = state.app_service.lock().await;
        svc.set_setting(&req.key, &req.value)
            .map(|()| req.key == "api_key")
            .map_err(|e| e.to_string())
    };
    match is_api_key_and_ok {
        Ok(true) => {
            let mut wx = state.weather_service.lock().await;
            wx.update_api_key(req.value);
            StatusCode::OK
        }
        Ok(false) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

/// Starts the axum HTTP server, serving the REST API and static WASM frontend files.
pub async fn run_server(
    app_state: AppState,
    dist_dir: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let state: SharedState = Arc::new(app_state);

    let api_router = Router::new()
        .route("/aircraft", get(get_aircraft))
        .route("/aircraft/reset", post(reset_flown))
        .route("/aircraft/{id}/flown", put(toggle_flown))
        .route("/airports", get(get_airports))
        .route("/runways", get(get_runways))
        .route("/history", get(get_history).post(add_history))
        .route("/weather/{icao}", get(get_weather))
        .route("/settings", get(get_settings).post(save_setting))
        .with_state(Arc::clone(&state));

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(|origin: &HeaderValue, _| {
            let s = origin.to_str().unwrap_or("");
            s.starts_with("http://localhost") || s.starts_with("http://127.0.0.1")
        }))
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any);

    let app = Router::new()
        .nest("/api", api_router)
        .fallback_service(ServeDir::new(&dist_dir))
        .layer(cors);

    let addr = std::env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    log::info!("Web server listening on http://{}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}
