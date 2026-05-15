// Native-only modules (use diesel, file system, or terminal I/O)
#[cfg(not(target_arch = "wasm32"))]
pub mod cli;
#[cfg(not(target_arch = "wasm32"))]
pub mod console_utils;
#[cfg(not(target_arch = "wasm32"))]
pub mod database;
#[cfg(not(target_arch = "wasm32"))]
pub mod errors;
#[cfg(not(target_arch = "wasm32"))]
pub mod schema;
#[cfg(all(not(target_arch = "wasm32"), any(test, debug_assertions)))]
pub mod test_helpers;
pub mod traits;

// Modules available for both native and WASM
pub mod date_utils;
pub mod models;
pub mod modules;
pub mod util;

// GUI module — shared rendering code used by both desktop (gui) and web (web) builds
#[cfg(any(feature = "gui", feature = "web"))]
pub mod gui;

// Backend HTTP server (native only)
#[cfg(all(feature = "server", not(target_arch = "wasm32")))]
pub mod server;

// WASM web module
#[cfg(target_arch = "wasm32")]
pub mod web;

// ---- Native-only imports and code below ----

#[cfg(not(target_arch = "wasm32"))]
use std::path::{Path, PathBuf};

#[cfg(not(target_arch = "wasm32"))]
use diesel::prelude::*;
#[cfg(not(target_arch = "wasm32"))]
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
#[cfg(not(target_arch = "wasm32"))]
use log::LevelFilter;
#[cfg(not(target_arch = "wasm32"))]
use log4rs::append::console::ConsoleAppender;
#[cfg(not(target_arch = "wasm32"))]
use log4rs::append::file::FileAppender;
#[cfg(not(target_arch = "wasm32"))]
use log4rs::encode::pattern::PatternEncoder;

#[cfg(not(target_arch = "wasm32"))]
use crate::database::{DatabasePool, get_airport_db_path, get_install_shared_data_dir};
#[cfg(not(target_arch = "wasm32"))]
use crate::errors::Error;

#[cfg(all(feature = "gui", not(target_arch = "wasm32")))]
use {
    eframe::{AppCreator, egui_wgpu, egui_wgpu::WgpuSetupCreateNew, wgpu},
    egui::ViewportBuilder,
    std::sync::Arc,
};

// SQL helper and migrations (native only)
#[cfg(not(target_arch = "wasm32"))]
define_sql_function! {fn random() -> Text }
#[cfg(not(target_arch = "wasm32"))]
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
#[cfg(all(feature = "gui", not(target_arch = "wasm32")))]
const APP_ID: &str = "com.github.daan.flight-planner";

// ---- WASM entry point ----

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn web_main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Info).ok();
    log::info!("Flight Planner web starting...");

    let web_options = eframe::WebOptions::default();
    wasm_bindgen_futures::spawn_local(async move {
        let canvas = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id("canvas"))
            .and_then(|e| e.dyn_into::<web_sys::HtmlCanvasElement>().ok())
            .expect("canvas element not found");

        eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| {
                    let mut fonts = egui::FontDefinitions::default();
                    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
                    cc.egui_ctx.set_fonts(fonts);
                    Ok(Box::new(
                        gui::ui::Gui::new_web(cc).expect("web GUI init failed"),
                    ))
                }),
            )
            .await
            .expect("failed to start eframe web");
    });
}

// ---- Native-only application code ----

/// Initialize logging and run the application.
#[cfg(all(not(tarpaulin_include), not(target_arch = "wasm32")))]
pub fn run_app() {
    match internal_run_app() {
        Ok(()) => {}
        Err(e) => {
            eprintln!("Application failed to start: {e}");
            log::error!("Startup error: {e}");
        }
    }
}

/// Get the application data directory in the user's home folder.
#[cfg(not(target_arch = "wasm32"))]
pub fn get_app_data_dir() -> Result<PathBuf, Error> {
    if let Some(dir) = crate::util::validate_env_path("FLIGHT_PLANNER_DATA_DIR") {
        return Ok(dir);
    }

    let Some(data_dir) = dirs::data_dir() else {
        return Err(Error::Other(std::io::Error::other(
            "Failed to resolve system data directory",
        )));
    };

    #[cfg(target_os = "windows")]
    let app_data_dir = data_dir.join("FlightPlanner");

    #[cfg(not(target_os = "windows"))]
    let app_data_dir = data_dir.join("flight-planner");

    if !app_data_dir.exists() {
        std::fs::create_dir_all(&app_data_dir)?;
    }

    Ok(app_data_dir)
}

