#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
    clippy::complexity,
    clippy::perf
)]

use diesel::prelude::*;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use eframe::egui_wgpu;
use eframe::egui_wgpu::WgpuSetupCreateNew;
use eframe::wgpu;
use gui::ui::Gui;
use log4rs::append::file::FileAppender;
use log4rs::encode::pattern::PatternEncoder;
use std::path::PathBuf; // Path removed
use std::fs;
use std::sync::{Arc, LazyLock};
use util::calculate_haversine_distance_nm;
use directories::ProjectDirs;

use crate::database::DatabasePool; // AIRPORT_DB_FILENAME removed
use crate::errors::Error;
use crate::models::Aircraft;
use eframe::AppCreator;
use egui::ViewportBuilder;
use errors::ValidationError;
use modules::aircraft::format_aircraft;
use modules::airport::format_airport;
use modules::runway::format_runway;
use traits::{AircraftOperations, AirportOperations, DatabaseOperations, HistoryOperations};

mod database;
mod errors;
mod gui;
mod models;
mod modules;
mod schema;
mod traits;
pub mod util;

define_sql_function! {fn random() -> Text }

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

// Define PROJECT_DIRS using LazyLock, similar to database.rs
static PROJECT_DIRS: LazyLock<Option<ProjectDirs>> = LazyLock::new(|| {
    ProjectDirs::from("com.github.flightplanner.FlightPlanner", "FlightPlanner",  "FlightPlannerApp")
});

// Function to get the application specific data directory, similar to database.rs
fn get_app_data_dir() -> PathBuf {
    let base_dirs = PROJECT_DIRS.as_ref().expect("Could not get project directories");
    let data_dir = base_dirs.data_dir().join("flight-planner");
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir).expect("Failed to create app data directory");
    }
    data_dir
}

fn main() {
    // Setup logging
    let log_dir = get_app_data_dir().join("log");
    if !log_dir.exists() {
        fs::create_dir_all(&log_dir).expect("Failed to create log directory");
    }
    let log_file_path = log_dir.join("output.log");

    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
        .build(log_file_path) // Use the new path here
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

    // Use the new function from database.rs for the path
    let airport_db_actual_path = crate::database::airport_db_path();
    if !airport_db_actual_path.exists() {
        let error_message = format!(
            "Error: Airports database not found.
            Expected at: {}
            Please ensure 'airports.db3' is present in this location.
            You might need to download it or place it here if you have it separately.",
            airport_db_actual_path.display()
        );
        eprintln!("{error_message}"); // Print to stderr for errors
        log::error!("Airports database not found at {}. User notified.", airport_db_actual_path.display());
        // Exiting the program. In a more robust setup, this might return a specific error.
        return;
    }

    if let Err(e) = run() {
        log::error!("Application error: {e}");
    }
}

