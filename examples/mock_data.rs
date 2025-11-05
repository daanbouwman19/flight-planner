/// Mock data generator for benchmarks
///
/// This module generates consistent, seeded mock airport data that resembles a real airport database.
/// Aircraft data is loaded from the actual aircrafts.csv file in the repository.
/// The mock data is deterministic across runs for reliable benchmarking.

use flight_planner::models::{Aircraft, Airport, Runway};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use std::collections::HashMap;
use std::sync::Arc;

const SEED: u64 = 42; // Fixed seed for reproducibility
const AIRCRAFTS_CSV_PATH: &str = "aircrafts.csv";

// Realistic US airport names and locations
// Data represents a mix of major hubs, regional, and small airports
const AIRPORT_DATA: &[(&str, &str, f64, f64, i32)] = &[
    // Major hubs
    ("Hartsfield-Jackson Atlanta International Airport", "KATL", 33.6407, -84.4277, 1026),
    ("Los Angeles International Airport", "KLAX", 33.9416, -118.4085, 125),
    ("O'Hare International Airport", "KORD", 41.9742, -87.9073, 668),
    ("Dallas/Fort Worth International Airport", "KDFW", 32.8998, -97.0403, 607),
    ("Denver International Airport", "KDEN", 39.8561, -104.6737, 5434),
    ("John F. Kennedy International Airport", "KJFK", 40.6413, -73.7781, 13),
    ("San Francisco International Airport", "KSFO", 37.6213, -122.3790, 13),
    ("Seattle-Tacoma International Airport", "KSEA", 47.4502, -122.3088, 433),
    ("McCarran International Airport", "KLAS", 36.0840, -115.1537, 2181),
    ("Phoenix Sky Harbor International Airport", "KPHX", 33.4352, -112.0101, 1135),
    
    // Regional airports
    ("Austin-Bergstrom International Airport", "KAUS", 30.1945, -97.6699, 542),
    ("Nashville International Airport", "KBNA", 36.1263, -86.6769, 599),
    ("Portland International Airport", "KPDX", 45.5898, -122.5951, 31),
    ("Sacramento International Airport", "KSMF", 38.6954, -121.5908, 27),
    ("San Diego International Airport", "KSAN", 32.7336, -117.1897, 17),
    ("Salt Lake City International Airport", "KSLC", 40.7899, -111.9791, 4227),
    ("Tampa International Airport", "KTPA", 27.9755, -82.5332, 26),
    ("Raleigh-Durham International Airport", "KRDU", 35.8776, -78.7875, 435),
    ("Louis Armstrong New Orleans International Airport", "KMSY", 29.9934, -90.2580, 4),
    ("Indianapolis International Airport", "KIND", 39.7173, -86.2944, 797),
];

/// Load aircraft from the repository's aircrafts.csv file
pub fn load_aircraft_from_csv() -> Result<Vec<Arc<Aircraft>>, Box<dyn std::error::Error>> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    
    let file = File::open(AIRCRAFTS_CSV_PATH)?;
    let reader = BufReader::new(file);
    let mut aircraft = Vec::new();
    
    for (id, line) in reader.lines().enumerate() {
        let line = line?;
        if id == 0 {
            // Skip header
            continue;
        }
        
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() < 9 {
            continue; // Skip malformed lines
        }
        
        let manufacturer = fields[0].to_string();
        let variant = fields[1].to_string();
        let icao_code = fields[2].to_string();
        let flown: i32 = fields[3].parse().unwrap_or(0);
        let aircraft_range: i32 = fields[4].parse().unwrap_or(0);
        let category = fields[5].to_string();
        let cruise_speed: i32 = fields[6].parse().unwrap_or(0);
        let date_flown = if fields[7].is_empty() {
            None
        } else {
            Some(fields[7].to_string())
        };
        let takeoff_distance: Option<i32> = fields[8].parse().ok();
        
        aircraft.push(Arc::new(Aircraft {
            id: id as i32,
            manufacturer,
            variant,
            icao_code,
            flown,
            aircraft_range,
            category,
            cruise_speed,
            date_flown,
            takeoff_distance,
        }));
    }
    
    Ok(aircraft)
}