/// (For testing and internal use) Get the candidate paths for `aircrafts.csv`
#[doc(hidden)]
#[cfg(not(target_arch = "wasm32"))]
pub fn get_aircraft_csv_candidate_paths() -> Vec<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Ok(app_data_dir) = get_app_data_dir() {
        candidates.push(app_data_dir.join("aircrafts.csv"));
    }

    candidates.push(PathBuf::from("aircrafts.csv"));

    if let Ok(shared_dir) = get_install_shared_data_dir() {
        candidates.push(shared_dir.join("aircrafts.csv"));
    }

    candidates
}

/// Simple GUI to show the airport database warning
#[cfg(all(feature = "gui", not(target_arch = "wasm32")))]
struct AirportDatabaseWarning {
    app_data_dir: PathBuf,
}

#[cfg(all(feature = "gui", not(target_arch = "wasm32")))]
impl AirportDatabaseWarning {
    fn new(app_data_dir: &Path) -> Self {
        Self {
            app_data_dir: app_data_dir.to_path_buf(),
        }
    }
}

#[cfg(all(feature = "gui", not(target_arch = "wasm32")))]
impl eframe::App for AirportDatabaseWarning {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                ui.heading("❌ Missing Airports Database");
                ui.add_space(20.0);
                ui.label(
                    "The Flight Planner requires an airports database file (airports.db3) to function.",
                );
                ui.label(
                    "This file is not included with the application and must be provided by the user.",
                );
                ui.add_space(20.0);
                ui.label("📁 Application data directory:");
                ui.code(format!("{}", self.app_data_dir.display()));
                ui.add_space(20.0);
                ui.label("📋 To fix this issue:");
                ui.label("1. Obtain an airports database file (airports.db3)");
                ui.label(format!("2. Copy it to: {}", self.app_data_dir.display()));
                ui.label("3. Restart the application");
                ui.add_space(20.0);
                ui.label(
                    "💡 Alternative: Run the application from the directory containing airports.db3",
                );
                ui.add_space(20.0);
                if ui.button("Close Application").clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
        });
    }
}

/// Main application startup logic
#[cfg(all(not(tarpaulin_include), not(target_arch = "wasm32")))]
fn internal_run_app() -> Result<(), Error> {
    #[cfg(feature = "dotenvy")]
    {
        dotenvy::dotenv().ok();
    }
    let app_data_dir = get_app_data_dir()?;
    let logs_dir = app_data_dir.join("logs");
    std::fs::create_dir_all(&logs_dir)?;
    let log_file_path = logs_dir.join("output.log");

    let console_appender = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{h({l})} - {m}{n}")))
        .build();

    let file_appender = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {l} - {m}{n}")))
        .build(&log_file_path)
        .map_err(|e| Error::LogConfig(format!("failed creating file appender: {e}")))?;

    let config = log4rs::Config::builder()
        .appender(log4rs::config::Appender::builder().build("console", Box::new(console_appender)))
        .appender(log4rs::config::Appender::builder().build("logfile", Box::new(file_appender)))
        .logger(log4rs::config::Logger::builder().build("wgpu_core", LevelFilter::Error))
        .logger(log4rs::config::Logger::builder().build("wgpu_hal", LevelFilter::Warn))
        .logger(log4rs::config::Logger::builder().build("egui_wgpu", LevelFilter::Warn))
        .logger(log4rs::config::Logger::builder().build("eframe", LevelFilter::Warn))
        .logger(log4rs::config::Logger::builder().build("egui", LevelFilter::Warn))
        .build(
            log4rs::config::Root::builder()
                .appender("console")
                .appender("logfile")
                .build(LevelFilter::Info),
        )
        .map_err(|e| Error::LogConfig(format!("failed building log4rs config: {e}")))?;

    log4rs::init_config(config)
        .map_err(|e| Error::LogConfig(format!("failed initializing log4rs: {e}")))?;

    let airport_db_path = get_airport_db_path()?;
    if !airport_db_path.exists() {
        show_airport_database_warning(&airport_db_path, &app_data_dir);
        return Ok(());
    }

    if let Err(e) = run() {
        log::error!("Application error: {e}");
    }
    Ok(())
}

/// Run database migrations on both aircraft and airport databases.
#[cfg(not(target_arch = "wasm32"))]
pub fn run_database_migrations(database_pool: &DatabasePool) -> Result<(), Error> {
    database_pool
        .aircraft_pool
        .get()?
        .run_pending_migrations(MIGRATIONS)
        .map_err(|e| Error::Migration(e.to_string()))?;

    database_pool
        .airport_pool
        .get()?
        .run_pending_migrations(MIGRATIONS)
        .map_err(|e| Error::Migration(e.to_string()))?;

    Ok(())
}

