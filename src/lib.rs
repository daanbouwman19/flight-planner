use diesel::prelude::*;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
#[cfg(feature = "gui")]
use eframe::egui_wgpu;
#[cfg(feature = "gui")]
use eframe::egui_wgpu::WgpuSetupCreateNew;
#[cfg(feature = "gui")]
use eframe::wgpu;
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::encode::pattern::PatternEncoder;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::database::{DatabasePool, get_airport_db_path, get_install_shared_data_dir};
use crate::errors::Error;
#[cfg(feature = "gui")]
use eframe::AppCreator;
#[cfg(feature = "gui")]
use egui::ViewportBuilder;

// Application identifier - must match the desktop file name (without .desktop extension)
const APP_ID: &str = "com.github.daan.flight-planner";

/// Get the application data directory in the user's home folder.
///
/// This creates a dedicated directory for storing logs, databases, and other
/// application data. The directory structure follows platform conventions:
/// - **Linux**: `~/.local/share/flight-planner/`
/// - **macOS**: `~/Library/Application Support/flight-planner/`
/// - **Windows**: `%APPDATA%\FlightPlanner\`
///
/// # Returns
///
/// A `Result` containing the `PathBuf` to the application data directory on
/// success, or an `Error` if the directory cannot be resolved or created.
pub fn get_app_data_dir() -> Result<PathBuf, Error> {
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

/// Try to locate an aircrafts.csv file in common locations
fn find_aircraft_csv_path() -> Option<PathBuf> {
    let candidates = get_aircraft_csv_candidate_paths();

    candidates.into_iter().find(|path| path.exists())
}

/// (For testing and internal use) Get the candidate paths for `aircrafts.csv`
pub fn get_aircraft_csv_candidate_paths() -> Vec<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Ok(app_data_dir) = get_app_data_dir() {
        candidates.push(app_data_dir.join("aircrafts.csv"));
    }

    // Current working directory
    candidates.push(PathBuf::from("aircrafts.csv"));

    // System-wide install location via helper
    if let Ok(shared_dir) = get_install_shared_data_dir() {
        candidates.push(shared_dir.join("aircrafts.csv"));
    }

    candidates
}

/// Logs a standardized error message when the airport database is not found.
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

/// Prints a standardized error message to the console when the airport database is not found.
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

/// Show a warning when the airports database is not found
#[cfg(feature = "gui")]
fn show_airport_database_warning(airport_db_path: &Path, app_data_dir: &Path) {
    log_db_warning(airport_db_path, app_data_dir);

    // Check if we're running in CLI mode
    let is_cli_mode = std::env::args().any(|arg| arg == "--cli");

    if is_cli_mode {
        print_db_warning_to_console(app_data_dir);
    } else {
        // GUI mode - show a simple dialog
        let _ = eframe::run_native(
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
    }
}

/// Show a warning when the airports database is not found (CLI-only version)
#[cfg(not(feature = "gui"))]
fn show_airport_database_warning(airport_db_path: &Path, app_data_dir: &Path) {
    log_db_warning(airport_db_path, app_data_dir);
    print_db_warning_to_console(app_data_dir);
}

/// Simple GUI to show the airport database warning
#[cfg(feature = "gui")]
struct AirportDatabaseWarning {
    app_data_dir: PathBuf,
}

#[cfg(feature = "gui")]
impl AirportDatabaseWarning {
    fn new(app_data_dir: &Path) -> Self {
        Self {
            app_data_dir: app_data_dir.to_path_buf(),
        }
    }
}

#[cfg(feature = "gui")]
impl eframe::App for AirportDatabaseWarning {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);

                // Title
                ui.heading("❌ Missing Airports Database");
                ui.add_space(20.0);

                // Error message
                ui.label("The Flight Planner requires an airports database file (airports.db3) to function.");
                ui.label("This file is not included with the application and must be provided by the user.");
                ui.add_space(20.0);

                // Application data directory
                ui.label("📁 Application data directory:");
                ui.code(format!("{}", self.app_data_dir.display()));
                ui.add_space(20.0);

                // Instructions
                ui.label("📋 To fix this issue:");
                ui.label("1. Obtain an airports database file (airports.db3)");
                ui.label(format!("2. Copy it to: {}", self.app_data_dir.display()));
                ui.label("3. Restart the application");
                ui.add_space(20.0);

                ui.label("💡 Alternative: Run the application from the directory containing airports.db3");
                ui.add_space(20.0);

                // Close button
                if ui.button("Close Application").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
        });
    }
}

