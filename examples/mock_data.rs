//! Mock data generator for benchmarks
//!
//! This module generates consistent, seeded mock airport data that statistically matches
//! a real airport database for accurate performance benchmarking.
//!
//! **Data Sources:**
//! - Aircraft: Loaded from actual `aircrafts.csv` file in the repository
//! - Airports: Generated based on statistical analysis of real airports
//! - Runways: Generated based on statistical analysis of real runways
//!
//! **Statistical Accuracy:**
//! All distributions are based on percentile analysis of the real database:
//! - Elevation distribution (percentile-based: P25, P50, P75, P90, P95)
//! - Runway count per airport (majority have 2 runways, matching real distribution)
//! - Runway length distribution (percentile-based from short to very long)
//! - Runway width distribution (percentile-based)
//! - Surface type distribution (ASP, GRE, WAT, U with exact percentages)
//! - Geographic distribution (global coverage produces realistic inter-airport distances)
//!
//! **Reproducibility:**
//! Fixed seed ensures deterministic output across runs for reliable benchmarking.
//!
//! **Regeneration:**
//! Run `analyze_airport_database.py` to regenerate statistics if the real database changes.

#![allow(dead_code)] // Module used by benchmark.rs

use flight_planner::models::{Aircraft, Airport, Runway};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use std::collections::HashMap;
use std::sync::Arc;

const SEED: u64 = 42; // Fixed seed for reproducibility
const AIRCRAFTS_CSV_PATH: &str = "aircrafts.csv";

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
///
/// Based on real database statistics:
/// - Elevation: Percentile-based distribution matching real data
/// - Geographic spread: Global coverage (full latitude/longitude range)
/// - Distance between airports: Natural distribution from uniform global spread
/// - Uses weighted percentile-based distribution for realistic characteristics
pub fn generate_mock_airports(count: usize) -> Vec<Arc<Airport>> {
    let mut rng = StdRng::seed_from_u64(SEED);
    let mut airports = Vec::with_capacity(count);
    
    for id in 0..count {
        // Generate simple airport identifiers
        // Names don't affect benchmark performance, so keep them simple
        let name = format!("Mock Airport {}", id + 1);
        
        // Generate ICAO code: 4 random letters for global coverage
        let icao = format!("{}{}{}{}", 
            (b'A' + rng.random_range(0..26)) as char,
            (b'A' + rng.random_range(0..26)) as char,
            (b'A' + rng.random_range(0..26)) as char,
            (b'A' + rng.random_range(0..26)) as char
        );
        
        // Global distribution matching real database coverage
        let latitude = -90.0 + rng.random::<f64>() * 173.0; // Full global range
        let longitude = -180.0 + rng.random::<f64>() * 360.0; // Full global range
        
        // Realistic elevation distribution based on percentile analysis
        let elevation_rand = rng.random::<f64>();
        let elevation = if elevation_rand < 0.25 {
            rng.random_range(-210..179)  // P0-P25: Sea level and below
        } else if elevation_rand < 0.50 {
            rng.random_range(179..710)  // P25-P50: Low elevation
        } else if elevation_rand < 0.75 {
            rng.random_range(710..1489)  // P50-P75: Medium elevation
        } else if elevation_rand < 0.90 {
            rng.random_range(1489..3018)  // P75-P90: High elevation
        } else if elevation_rand < 0.95 {
            rng.random_range(3018..4360)  // P90-P95: Very high elevation
        } else {
            rng.random_range(4360..14422)  // P95-P100: Extreme elevation
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
///
/// Based on real database statistics:
/// - Runway count distribution: Percentile-based (majority have 2 runways)
/// - Length: Percentile-based distribution from short to very long
/// - Width: Percentile-based distribution
/// - Surface types: Accurate distribution of ASP, GRE, WAT, U
pub fn generate_mock_runways(airports: &[Arc<Airport>]) -> HashMap<i32, Arc<Vec<Runway>>> {
    let mut rng = StdRng::seed_from_u64(SEED + 1);
    let mut runways_map = HashMap::new();

    for airport in airports {
        // Realistic runway count distribution based on actual database percentiles
        let runway_rand = rng.random::<f64>();
        let num_runways = if runway_rand < 0.7980 {
            2
        } else if runway_rand < 0.9662 {
            4
        } else if runway_rand < 0.9929 {
            6
        } else if runway_rand < 0.9979 {
            8
        } else if runway_rand < 0.9988 {
            10
        } else {
            12  // Includes rare 12, 14, 16 runway airports
        };
        
        let mut runways = Vec::new();

        for runway_idx in 0..num_runways {
            // Runways are typically oriented based on prevailing winds
            // For simulation, we'll use evenly distributed headings
            let base_heading = (runway_idx as f64 * 360.0 / num_runways as f64) as i32;
            let heading = base_heading % 360;
            let runway_number = ((heading + 5) / 10) % 36; // Round to nearest 10 degrees
            
            // Determine parallel runway suffix if multiple runways exist
            let suffix = if num_runways > 2 {
                match runway_idx % 3 {
                    0 => "L",
                    1 => "C",
                    _ => "R",
                }
            } else if num_runways == 2 {
                if runway_idx == 0 { "L" } else { "R" }
            } else {
                ""
            };

            let ident = format!("{:02}{}", runway_number, suffix);
            
            // Realistic runway length distribution based on percentile analysis
            let length_rand = rng.random::<f64>();
            let base_length = if length_rand < 0.25 {
                rng.random_range(80..2700)  // P0-P25
            } else if length_rand < 0.50 {
                rng.random_range(2700..3937)  // P25-P50
            } else if length_rand < 0.75 {
                rng.random_range(3937..5906)  // P50-P75
            } else if length_rand < 0.90 {
                rng.random_range(5906..8999)  // P75-P90
            } else {
                rng.random_range(8999..21119)  // P90-P100
            };
            
            // High altitude airports need extra runway length (thinner air)
            let length = if airport.Elevation > 3000 {
                (base_length as f64 * 1.15) as i32 // 15% longer at high altitude
            } else {
                base_length
            };

            // Realistic runway width distribution based on percentile analysis
            let width_rand = rng.random::<f64>();
            let width = if width_rand < 0.25 {
                rng.random_range(9..70)  // P0-P25
            } else if width_rand < 0.50 {
                rng.random_range(70..98)  // P25-P50
            } else if width_rand < 0.75 {
                rng.random_range(98..148)  // P50-P75
            } else if width_rand < 0.90 {
                rng.random_range(148..150)  // P75-P90
            } else {
                rng.random_range(150..300)  // P90-P100 (capped for realism)
            };
            
            // Surface type distribution based on database percentages
            let surface_rand = rng.random::<f64>();
            let surface = if surface_rand < 0.7000 {
                "ASP"
            } else if surface_rand < 0.9940 {
                "GRE"
            } else if surface_rand < 0.9997 {
                "WAT"
            } else {
                "U"
            };

            runways.push(Runway {
                ID: (airport.ID * 10 + runway_idx),
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

// This file is a module used by benchmark.rs, not a standalone example
// The main function is required by Cargo but not used
fn main() {}