fn run() -> Result<(), Error> {
    let mut database_pool = DatabasePool::new();
    let mut use_gui = true; // Default to GUI

    // Check for --tui or --console flags
    for arg in std::env::args().skip(1) { // Skip the program name
        if arg == "--tui" || arg == "--console" {
            use_gui = false;
            break; // Flag found, no need to check further
        }
    }

    database_pool
        .aircraft_pool
        .get()
        .unwrap()
        .run_pending_migrations(MIGRATIONS)
        .expect("Failed to run migrations");

    if use_gui {
        let icon = include_bytes!("../icon.png");
        let image = image::load_from_memory(icon)
            .expect("Failed to load icon")
            .to_rgba8();
        let (icon_width, icon_height) = image.dimensions();

        let native_options = eframe::NativeOptions {
            viewport: ViewportBuilder {
                inner_size: Some(egui::vec2(1200.0, 768.0)),
                close_button: Some(true),
                icon: Some(Arc::from(egui::IconData {
                    rgba: image.into_raw(),
                    width: icon_width,
                    height: icon_height,
                })),
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
                        }
                    }),
                    trace_path: None,
                }),
                present_mode: wgpu::PresentMode::AutoVsync,
                desired_maximum_frame_latency: Some(2),
                on_surface_error: Arc::new(|_| egui_wgpu::SurfaceErrorAction::SkipFrame),
            },
            ..Default::default()
        };

        let app_creator: AppCreator<'_> =
            Box::new(|cc| Ok(Box::new(Gui::new(cc, &mut database_pool))));
        _ = eframe::run_native("Flight planner", native_options, app_creator);
    } else {
        console_main(database_pool)?;
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

fn ask_mark_flown<T: AircraftOperations, F: Fn() -> Result<char, std::io::Error>>(
    database_connections: &mut T,
    aircraft: &mut Aircraft,
    ask_char_fn: F,
) -> Result<(), Error> {
    if matches!(ask_char_fn(), Ok('y')) {
        aircraft.date_flown = Some(chrono::Local::now().format("%Y-%m-%d").to_string());
        aircraft.flown = 1;
        database_connections.update_aircraft(aircraft)?;
    }

    Ok(())
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

fn read_yn<F: Fn() -> Result<char, std::io::Error>>(
    read_input: F,
) -> Result<bool, ValidationError> {
    let input = read_input().map_err(|e| ValidationError::InvalidData(e.to_string()))?;
    match input {
        'y' => Ok(true),
        'n' => Ok(false),
        _ => Err(ValidationError::InvalidData("Invalid input".to_string())),
    }
}

fn read_id<F: Fn() -> Result<String, std::io::Error>>(
    read_input: F,
) -> Result<i32, ValidationError> {
    let input = read_input().map_err(|e| ValidationError::InvalidData(e.to_string()))?;
    let id = match input.trim().parse::<i32>() {
        Ok(id) => id,
        Err(e) => {
            log::error!("Failed to parse id: {e}");
            return Err(ValidationError::InvalidData("Invalid id".to_string()));
        }
    };

    if id < 1 {
        return Err(ValidationError::InvalidId(id));
    }

    Ok(id)
}

#[cfg(test)]
mod tests {
    use crate::models::NewAircraft;

    use super::*;

    #[test]
    fn test_read_id() {
        // Test with valid input
        let input = "123\n";
        let result = read_id(|| -> Result<String, std::io::Error> { Ok(input.to_string()) });
        assert_eq!(result, Ok(123));

        // Test with invalid input (not a number)
        let input = "abc\n";
        let result = read_id(|| -> Result<String, std::io::Error> { Ok(input.to_string()) });
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid data: Invalid id".to_string()
        );

        // Test with invalid input (negative number)
        let input = "-5\n";
        let result = read_id(|| -> Result<String, std::io::Error> { Ok(input.to_string()) });
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid ID: -5".to_string()
        );
    }

    #[test]
    fn test_read_yn() {
        // Test with valid input "y"
        let result = read_yn(|| -> Result<char, std::io::Error> { Ok('y') });
        assert_eq!(result, Ok(true));

        // Test with valid input "n"
        let result = read_yn(|| -> Result<char, std::io::Error> { Ok('n') });
        assert_eq!(result, Ok(false));

        // Test with invalid input
        let result = read_yn(|| -> Result<char, std::io::Error> { Ok('x') });
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid data: Invalid input".to_string()
        );
    }

    #[test]
    fn test_ask_mark_flown() {
        use modules::aircraft::tests::setup_test_db;
        let mut db = setup_test_db();
        let aircraft = NewAircraft {
            manufacturer: "Boeing".to_string(),
            variant: "747-400".to_string(),
            icao_code: "B744".to_string(),
            flown: 0,
            date_flown: None,
            aircraft_range: 7260,
            category: "Heavy".to_string(),
            cruise_speed: 490,
            takeoff_distance: Some(9000),
        };
        db.add_aircraft(&aircraft).unwrap();
        let mut aircraft = db.get_all_aircraft().unwrap().pop().unwrap();

        // Test with user confirming
        let result: Result<(), Error> = ask_mark_flown(&mut db, &mut aircraft, || Ok('y'));
        assert!(result.is_ok());
        assert_eq!(aircraft.flown, 1);
        assert!(aircraft.date_flown.is_some());

        // Reset the aircraft
        aircraft.flown = 0;
        aircraft.date_flown = None;

        // Test with user declining
        let result = ask_mark_flown(&mut db, &mut aircraft, || Ok('n'));
        assert!(result.is_ok());
        assert_eq!(aircraft.flown, 0);
        assert!(aircraft.date_flown.is_none());
    }
}
