use crate::gui::services::{AppService, WeatherService};
use crate::models::{Aircraft, Airport, HistoryItemResponse, RouteResponse, Runway};
use crate::modules::data_operations::DataOperations;
use crate::modules::routes::RouteGenerator;
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
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    services::ServeDir,
};

/// Shared application state for all request handlers.
pub struct AppState {
    pub app_service: Mutex<AppService>,
    pub weather_service: Mutex<WeatherService>,
    /// Runway data pre-grouped by airport ID at startup (static data, never changes).
    pub cached_runways: HashMap<i32, Vec<Runway>>,
    /// Route generator shared across route-generation requests (airports + spatial index).
    pub route_generator: Arc<RouteGenerator>,
    /// Airports indexed by ICAO for fast lookups during enrichment.
    pub airport_by_icao: HashMap<String, Arc<Airport>>,
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
        let route_generator = app_service.route_generator().clone();
        let airport_by_icao = app_service
            .airports()
            .iter()
            .map(|a| (a.ICAO.clone(), a.clone()))
            .collect();
        Self {
            app_service: Mutex::new(app_service),
            weather_service: Mutex::new(weather_service),
            cached_runways,
            route_generator,
            airport_by_icao,
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

#[derive(Deserialize)]
pub struct GenerateRoutesRequest {
    pub mode: String,
    pub aircraft_id: Option<i32>,
    pub departure_icao: Option<String>,
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

#[derive(Deserialize)]
pub struct AirportSearchQuery {
    pub q: Option<String>,
    pub limit: Option<usize>,
}

async fn search_airports(
    State(state): State<SharedState>,
    axum::extract::Query(query): axum::extract::Query<AirportSearchQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(50).min(200);
    let q = query.q.unwrap_or_default();
    let svc = state.app_service.lock().await;

    if q.trim().is_empty() {
        let airports: Vec<Airport> = svc
            .airports()
            .iter()
            .take(limit)
            .map(|a| a.as_ref().clone())
            .collect();
        return Json(airports);
    }

    let airports: Vec<Airport> = svc
        .airports()
        .iter()
        .filter(|a| {
            crate::util::contains_case_insensitive(&a.Name, &q)
                || crate::util::contains_case_insensitive(&a.ICAO, &q)
        })
        .take(limit)
        .map(|a| a.as_ref().clone())
        .collect();
    Json(airports)
}

#[derive(Deserialize)]
pub struct RandomAirportsQuery {
    pub n: Option<usize>,
}

async fn random_airports(
    State(state): State<SharedState>,
    axum::extract::Query(query): axum::extract::Query<RandomAirportsQuery>,
) -> impl IntoResponse {
    let n = query.n.unwrap_or(50).min(500);
    let svc = state.app_service.lock().await;
    let all = svc.airports();
    let airports: Vec<Airport> =
        crate::modules::data_operations::DataOperations::generate_random_airports(all, n)
            .into_iter()
            .map(|a| a.as_ref().clone())
            .collect();
    Json(airports)
}

async fn get_airport_by_icao(
    State(state): State<SharedState>,
    Path(icao): Path<String>,
) -> impl IntoResponse {
    match state.airport_by_icao.get(&icao) {
        Some(a) => Json((**a).clone()).into_response(),
        None => (StatusCode::NOT_FOUND, "Airport not found").into_response(),
    }
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
    let aircraft_by_id: HashMap<i32, Arc<Aircraft>> =
        svc.aircraft().iter().map(|a| (a.id, a.clone())).collect();
    let history = match svc.database_pool().get_history() {
        Ok(h) => h,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let response: Vec<HistoryItemResponse> = history
        .into_iter()
        .map(|h| {
            let aircraft_name = aircraft_by_id.get(&h.aircraft).map_or_else(
                || format!("Unknown Aircraft (ID: {})", h.aircraft),
                |a| format!("{} {}", a.manufacturer, a.variant),
            );
            let departure_name = state
                .airport_by_icao
                .get(&h.departure_icao)
                .map_or_else(|| "Unknown Airport".to_string(), |a| a.Name.clone());
            let arrival_name = state
                .airport_by_icao
                .get(&h.arrival_icao)
                .map_or_else(|| "Unknown Airport".to_string(), |a| a.Name.clone());
            let distance_nm = h.distance.unwrap_or_else(|| {
                match (
                    state.airport_by_icao.get(&h.departure_icao),
                    state.airport_by_icao.get(&h.arrival_icao),
                ) {
                    (Some(dep), Some(arr)) => {
                        crate::util::calculate_haversine_distance_nm(dep, arr)
                    }
                    _ => 0,
                }
            });
            HistoryItemResponse {
                id: h.id,
                departure_icao: h.departure_icao,
                departure_name,
                arrival_icao: h.arrival_icao,
                arrival_name,
                aircraft_id: h.aircraft,
                aircraft_name,
                date: h.date,
                distance_nm,
            }
        })
        .collect();

    Json(response).into_response()
}

async fn get_statistics(State(state): State<SharedState>) -> impl IntoResponse {
    let mut svc = state.app_service.lock().await;
    match svc.get_flight_statistics() {
        Ok(stats) => Json(stats).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[derive(Deserialize)]
pub struct RouteFromHistoryRequest {
    pub aircraft_id: i32,
    pub departure_icao: String,
    pub arrival_icao: String,
}

async fn route_from_history(
    State(state): State<SharedState>,
    Json(req): Json<RouteFromHistoryRequest>,
) -> impl IntoResponse {
    let svc = state.app_service.lock().await;
    let aircraft = match svc.aircraft().iter().find(|a| a.id == req.aircraft_id) {
        Some(a) => a.clone(),
        None => return (StatusCode::BAD_REQUEST, "Aircraft not found").into_response(),
    };
    let departure = match state.airport_by_icao.get(&req.departure_icao) {
        Some(a) => a.clone(),
        None => return (StatusCode::BAD_REQUEST, "Departure airport not found").into_response(),
    };
    let destination = match state.airport_by_icao.get(&req.arrival_icao) {
        Some(a) => a.clone(),
        None => return (StatusCode::BAD_REQUEST, "Arrival airport not found").into_response(),
    };
    let distance_nm = crate::util::calculate_haversine_distance_nm(&departure, &destination) as f64;
    let longest_runway = |airport_id: i32| {
        state
            .cached_runways
            .get(&airport_id)
            .and_then(|runs| runs.iter().map(|r| r.Length).max())
            .unwrap_or(0)
    };
    let response = RouteResponse {
        departure: (*departure).clone(),
        destination: (*destination).clone(),
        aircraft_id: aircraft.id,
        distance_nm,
        departure_runway_ft: longest_runway(departure.ID),
        destination_runway_ft: longest_runway(destination.ID),
    };
    Json(response).into_response()
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
    // WeatherService uses reqwest::blocking which creates its own tokio runtime;
    // run it on a dedicated blocking thread to avoid "cannot drop runtime" panics.
    let svc = state.weather_service.lock().await.clone();
    match tokio::task::spawn_blocking(move || svc.fetch_metar(&icao))
        .await
        .unwrap_or_else(|e| Err(crate::models::weather::WeatherError::Request(e.to_string())))
    {
        Ok(metar) => Json(metar).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn generate_routes(
    State(state): State<SharedState>,
    Json(req): Json<GenerateRoutesRequest>,
) -> impl IntoResponse {
    // Validate "specific" mode before entering the blocking task.
    if req.mode == "specific" && req.aircraft_id.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            "aircraft_id required for mode=specific",
        )
            .into_response();
    }

    let aircraft = state.app_service.lock().await.aircraft().to_vec();
    let rg = Arc::clone(&state.route_generator);

    let routes = tokio::task::spawn_blocking(move || match req.mode.as_str() {
        "not_flown" => {
            DataOperations::generate_not_flown_routes(&rg, &aircraft, req.departure_icao.as_deref())
        }
        "specific" => match aircraft.iter().find(|a| Some(a.id) == req.aircraft_id) {
            Some(a) => {
                DataOperations::generate_routes_for_aircraft(&rg, a, req.departure_icao.as_deref())
            }
            None => vec![],
        },
        _ => DataOperations::generate_random_routes(&rg, &aircraft, req.departure_icao.as_deref()),
    })
    .await
    .unwrap_or_default();

    let response: Vec<RouteResponse> = routes.iter().map(RouteResponse::from).collect();
    Json(response).into_response()
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
        .route("/airports/search", get(search_airports))
        .route("/airports/random", get(random_airports))
        .route("/airports/by-icao/{icao}", get(get_airport_by_icao))
        .route("/runways", get(get_runways))
        .route("/history", get(get_history).post(add_history))
        .route("/statistics", get(get_statistics))
        .route("/routes", post(generate_routes))
        .route("/routes/from-history", post(route_from_history))
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