/// Generate a set of mock airports with consistent seeded randomness
/// The mock data is based on real US airport characteristics
pub fn generate_mock_airports(count: usize) -> Vec<Arc<Airport>> {
    let mut rng = StdRng::seed_from_u64(SEED);
    let mut airports = Vec::with_capacity(count);
    
    // First, add the known airports
    for (idx, (name, icao, lat, lon, elevation)) in AIRPORT_DATA.iter().enumerate() {
        airports.push(Arc::new(Airport {
            ID: idx as i32,
            Name: name.to_string(),
            ICAO: icao.to_string(),
            PrimaryID: Some(idx as i32),
            Latitude: *lat,
            Longtitude: *lon,
            Elevation: *elevation,
            TransitionAltitude: Some(18000),
            TransitionLevel: Some(180),
            SpeedLimit: Some(250),
            SpeedLimitAltitude: Some(10000),
        }));
    }
    
    // Generate additional synthetic airports to reach the desired count
    let state_codes = ["AL", "AK", "AZ", "AR", "CA", "CO", "CT", "DE", "FL", "GA", 
                       "HI", "ID", "IL", "IN", "IA", "KS", "KY", "LA", "ME", "MD",
                       "MA", "MI", "MN", "MS", "MO", "MT", "NE", "NV", "NH", "NJ",
                       "NM", "NY", "NC", "ND", "OH", "OK", "OR", "PA", "RI", "SC",
                       "SD", "TN", "TX", "UT", "VT", "VA", "WA", "WV", "WI", "WY"];
    
    let airport_types = [
        "Regional Airport",
        "Municipal Airport", 
        "County Airport",
        "International Airport",
        "Executive Airport",
        "Memorial Airport",
    ];
    
    for id in AIRPORT_DATA.len()..count {
        let state = state_codes[rng.random_range(0..state_codes.len())];
        let airport_type = airport_types[rng.random_range(0..airport_types.len())];
        let city_number = rng.random_range(1..999);
        
        let name = format!("{} City {} {}", state, city_number, airport_type);
        
        // Generate ICAO code: K + 3 letters
        let icao = format!("K{}{}{}", 
            (b'A' + rng.random_range(0..26)) as char,
            (b'A' + rng.random_range(0..26)) as char,
            (b'A' + rng.random_range(0..26)) as char
        );
        
        // Distribute airports across US territory
        let latitude = 25.0 + rng.random::<f64>() * 24.0; // 25°N to 49°N
        let longitude = -125.0 + rng.random::<f64>() * 58.0; // -125°W to -67°W
        
        // Most airports are at lower elevations, some are high altitude
        let elevation = if rng.random::<f64>() < 0.15 {
            rng.random_range(3000..8000) // High altitude airports (15%)
        } else {
            rng.random_range(0..2000) // Lower elevation airports (85%)
        };
        
        airports.push(Arc::new(Airport {
            ID: id as i32,
            Name: name,
            ICAO: icao,
            PrimaryID: Some(id as i32),
            Latitude: latitude,
            Longtitude: longitude,
            Elevation: elevation,
            TransitionAltitude: Some(18000),
            TransitionLevel: Some(180),
            SpeedLimit: Some(250),
            SpeedLimitAltitude: Some(10000),
        }));
    }
    
    airports
}

/// Generate a set of mock runways for the given airports
/// Runway characteristics are based on typical US airport patterns
pub fn generate_mock_runways(airports: &[Arc<Airport>]) -> HashMap<i32, Arc<Vec<Runway>>> {
    let mut rng = StdRng::seed_from_u64(SEED + 1);
    let mut runways_map = HashMap::new();

    for airport in airports {
        // Larger airports and those at lower elevations typically have more/longer runways
        let num_runways = if airport.Elevation < 1000 {
            rng.random_range(1..=4) // Major airports: 1-4 runways
        } else {
            rng.random_range(1..=2) // High altitude/smaller: 1-2 runways
        };
        
        let mut runways = Vec::new();

        for runway_idx in 0..num_runways {
            // Runways are typically oriented based on prevailing winds
            // For simulation, we'll use evenly distributed headings
            let base_heading = (runway_idx as f64 * 360.0 / num_runways as f64) as i32;
            let heading = base_heading % 360;
            let runway_number = ((heading + 5) / 10) % 36; // Round to nearest 10 degrees
            
            // Determine parallel runway suffix if multiple runways exist
            let suffix = if num_runways > 1 {
                match runway_idx % 3 {
                    0 => "L",
                    1 => "C",
                    _ => "R",
                }
            } else {
                ""
            };

            let ident = format!("{:02}{}", runway_number, suffix);
            
            // Runway lengths vary based on airport type and elevation
            // Major airports: 7,000-12,000 ft
            // Regional: 5,000-8,000 ft  
            // Small: 3,000-5,000 ft
            // High altitude needs longer runways
            let base_length = if airport.Elevation < 500 {
                rng.random_range(7000..12000) // Sea level major airports
            } else if airport.Elevation < 2000 {
                rng.random_range(5000..9000) // Regional airports
            } else {
                rng.random_range(4000..7000) // High altitude airports
            };
            
            // High altitude airports need extra runway length
            let length = if airport.Elevation > 3000 {
                (base_length as f64 * 1.15) as i32 // 15% longer at high altitude
            } else {
                base_length
            };

            let width = if length > 8000 {
                rng.random_range(150..200) // Wide runways for large airports
            } else if length > 5000 {
                rng.random_range(100..150) // Medium width
            } else {
                rng.random_range(75..100) // Narrower for small airports
            };
            
            // Surface types: Most are asphalt or concrete
            let surface = if rng.random::<f64>() < 0.85 {
                "ASPH" // 85% asphalt
            } else {
                "CONC" // 15% concrete
            };

            runways.push(Runway {
                ID: (airport.ID * 10 + runway_idx as i32),
                AirportID: airport.ID,
                Ident: ident,
                TrueHeading: heading as f64,
                Length: length,
                Width: width,
                Surface: surface.to_string(),
                Latitude: airport.Latitude,
                Longtitude: airport.Longtitude,
                Elevation: airport.Elevation,
            });
        }

        runways_map.insert(airport.ID, Arc::new(runways));
    }

    runways_map
}

/// Generate an R-tree for spatial airport queries
pub fn generate_spatial_rtree(
    airports: &[Arc<Airport>],
) -> rstar::RTree<flight_planner::models::airport::SpatialAirport> {
    use flight_planner::models::airport::SpatialAirport;
    
    let spatial_airports: Vec<SpatialAirport> = airports
        .iter()
        .map(|airport| SpatialAirport {
            airport: Arc::clone(airport),
        })
        .collect();

    rstar::RTree::bulk_load(spatial_airports)
}
