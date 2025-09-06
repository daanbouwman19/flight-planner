use diesel::prelude::*;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use eframe::egui_wgpu;
use eframe::egui_wgpu::WgpuSetupCreateNew;
use eframe::wgpu;
use log4rs::append::file::FileAppender;
use log4rs::encode::pattern::PatternEncoder;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use util::calculate_haversine_distance_nm;

use crate::console_utils::{ask_mark_flown, read_id, read_yn};
use crate::database::{get_airport_db_path, DatabasePool};
use crate::errors::Error;
use eframe::AppCreator;
use egui::ViewportBuilder;
use modules::aircraft::format_aircraft;
use modules::airport::format_airport;
use modules::runway::format_runway;
use traits::{AircraftOperations, AirportOperations, DatabaseOperations, HistoryOperations};

// Application identifier - must match the desktop file name (without .desktop extension)
const APP_ID: &str = "com.github.daan.flight-planner";

/// Get the application data directory in the user's home folder
/// 
/// This creates a dedicated directory for storing logs, databases, and other
/// application data. The directory structure follows platform conventions:
/// - Linux: ~/.local/share/flight-planner/
/// - macOS: ~/Library/Application Support/flight-planner/
/// - Windows: %APPDATA%\FlightPlanner\
pub fn get_app_data_dir() -> PathBuf {
    let data_dir = dirs::data_dir().expect("Failed to get application data directory");

    #[cfg(target_os = "windows")]
    let app_data_dir = data_dir.join("FlightPlanner");
    
    #[cfg(not(target_os = "windows"))]
    let app_data_dir = data_dir.join("flight-planner");
    
    // Create the directory if it doesn't exist
    if !app_data_dir.exists() {
        std::fs::create_dir_all(&app_data_dir)
            .expect("Failed to create application data directory");
    }
    
    app_data_dir
}

