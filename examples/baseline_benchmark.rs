// Baseline benchmark to measure original route generation performance
// This will use the old RouteGenerator structure without optimizations

use flight_planner::database::DatabasePool;
use flight_planner::gui::services::AppService;
use std::time::Instant;

fn main() {
    println!("Baseline Route Generation Performance (Original Code)");
    println!("====================================================");
    
    // Create database connection and AppService once
    if let Ok(db_pool) = DatabasePool::new(None, None) {
        if let Ok(service) = AppService::new(db_pool) {
            println!("Service created. Testing route generation performance...\n");
            
            // Warm up
            let _ = service.route_generator().generate_random_routes(service.aircraft(), None);
            
            // Test random routes multiple times for accurate baseline
            let mut total_time = std::time::Duration::ZERO;
            let iterations = 10;
            
            for i in 0..iterations {
                let start = Instant::now();
                let routes = service.route_generator().generate_random_routes(service.aircraft(), None);
                let duration = start.elapsed();
                total_time += duration;
                
                println!("Iteration {}: {} RandomRoutes in {:?}", i + 1, routes.len(), duration);
            }
            
            let avg_time = total_time / iterations;
            println!("\nAverage RandomRoutes generation time: {:?}", avg_time);
            println!("Average per route: {:?}", avg_time / 50);
            
            // Test not flown routes
            total_time = std::time::Duration::ZERO;
            
            for i in 0..iterations {
                let start = Instant::now();
                let routes = service.route_generator().generate_random_not_flown_aircraft_routes(service.aircraft(), None);
                let duration = start.elapsed();
                total_time += duration;
                
                println!("Iteration {}: {} NotFlownRoutes in {:?}", i + 1, routes.len(), duration);
            }
            
            let avg_time = total_time / iterations;
            println!("\nAverage NotFlownRoutes generation time: {:?}", avg_time);
            println!("Average per route: {:?}", avg_time / 50);
            
        } else {
            println!("Failed to create AppService");
        }
    } else {
        println!("Failed to create database pool");
    }
}