/// Import aircraft from CSV if the database table is empty.
#[cfg(not(target_arch = "wasm32"))]
pub fn import_aircraft_csv_if_empty(database_pool: &DatabasePool) {
    if let Some(csv_path) = find_aircraft_csv_path() {
        match database_pool.aircraft_pool.get() {
            Ok(mut conn) => {
                match crate::modules::aircraft::import_aircraft_from_csv_if_empty(
                    &mut conn, &csv_path,
                ) {
                    Ok(true) => log::info!(
                        "Aircraft table was empty. Imported from {}",
                        csv_path.display()
                    ),
                    Ok(false) => log::debug!(
                        "Aircraft table not empty or no rows to import from {}",
                        csv_path.display()
                    ),
                    Err(e) => {
                        log::warn!(
                            "Failed to import aircraft from {}: {}",
                            csv_path.display(),
                            e
                        )
                    }
                }
            }
            Err(e) => log::warn!("Failed to get DB connection for import: {e}"),
        }
    } else {
        log::debug!("No aircrafts.csv found in common locations; skipping import");
    }
}

/// Core application logic after initialization
#[cfg(all(not(tarpaulin_include), not(target_arch = "wasm32")))]
fn run() -> Result<(), Error> {
    log::info!("Starting application run sequence...");

    let args: Vec<String> = std::env::args().collect();

    // Server mode
    #[cfg(all(feature = "server", not(target_arch = "wasm32")))]
    if args.len() > 1 && args[1] == "--web" {
        let database_pool = DatabasePool::new(None, None)?;
        run_database_migrations(&database_pool)?;
        crate::database::apply_database_optimizations(&database_pool)?;
        import_aircraft_csv_if_empty(&database_pool);
        let mut app_service = gui::services::AppService::new(database_pool.clone())
            .map_err(|e| Error::Other(std::io::Error::other(e.to_string())))?;
        let api_key = app_service
            .get_api_key()
            .unwrap_or_default()
            .unwrap_or_default();
        let weather_service = gui::services::WeatherService::new(api_key, database_pool);
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| Error::Other(std::io::Error::other(e.to_string())))?;
        rt.block_on(server::run_server(
            server::AppState::new(app_service, weather_service),
            std::path::PathBuf::from("dist"),
        ))
        .map_err(|e| Error::Other(std::io::Error::other(e.to_string())))?;
        return Ok(());
    }

    #[cfg(all(feature = "gui", not(target_arch = "wasm32")))]
    let mut use_cli = false;

    if args.len() > 1 && (args[1] == "--cli" || args[1] == "-c") {
        #[cfg(all(feature = "gui", not(target_arch = "wasm32")))]
        {
            use_cli = true;
        }
    }

    #[cfg(all(feature = "gui", not(target_arch = "wasm32")))]
    if !use_cli {
        let icon_data = load_icon_for_eframe();

        let native_options = eframe::NativeOptions {
            viewport: ViewportBuilder {
                inner_size: Some(egui::vec2(1200.0, 768.0)),
                close_button: Some(true),
                icon: icon_data,
                title: Some("Flight Planner".to_string()),
                app_id: Some(APP_ID.to_string()),
                ..Default::default()
            },
            wgpu_options: egui_wgpu::WgpuConfiguration {
                wgpu_setup: egui_wgpu::WgpuSetup::CreateNew(WgpuSetupCreateNew {
                    instance_descriptor: wgpu::InstanceDescriptor {
                        backends: wgpu::Backends::VULKAN,
                        flags: wgpu::InstanceFlags::default(),
                        memory_budget_thresholds: wgpu::MemoryBudgetThresholds::default(),
                        backend_options: wgpu::BackendOptions::default(),
                        display: None,
                    },
                    power_preference: wgpu::PowerPreference::default(),
                    native_adapter_selector: None,
                    display_handle: None,
                    device_descriptor: Arc::new(|adapter| {
                        let base_limits = if adapter.get_info().backend == wgpu::Backend::Gl {
                            wgpu::Limits::downlevel_webgl2_defaults()
                        } else {
                            wgpu::Limits::default()
                        };
                        wgpu::DeviceDescriptor {
                            label: Some("flight planner wgpu device"),
                            required_features: wgpu::Features::default(),
                            required_limits: base_limits,
                            memory_hints: wgpu::MemoryHints::default(),
                            trace: wgpu::Trace::Off,
                            experimental_features: wgpu::ExperimentalFeatures::default(),
                        }
                    }),
                }),
                present_mode: wgpu::PresentMode::AutoVsync,
                desired_maximum_frame_latency: Some(2),
                on_surface_status: Arc::new(|_| egui_wgpu::SurfaceErrorAction::SkipFrame),
            },
            ..Default::default()
        };

        let app_creator: AppCreator<'_> = Box::new(|cc| {
            let mut fonts = egui::FontDefinitions::default();
            egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
            cc.egui_ctx.set_fonts(fonts);

            log::info!("Initializing Gui...");
            let start = std::time::Instant::now();
            match gui::ui::Gui::new(cc, None) {
                Ok(gui) => {
                    log::info!("Gui initialized in {:?}", start.elapsed());
                    Ok(Box::new(gui))
                }
                Err(e) => {
                    log::error!("Failed to create GUI: {e}");
                    Err(Box::new(std::io::Error::other(
                        "Failed to initialize application",
                    )))
                }
            }
        });
        log::info!("Starting eframe::run_native...");
        _ = eframe::run_native("Flight Planner", native_options, app_creator);
        return Ok(());
    }

    // CLI Mode or Non-GUI build
    let database_pool = DatabasePool::new(None, None)?;
    println!("Database pool created.");

    run_database_migrations(&database_pool)?;
    println!("Database migrations completed.");

    crate::database::apply_database_optimizations(&database_pool)?;
    println!("Database optimizations applied.");

    import_aircraft_csv_if_empty(&database_pool);

    #[cfg(all(feature = "gui", not(target_arch = "wasm32")))]
    if use_cli {
        cli::console_main(database_pool, cli::ConsoleInteraction::new())?;
    }

    #[cfg(not(feature = "gui"))]
    cli::console_main(database_pool, cli::ConsoleInteraction::new())?;
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn find_aircraft_csv_path() -> Option<PathBuf> {
    let candidates = get_aircraft_csv_candidate_paths();
    candidates.into_iter().find(|path| path.exists())
}