/// Show a warning when the airports database is not found
fn show_airport_database_warning(airport_db_path: &Path, app_data_dir: &Path) {
    // Log the error for debugging
    log::error!("Airports database not found at {}", airport_db_path.display());
    log::error!("Please place your airports.db3 file in: {}", app_data_dir.display());
    
    // Check if we're running in CLI mode
    let is_cli_mode = std::env::args().any(|arg| arg == "--cli");
    
    if is_cli_mode {
        // Console output for CLI mode
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

/// Simple GUI to show the airport database warning
struct AirportDatabaseWarning {
    app_data_dir: PathBuf,
}

impl AirportDatabaseWarning {
    fn new(app_data_dir: &Path) -> Self {
        Self {
            app_data_dir: app_data_dir.to_path_buf(),
        }
    }
}

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

pub mod console_utils;
pub mod database;
pub mod date_utils;
pub mod errors;
pub mod gui;
pub mod models;
pub mod modules;
pub mod schema;
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
fn load_icon_for_eframe() -> Option<Arc<egui::IconData>> {
    let icon_bytes = include_bytes!("../assets/icons/icon-64x64.png");
    
    match image::load_from_memory_with_format(icon_bytes, image::ImageFormat::Png) {
        Ok(img) => {
            // Convert to RGBA8 format and use original dimensions
            let rgba_img = img.to_rgba8();
            let (width, height) = rgba_img.dimensions();
            
            log::info!("Loaded icon with dimensions {}x{} for eframe", width, height);
            Some(Arc::from(egui::IconData {
                rgba: rgba_img.into_raw(),
                width,
                height,
            }))
        }
        Err(e) => {
            log::warn!("Failed to load icon: {}. Application will run without icon.", e);
            None
        }
    }
}

/// Initialize logging and run the application
pub fn run_app() {
    // Get application data directory and create logs subdirectory
    let app_data_dir = get_app_data_dir();
    let logs_dir = app_data_dir.join("logs");
    std::fs::create_dir_all(&logs_dir).expect("Failed to create logs directory");
    
    let log_file_path = logs_dir.join("output.log");
    
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
        .build(&log_file_path)
        .unwrap();

    let config = log4rs::Config::builder()
        .appender(log4rs::config::Appender::builder().build("logfile", Box::new(logfile)))
        .build(
            log4rs::config::Root::builder()
                .appender("logfile")
                .build(log::LevelFilter::Info),
        )
        .unwrap();

    log4rs::init_config(config).unwrap();

    let airport_db_path = get_airport_db_path();
    if !airport_db_path.exists() {
        show_airport_database_warning(&airport_db_path, &app_data_dir);
        return;
    }

    if let Err(e) = run() {
        log::error!("Application error: {e}");
    }
}

fn run() -> Result<(), Error> {
    let database_pool = DatabasePool::new();
    let mut use_cli = false;

    for arg in std::env::args() {
        if arg == "--cli" {
            use_cli = true;
        }
    }

    database_pool
        .aircraft_pool
        .get()
        .unwrap()
        .run_pending_migrations(MIGRATIONS)
        .expect("Failed to run migrations");

    if use_cli {
        console_main(database_pool)?;
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
    Ok(())
}

fn console_main<T: DatabaseOperations>(mut database_connections: T) -> Result<(), Error> {
    let terminal = console::Term::stdout();
    terminal.clear_screen()?;

    loop {
        let not_flown_aircraft_count = database_connections.get_not_flown_count()?;

        println!(
            "\nWelcome to the flight planner\n\
             --------------------------------------------------\n\
             Number of not flown aircraft: {not_flown_aircraft_count}\n\
             What do you want to do?\n\
             1. Get random airport\n\
             2. Get random aircraft\n\
             3. Random aircraft from random airport\n\
             4. Random not flown aircraft, airport and destination\n\
             5. random aircraft and route\n\
             s, Random route for selected aircraft\n\
             l. List all aircraft\n\
             m. Mark aircraft as flown\n\
             h. History\n\
             q. Quit\n"
        );

        terminal.write_str("Enter your choice: ")?;
        let input = match terminal.read_char() {
            Ok(c) => c,
            Err(e) => {
                log::error!("Failed to read input: {e}");
                continue;
            }
        };
        terminal.clear_screen()?;

        match input {
            '1' => show_random_airport(&mut database_connections)?,
            '2' => show_random_not_flown_aircraft(&mut database_connections),
            '3' => show_random_aircraft_with_random_airport(&mut database_connections)?,
            '4' => show_random_not_flown_aircraft_and_route(&mut database_connections)?,
            '5' => show_random_aircraft_and_route(&mut database_connections)?,
            's' => show_random_route_for_selected_aircraft(&mut database_connections)?,
            'l' => show_all_aircraft(&mut database_connections)?,
            'm' => show_mark_all_not_flown(&mut database_connections)?,
            'h' => show_history(&mut database_connections)?,
            'q' => {
                log::info!("Quitting");
                return Ok(());
            }
            _ => {
                println!("Invalid input");
            }
        }
    }
}

fn show_mark_all_not_flown<T: DatabaseOperations>(
    database_connections: &mut T,
) -> Result<(), Error> {
    let terminal = console::Term::stdout();
    let ask_confirm = || -> std::io::Result<char> {
        terminal.write_str("Do you want to mark all aircraft as flown? (y/n)\n")?;
        terminal.read_char()
    };

    mark_all_not_flown(database_connections, ask_confirm)
}

fn mark_all_not_flown<T: AircraftOperations, F: Fn() -> Result<char, std::io::Error>>(
    database_connections: &mut T,
    confirm_fn: F,
) -> Result<(), Error> {
    match read_yn(confirm_fn) {
        Ok(true) => {
            database_connections.mark_all_aircraft_not_flown()?;
        }
        Ok(false) => {
            log::info!("Not marking all aircraft as flown");
        }
        Err(e) => {
            log::error!("Failed to read input: {e}");
        }
    }

    Ok(())
}

fn show_random_airport<T: AirportOperations>(database_connections: &mut T) -> Result<(), Error> {
    let airport = database_connections.get_random_airport()?;
    println!("{}", format_airport(&airport));

    let runways = database_connections.get_runways_for_airport(&airport)?;
    for runway in runways {
        println!("{}", format_runway(&runway));
    }

    Ok(())
}

fn show_random_not_flown_aircraft<T: AircraftOperations>(database_connections: &mut T) {
    match database_connections.random_not_flown_aircraft() {
        Ok(aircraft) => {
            println!("{}", format_aircraft(&aircraft));
        }
        Err(e) => {
            log::error!("Failed to get random not flown aircraft: {e}");
        }
    }
}

fn show_random_aircraft_with_random_airport<T: DatabaseOperations>(
    database_connections: &mut T,
) -> Result<(), Error> {
    let aircraft = database_connections.random_not_flown_aircraft()?;
    let airport = database_connections.get_random_airport_for_aircraft(&aircraft)?;

    println!("Aircraft: {}", format_aircraft(&aircraft));
    println!("Airport: {}", format_airport(&airport));

    for runway in database_connections.get_runways_for_airport(&airport)? {
        println!("{}", format_runway(&runway));
    }

    Ok(())
}

fn show_random_aircraft_and_route<T: DatabaseOperations>(
    database_connections: &mut T,
) -> Result<(), Error> {
    let aircraft = database_connections.random_aircraft()?;
    let departure = database_connections.get_random_airport_for_aircraft(&aircraft)?;
    let destination = database_connections.get_destination_airport(&aircraft, &departure)?;
    let distance = calculate_haversine_distance_nm(&departure, &destination);

    println!("Aircraft: {}", format_aircraft(&aircraft));
    println!("Departure: {}", format_airport(&departure));
    println!("Destination: {}", format_airport(&destination));
    println!("Distance: {distance:.2}nm");

    println!("\nDeparture runways:");
    for runway in database_connections.get_runways_for_airport(&departure)? {
        println!("{}", format_runway(&runway));
    }

    println!("\nDestination runways:");
    for runway in database_connections.get_runways_for_airport(&destination)? {
        println!("{}", format_runway(&runway));
    }

    Ok(())
}

fn show_all_aircraft<T: AircraftOperations>(database_connections: &mut T) -> Result<(), Error> {
    let all_aircraft = database_connections.get_all_aircraft()?;
    for aircraft in all_aircraft {
        println!("{}", format_aircraft(&aircraft));
    }

    Ok(())
}

fn show_random_not_flown_aircraft_and_route<T: DatabaseOperations>(
    database_connections: &mut T,
) -> Result<(), Error> {
    let ask_char_fn = || -> Result<char, std::io::Error> {
        let term = console::Term::stdout();
        term.write_str("Do you want to mark the aircraft as flown? (y/n)\n")
            .unwrap();
        match term.read_char() {
            Ok(c) => Ok(c),
            Err(e) => Err(e),
        }
    };

    random_not_flown_aircraft_and_route(database_connections, ask_char_fn)
}

fn random_not_flown_aircraft_and_route<
    T: DatabaseOperations,
    F: Fn() -> Result<char, std::io::Error>,
>(
    database_connections: &mut T,
    ask_char_fn: F,
) -> Result<(), Error> {
    let mut aircraft = database_connections.random_not_flown_aircraft()?;
    let departure = database_connections.get_random_airport_for_aircraft(&aircraft)?;
    let destination = database_connections.get_destination_airport(&aircraft, &departure)?;
    let distance = calculate_haversine_distance_nm(&departure, &destination);

    println!("Aircraft: {}", format_aircraft(&aircraft));
    println!("Departure: {}", format_airport(&departure));
    println!("Destination: {}", format_airport(&destination));
    println!("Distance: {distance:.2}nm");

    println!("\nDeparture runways:");
    for runway in database_connections.get_runways_for_airport(&departure)? {
        println!("{}", format_runway(&runway));
    }

    println!("\nDestination runways:");
    for runway in database_connections.get_runways_for_airport(&destination)? {
        println!("{}", format_runway(&runway));
    }

    ask_mark_flown(database_connections, &mut aircraft, ask_char_fn)?;
    database_connections.add_to_history(&departure, &destination, &aircraft)?;

    Ok(())
}

fn show_history<T: HistoryOperations + AircraftOperations>(
    database_connections: &mut T,
) -> Result<(), Error> {
    let history_data = database_connections.get_history()?;
    let aircraft_data = database_connections.get_all_aircraft()?;

    if history_data.is_empty() {
        println!("No history found");
        return Ok(());
    }

    for record in history_data {
        let Some(aircraft) = aircraft_data.iter().find(|a| a.id == record.aircraft) else {
            log::warn!("Aircraft not found for id: {}", record.aircraft);
            return Err(Error::Diesel(diesel::result::Error::NotFound));
        };

        println!(
            "Date: {}\nDeparture: {}\nDestination: {}\nAircraft: {} {} ({})\n",
            record.date,
            record.departure_icao,
            record.arrival_icao,
            aircraft.manufacturer,
            aircraft.variant,
            aircraft.icao_code
        );
    }

    Ok(())
}

fn show_random_route_for_selected_aircraft<T: DatabaseOperations>(
    database_connections: &mut T,
) -> Result<(), Error> {
    let terminal = console::Term::stdout();
    let ask_input_id = || -> Result<String, std::io::Error> {
        terminal.write_str("Enter aircraft id: ")?;
        terminal.read_line()
    };

    random_route_for_selected_aircraft(database_connections, ask_input_id)
}

fn random_route_for_selected_aircraft<
    T: DatabaseOperations,
    F: Fn() -> Result<String, std::io::Error>,
>(
    database_connections: &mut T,
    aircraft_id_fn: F,
) -> Result<(), Error> {
    let aircraft_id = match read_id(aircraft_id_fn) {
        Ok(id) => id,
        Err(e) => {
            log::warn!("Invalid id: {e}");
            return Ok(());
        }
    };

    let aircraft = database_connections.get_aircraft_by_id(aircraft_id)?;
    let departure = database_connections.get_random_airport_for_aircraft(&aircraft)?;
    let destination = database_connections.get_destination_airport(&aircraft, &departure)?;
    let distance = calculate_haversine_distance_nm(&departure, &destination);

    println!("Aircraft: {}", format_aircraft(&aircraft));
    println!("Departure: {}", format_airport(&departure));
    println!("Destination: {}", format_airport(&destination));
    println!("Distance: {distance:.2}nm");

    println!("\nDeparture runways:");
    for runway in database_connections.get_runways_for_airport(&departure)? {
        println!("{}", format_runway(&runway));
    }

    println!("\nDestination runways:");
    for runway in database_connections.get_runways_for_airport(&destination)? {
        println!("{}", format_runway(&runway));
    }

    Ok(())
}
