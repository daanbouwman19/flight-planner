// Performance benchmark for search optimization and route generation

use flight_planner::database::DatabasePool;
use flight_planner::gui::data::{ListItemAirport, ListItemRoute, TableItem};
use flight_planner::gui::services::popup_service::DisplayMode;
use flight_planner::gui::services::{AppService, SearchService};
use flight_planner::models::{Aircraft, Airport};
use std::sync::Arc;
use std::time::Instant;

fn create_large_dataset(size: usize) -> Vec<Arc<TableItem>> {
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

fn create_route_dataset(size: usize) -> Vec<Arc<TableItem>> {
    (0..size)
        .map(|i| {
            let departure = Airport {
                ID: i as i32,
                Name: format!("Departure Airport {}", i),
                ICAO: format!("DEP{}", i % 1000),
                PrimaryID: None,
                Latitude: 51.0 + (i as f64 * 0.01),
                Longtitude: 0.0 + (i as f64 * 0.01),
                Elevation: 100,
                TransitionAltitude: None,
                TransitionLevel: None,
                SpeedLimit: None,
                SpeedLimitAltitude: None,
            };

            let destination = Airport {
                ID: (i + 1000) as i32,
                Name: format!("Destination Airport {}", i),
                ICAO: format!("DST{}", i % 1000),
                PrimaryID: None,
                Latitude: 52.0 + (i as f64 * 0.01),
                Longtitude: 1.0 + (i as f64 * 0.01),
                Elevation: 200,
                TransitionAltitude: None,
                TransitionLevel: None,
                SpeedLimit: None,
                SpeedLimitAltitude: None,
            };

            let aircraft = Aircraft {
                id: i as i32,
                manufacturer: format!("Manufacturer {}", i % 10),
                variant: format!("Variant {}", i % 20),
                icao_code: format!("AC{}", i % 100),
                flown: 0,
                aircraft_range: 1000 + (i % 500) as i32,
                category: "Jet".to_string(),
                cruise_speed: 400 + (i % 200) as i32,
                date_flown: None,
                takeoff_distance: Some(2000 + (i % 1000) as i32),
            };

            let route = ListItemRoute {
                departure: Arc::new(departure),
                destination: Arc::new(destination),
                aircraft: Arc::new(aircraft),
                departure_runway_length: format!("{}ft", 3000 + i % 2000),
                destination_runway_length: format!("{}ft", 3000 + i % 2000),
                route_length: format!("{} nm", 100 + i % 1000),
            };

            Arc::new(TableItem::Route(route))
        })
        .collect()
}

fn benchmark_search(items: &[Arc<TableItem>], query: &str, description: &str) {
    let start = Instant::now();
    let results = SearchService::filter_items_static(items, query);
    let duration = start.elapsed();

    println!(
        "{}: {} results in {:?}",
        description,
        results.len(),
        duration
    );
}

fn benchmark_route_generation(
    app_service: &AppService,
    mode: DisplayMode,
    _count: usize,
) {
    let start = Instant::now();

    // Generate routes using the route generator directly
    let routes = match mode {
        DisplayMode::RandomRoutes => app_service
            .route_generator()
            .generate_random_routes(app_service.aircraft(), None),
        DisplayMode::NotFlownRoutes => app_service
            .route_generator()
            .generate_random_not_flown_aircraft_routes(app_service.aircraft(), None),
        _ => Vec::new(),
    };

    let duration = start.elapsed();
    println!(
        "Generated {} {:?} routes in {:?}",
        routes.len(),
        mode,
        duration
    );
}

fn main() {
    println!("Search Performance Benchmark");
    println!("============================");

    // Test with different dataset sizes
    let small_dataset = create_large_dataset(100);
    let medium_dataset = create_large_dataset(1000);
    let large_dataset = create_large_dataset(10000);

    println!("\n🔍 SEARCH BENCHMARKS - AIRPORT DATA");
    println!("===================================");

    println!("\nSmall dataset (100 items):");
    benchmark_search(&small_dataset, "Airport", "Short query");
    benchmark_search(&small_dataset, "A001", "ICAO search");
    benchmark_search(&small_dataset, "1200ft", "Runway search");

    println!("\nMedium dataset (1000 items):");
    benchmark_search(&medium_dataset, "Airport", "Short query");
    benchmark_search(&medium_dataset, "A001", "ICAO search");
    benchmark_search(&medium_dataset, "1200ft", "Runway search");

    println!("\nLarge dataset (10000 items):");
    benchmark_search(&large_dataset, "Airport", "Short query");
    benchmark_search(&large_dataset, "A001", "ICAO search");
    benchmark_search(&large_dataset, "1200ft", "Runway search");

    println!("\nSingle character search (should be instant):");
    benchmark_search(&large_dataset, "A", "Single char");

    println!("\nEmpty query (should return all):");
    benchmark_search(&large_dataset, "", "Empty query");

    // Test route data search performance
    println!("\n🛫 SEARCH BENCHMARKS - ROUTE DATA");
    println!("=================================");

    let route_dataset = create_route_dataset(5000);

    println!("\nRoute dataset (5000 items):");
    benchmark_search(&route_dataset, "Departure", "Airport name search");
    benchmark_search(&route_dataset, "DEP", "Departure ICAO search");
    benchmark_search(&route_dataset, "DST", "Destination ICAO search");
    benchmark_search(&route_dataset, "Manufacturer", "Aircraft manufacturer");
    benchmark_search(&route_dataset, "Variant", "Aircraft variant");
    benchmark_search(&route_dataset, "nm", "Distance search");

    // Test route generation performance (simplified without real database)
    println!("\n⚡ ROUTE GENERATION BENCHMARKS");
    println!("=============================");
    
    // Create a single app service for route generation benchmarks
    if let Ok(db_pool) = DatabasePool::new(None, None) {
        if let Ok(app_service) = AppService::new(db_pool) {
            println!("\nRoute generation performance:");
            benchmark_route_generation(&app_service, DisplayMode::RandomRoutes, 50);
            benchmark_route_generation(&app_service, DisplayMode::NotFlownRoutes, 50);
        } else {
            println!("\nSkipping route generation benchmarks (AppService creation failed)");
        }
    } else {
        println!("\nSkipping route generation benchmarks (database setup failed)");
    }    println!("\n✅ Benchmark Complete!");
}