pub mod cli;
pub mod console_utils;
pub mod database;
pub mod date_utils;
pub mod errors;
#[cfg(feature = "gui")]
pub mod gui;
pub mod models;
pub mod modules;
pub mod schema;
#[cfg(any(test, debug_assertions))]
pub mod test_helpers;
pub mod traits;
pub mod util;

define_sql_function! {fn random() -> Text }

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// Load icon for eframe (used on X11, fallback on Wayland)
///
/// This function loads the icon for eframe's ViewportBuilder.
/// On Wayland, the desktop file approach is used instead, but this
/// provides fallback support for X11 and other platforms.
/// Uses a properly sized 64x64 icon for optimal display quality.
#[cfg(feature = "gui")]
fn load_icon_for_eframe() -> Option<Arc<egui::IconData>> {
    let icon_bytes = include_bytes!("../assets/icons/icon-64x64.png");

    match image::load_from_memory_with_format(icon_bytes, image::ImageFormat::Png) {
        Ok(img) => {
            // Convert to RGBA8 format and use original dimensions
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

/// Initialize logging and run the application.
///
/// This is the main entry point of the application. It sets up logging,
/// checks for the necessary database files, and then launches either the
/// command-line interface (CLI) or the graphical user interface (GUI).
///
/// Any errors that occur during startup are logged and printed to the console.
pub fn run_app() {
    match internal_run_app() {
        Ok(()) => {}
        Err(e) => {
            eprintln!("Application failed to start: {e}");
            log::error!("Startup error: {e}");
        }
    }
}

fn internal_run_app() -> Result<(), Error> {
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

fn run() -> Result<(), Error> {
    let database_pool = DatabasePool::new(None, None)?;
    let mut use_cli = false;

    for arg in std::env::args() {
        if arg == "--cli" {
            use_cli = true;
        }
    }

    database_pool
        .aircraft_pool
        .get()?
        .run_pending_migrations(MIGRATIONS)
        .map_err(|e| Error::Migration(e.to_string()))?;

    // After migrations, auto-import aircraft CSV if table is empty
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
                    Err(e) => log::warn!(
                        "Failed to import aircraft from {}: {}",
                        csv_path.display(),
                        e
                    ),
                }
            }
            Err(e) => log::warn!("Failed to get DB connection for import: {e}"),
        }
    } else {
        log::debug!("No aircrafts.csv found in common locations; skipping import");
    }

    #[cfg(feature = "gui")]
    if use_cli {
        cli::console_main(database_pool)?;
    } else {
        // Load and prepare icon with Wayland compatibility
        let icon_data = load_icon_for_eframe();

        let native_options = eframe::NativeOptions {
            viewport: ViewportBuilder {
                inner_size: Some(egui::vec2(1200.0, 768.0)),
                close_button: Some(true),
                icon: icon_data,
                title: Some("Flight Planner".to_string()),
                // Set application class name for better Wayland compositor integration
                // This must match the desktop file name (without .desktop extension)
                app_id: Some(APP_ID.to_string()),
                ..Default::default()
            },
            wgpu_options: egui_wgpu::WgpuConfiguration {
                wgpu_setup: egui_wgpu::WgpuSetup::CreateNew(WgpuSetupCreateNew {
                    instance_descriptor: wgpu::InstanceDescriptor {
                        backends: wgpu::Backends::VULKAN,
                        ..Default::default()
                    },
                    power_preference: wgpu::PowerPreference::default(),
                    native_adapter_selector: None,
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
                on_surface_error: Arc::new(|_| egui_wgpu::SurfaceErrorAction::SkipFrame),
            },
            ..Default::default()
        };

        let app_creator: AppCreator<'_> =
            Box::new(|cc| match gui::ui::Gui::new(cc, database_pool) {
                Ok(gui) => Ok(Box::new(gui)),
                Err(e) => {
                    log::error!("Failed to create GUI: {e}");
                    Err(Box::new(std::io::Error::other(
                        "Failed to initialize application",
                    )))
                }
            });
        _ = eframe::run_native("Flight Planner", native_options, app_creator);
    }

    #[cfg(not(feature = "gui"))]
    // If the GUI feature is disabled, we default to the CLI.
    cli::console_main(database_pool)?;
    Ok(())
}