#[cfg(not(target_arch = "wasm32"))]
fn log_db_warning(airport_db_path: &Path, app_data_dir: &Path) {
    log::error!(
        "Airports database not found at {}",
        airport_db_path.display()
    );
    log::error!(
        "Please place your airports.db3 file in: {}",
        app_data_dir.display()
    );
}

#[cfg(not(target_arch = "wasm32"))]
fn print_db_warning_to_console(app_data_dir: &Path) {
    println!();
    println!("❌ ERROR: Airports database not found!");
    println!();
    println!("The Flight Planner requires an airports database file (airports.db3) to function.");
    println!("This file is not included with the application and must be provided by the user.");
    println!();
    println!("📁 Application data directory: {}", app_data_dir.display());
    println!();
    println!("📋 To fix this issue:");
    println!("   1. Obtain an airports database file (airports.db3)");
    println!("   2. Copy it to: {}", app_data_dir.display());
    println!("   3. Run the application again");
    println!();
    println!("💡 Alternative: Run the application from the directory containing airports.db3");
    println!();
}

#[cfg(all(not(tarpaulin_include), feature = "gui", not(target_arch = "wasm32")))]
fn show_airport_database_warning(airport_db_path: &Path, app_data_dir: &Path) {
    log_db_warning(airport_db_path, app_data_dir);

    let is_cli_mode = std::env::args().any(|arg| arg == "--cli");

    if is_cli_mode {
        print_db_warning_to_console(app_data_dir);
    } else {
        let result = eframe::run_native(
            "Flight Planner - Missing Database",
            eframe::NativeOptions {
                viewport: ViewportBuilder {
                    inner_size: Some(egui::vec2(600.0, 400.0)),
                    resizable: Some(false),
                    ..Default::default()
                },
                ..Default::default()
            },
            Box::new(|_cc| Ok(Box::new(AirportDatabaseWarning::new(app_data_dir)))),
        );

        if let Err(e) = result {
            log::warn!(
                "Failed to display GUI warning dialog: {e}. Falling back to console output."
            );
            print_db_warning_to_console(app_data_dir);
        }
    }
}

#[cfg(all(
    not(tarpaulin_include),
    not(feature = "gui"),
    not(target_arch = "wasm32")
))]
fn show_airport_database_warning(airport_db_path: &Path, app_data_dir: &Path) {
    log_db_warning(airport_db_path, app_data_dir);
    print_db_warning_to_console(app_data_dir);
}

#[cfg(all(not(tarpaulin_include), feature = "gui", not(target_arch = "wasm32")))]
fn load_icon_for_eframe() -> Option<Arc<egui::IconData>> {
    let icon_bytes = include_bytes!("../assets/icons/icon-64x64.png");

    match image::load_from_memory_with_format(icon_bytes, image::ImageFormat::Png) {
        Ok(img) => {
            let rgba_img = img.to_rgba8();
            let (width, height) = rgba_img.dimensions();
            log::info!("Loaded icon with dimensions {width}x{height} for eframe");
            Some(Arc::from(egui::IconData {
                rgba: rgba_img.into_raw(),
                width,
                height,
            }))
        }
        Err(e) => {
            log::warn!("Failed to load icon: {e}. Application will run without icon.");
            None
        }
    }
}
