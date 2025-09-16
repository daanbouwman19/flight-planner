// Comprehensive benchmark for flight planner performance
// 
// This benchmark tests:
// - Search performance with different query types and dataset sizes
// - Route generation performance with timing measurements
//
// Run with: cargo run --release --example benchmark

use flight_planner::database::DatabasePool;
use flight_planner::gui::data::{ListItemAirport, TableItem};
use flight_planner::gui::services::{AppService, SearchService};
use std::sync::Arc;
use std::time::Instant;

fn create_test_dataset(size: usize) -> Vec<Arc<TableItem>> {
    (0..size)
        .map(|i| {
            Arc::new(TableItem::Airport(ListItemAirport::new(
                format!("Airport {}", i),
                format!("A{:03}", i),
                format!("{}00ft", 10 + i % 50),
            )))
        })
        .collect()
}

fn benchmark_search_performance() {
    println!("🔍 SEARCH PERFORMANCE");
    println!("====================");
    
    let dataset = create_test_dataset(10000);
    
    let test_cases = [
        ("Single char", "A"),
        ("Short query", "Airport"),
        ("Specific match", "A001"),
        ("No results", "ZZZZZ"),
    ];
    
    for (description, query) in test_cases {
        let start = Instant::now();
        let results = SearchService::filter_items_static(&dataset, query);
        let duration = start.elapsed();
        
        println!("  {}: {} results in {:?}", description, results.len(), duration);
    }
}

fn benchmark_route_generation() {
    println!("\n⚡ ROUTE GENERATION PERFORMANCE");
    println!("===============================");
    
    if let Ok(db_pool) = DatabasePool::new(None, None) {
        if let Ok(service) = AppService::new(db_pool) {
            println!("  Using real database with {} airports and {} aircraft", 
                     service.airports().len(), 
                     service.aircraft().len());
            
            let iterations = 1000;
            println!("  Running {} iterations for statistical accuracy...", iterations);
            
            // Warm up
            let _ = service
                .route_generator()
                .generate_random_routes(service.aircraft(), None);
            
            // Test Random Routes
            println!("  \n🎲 Testing Random Routes:");
            let mut total_time = std::time::Duration::ZERO;
            let mut route_counts = Vec::new();
            let mut times = Vec::new();
            
            // Progress indicator
            let progress_interval = iterations / 10;
            
            for i in 0..iterations {
                let start = Instant::now();
                let routes = service
                    .route_generator()
                    .generate_random_routes(service.aircraft(), None);
                let duration = start.elapsed();
                
                total_time += duration;
                route_counts.push(routes.len());
                times.push(duration);
                
                // Show progress every 10%
                if (i + 1) % progress_interval == 0 {
                    let progress = ((i + 1) * 100) / iterations;
                    println!("    Progress: {}% ({}/{})", progress, i + 1, iterations);
                }
            }
            
            // Calculate statistics for Random Routes
            let avg_time = total_time / iterations;
            let avg_routes = route_counts.iter().sum::<usize>() as f64 / iterations as f64;
            
            // Calculate standard deviation for timing
            let mean_nanos = avg_time.as_nanos() as f64;
            let variance = times.iter()
                .map(|&t| {
                    let diff = t.as_nanos() as f64 - mean_nanos;
                    diff * diff
                })
                .sum::<f64>() / iterations as f64;
            let std_dev = std::time::Duration::from_nanos(variance.sqrt() as u64);
            
            // Find min/max times
            let min_time = times.iter().min().unwrap();
            let max_time = times.iter().max().unwrap();
            
            println!("    📊 Random Routes Results:");
            println!("      Average routes: {:.1}", avg_routes);
            println!("      Average time: {:?} (±{:?})", avg_time, std_dev);
            println!("      Range: {:?} - {:?}", min_time, max_time);
            println!("      Per route: {:?}", avg_time / flight_planner::modules::routes::GENERATE_AMOUNT as u32);
            
            // Test Not Flown Routes
            println!("  \n✈️ Testing Not Flown Routes:");
            let mut total_time_nf = std::time::Duration::ZERO;
            let mut route_counts_nf = Vec::new();
            let mut times_nf = Vec::new();
            
            for i in 0..iterations {
                let start = Instant::now();
                let routes = service
                    .route_generator()
                    .generate_random_not_flown_aircraft_routes(service.aircraft(), None);
                let duration = start.elapsed();
                
                total_time_nf += duration;
                route_counts_nf.push(routes.len());
                times_nf.push(duration);
                
                // Show progress every 10%
                if (i + 1) % progress_interval == 0 {
                    let progress = ((i + 1) * 100) / iterations;
                    println!("    Progress: {}% ({}/{})", progress, i + 1, iterations);
                }
            }
            
            // Calculate statistics for Not Flown Routes
            let avg_time_nf = total_time_nf / iterations;
            let avg_routes_nf = route_counts_nf.iter().sum::<usize>() as f64 / iterations as f64;
            
            let mean_nanos_nf = avg_time_nf.as_nanos() as f64;
            let variance_nf = times_nf.iter()
                .map(|&t| {
                    let diff = t.as_nanos() as f64 - mean_nanos_nf;
                    diff * diff
                })
                .sum::<f64>() / iterations as f64;
            let std_dev_nf = std::time::Duration::from_nanos(variance_nf.sqrt() as u64);
            
            let min_time_nf = times_nf.iter().min().unwrap();
            let max_time_nf = times_nf.iter().max().unwrap();
            
            println!("    📊 Not Flown Routes Results:");
            println!("      Average routes: {:.1}", avg_routes_nf);
            println!("      Average time: {:?} (±{:?})", avg_time_nf, std_dev_nf);
            println!("      Range: {:?} - {:?}", min_time_nf, max_time_nf);
            println!("      Per route: {:?}", avg_time_nf / flight_planner::modules::routes::GENERATE_AMOUNT as u32);
            
            println!("\n  🏁 SUMMARY:");
            println!("    Total iterations: {} x 2 = {}", iterations, iterations * 2);
            println!("    Total test time: {:?}", total_time + total_time_nf);
            println!("    Performance difference: {:.1}%", 
                     ((avg_time_nf.as_nanos() as f64 / avg_time.as_nanos() as f64) - 1.0) * 100.0);
            
        } else {
            println!("  ❌ Failed to create AppService");
        }
    } else {
        println!("  ❌ Failed to create database connection");
    }
}

fn main() {
    println!("Flight Planner Performance Benchmark");
    println!("====================================");
    
    benchmark_search_performance();
    benchmark_route_generation();
    
    println!("\n✅ Benchmark Complete!");
}