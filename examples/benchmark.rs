// Comprehensive benchmark for flight planner performance
//
// This benchmark tests:
// - Search performance with different query types and dataset sizes
// - Route generation performance with timing measurements
//
// If the airport database is not available, it will use mock data for consistent testing.
//
// Usage:
//   cargo run --release --example benchmark              # Use real database or default mock (16,343 airports)
//   cargo run --release --example benchmark 5000         # Use mock data with 5,000 airports
//   cargo run --release --example benchmark 50000        # Use mock data with 50,000 airports

mod mock_data;

use flight_planner::database::DatabasePool;
use flight_planner::gui::data::{ListItemAirport, ListItemRoute, TableItem};
use flight_planner::gui::services::{AppService, SearchService};
use flight_planner::modules::routes::RouteGenerator;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug)]
struct BenchmarkResults {
    avg_routes: f64,
    avg_time: Duration,
    std_dev: Duration,
    min_time: Duration,
    max_time: Duration,
    total_time: Duration,
}

fn create_test_dataset(size: usize) -> Vec<Arc<TableItem>> {
    (0..size)
        .map(|i| {
            Arc::new(TableItem::Airport(ListItemAirport::new(
                format!("Airport {i}"),
                format!("A{i:03}"),
                format!("{}00ft", 10 + i % 50),
            )))
        })
        .collect()
}

/// Generic function to benchmark route generation with any closure
fn benchmark_route_generation_impl<F>(
    test_name: &str,
    iterations: u32,
    mut route_generator: F,
) -> BenchmarkResults
where
    F: FnMut() -> Vec<ListItemRoute>,
{
    println!("  \n{test_name}");

    let mut total_time = Duration::ZERO;
    let mut route_counts = Vec::new();
    let mut times = Vec::new();

    // Progress indicator - guard against division by zero for small iteration counts
    let progress_interval = std::cmp::max(1, iterations / 10);

    for i in 0..iterations {
        let start = Instant::now();
        let routes = route_generator();
        let duration = start.elapsed();

        total_time += duration;
        route_counts.push(routes.len());
        times.push(duration);

        // Show progress every 10% (or for small iteration counts, more frequently)
        if (i + 1) % progress_interval == 0 {
            let progress = ((i + 1) * 100) / iterations;
            println!("    Progress: {}% ({}/{})", progress, i + 1, iterations);
        }
    }

    // Calculate statistics
    let avg_time = total_time / iterations;
    let avg_routes = route_counts.iter().sum::<usize>() as f64 / iterations as f64;

    // Calculate standard deviation for timing
    let mean_nanos = avg_time.as_nanos() as f64;
    let variance = times
        .iter()
        .map(|&t| {
            let diff = t.as_nanos() as f64 - mean_nanos;
            diff * diff
        })
        .sum::<f64>()
        / iterations as f64;
    let std_dev = Duration::from_nanos(variance.sqrt() as u64);

    // Find min/max times
    let min_time = *times
        .iter()
        .min()
        .expect("At least one timing measurement should exist");
    let max_time = *times
        .iter()
        .max()
        .expect("At least one timing measurement should exist");

    BenchmarkResults {
        avg_routes,
        avg_time,
        std_dev,
        min_time,
        max_time,
        total_time,
    }
}

/// Print benchmark results in a consistent format
fn print_benchmark_results(test_name: &str, results: &BenchmarkResults) {
    println!("    📊 {test_name} Results:");
    println!("      Average routes: {:.1}", results.avg_routes);
    println!(
        "      Average time: {}ms (±{}ms)",
        results.avg_time.as_millis(),
        results.std_dev.as_millis()
    );
    println!(
        "      Range: {}ms - {}ms",
        results.min_time.as_millis(),
        results.max_time.as_millis()
    );
    println!(
        "      Per route: {}ms",
        (results.avg_time / flight_planner::modules::routes::GENERATE_AMOUNT as u32).as_millis()
    );
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

        println!(
            "  {}: {} results in {:?}",
            description,
            results.len(),
            duration
        );
    }
}

fn benchmark_route_generation_with_count(custom_airport_count: Option<usize>) {
    println!("\n⚡ ROUTE GENERATION PERFORMANCE");
    println!("===============================");

    // Try to use real database first, fall back to mock data
    let (route_generator, aircraft, using_mock) = if let Some(count) = custom_airport_count {
        // Force mock data if custom count is specified
        println!("  Using mock data with custom airport count: {}", count);
        create_mock_data_with_count(count)
    } else if let Ok(db_pool) = DatabasePool::new(None, None) {
        if let Ok(service) = AppService::new(db_pool) {
            println!(
                "  Using real database with {} airports and {} aircraft",
                service.airports().len(),
                service.aircraft().len()
            );
            (
                Arc::clone(service.route_generator()),
                service.aircraft().to_vec(),
                false,
            )
        } else {
            println!("  ⚠️  Database connection failed, using mock data");
            create_mock_data()
        }
    } else {
        println!("  ℹ️  No database available, using mock data");
        create_mock_data()
    };

    if using_mock {
        println!(
            "  Generated {} airports and {} aircraft",
            route_generator.all_airports.len(),
            aircraft.len()
        );
    }

    let iterations = 100;
    const BENCHMARK_GENERATE_AMOUNT: usize = 1000; // 20x more work
    println!(
        "  Running {iterations} iterations for statistical accuracy (generating {BENCHMARK_GENERATE_AMOUNT} routes each)..."
    );

    // Warm up
    let _ =
        route_generator.generate_random_routes_generic(&aircraft, BENCHMARK_GENERATE_AMOUNT, None);

    // Test Random Routes using the helper function
    let random_results =
        benchmark_route_generation_impl("🎲 Testing Random Routes:", iterations, || {
            route_generator.generate_random_routes_generic(
                &aircraft,
                BENCHMARK_GENERATE_AMOUNT,
                None,
            )
        });
    print_benchmark_results("Random Routes", &random_results);

    println!("\n  🏁 SUMMARY:");
    println!("    Total iterations: {}", iterations);
    println!("    Total test time: {:?}", random_results.total_time);
}

/// Create mock data for benchmarking when database is not available
fn create_mock_data() -> (
    Arc<RouteGenerator>,
    Vec<Arc<flight_planner::models::Aircraft>>,
    bool,
) {
    create_mock_data_with_count(mock_data::DEFAULT_AIRPORT_COUNT)
}

/// Create mock data with a specific airport count
fn create_mock_data_with_count(
    airport_count: usize,
) -> (
    Arc<RouteGenerator>,
    Vec<Arc<flight_planner::models::Aircraft>>,
    bool,
) {
    // Load aircraft from the CSV file in the repository
    let aircraft = match mock_data::load_aircraft_from_csv() {
        Ok(aircraft) => {
            println!(
                "  ℹ️  Loaded {} aircraft from aircrafts.csv",
                aircraft.len()
            );
            aircraft
        }
        Err(e) => {
            eprintln!("  ⚠️  Failed to load aircrafts.csv: {}", e);
            eprintln!("  ℹ️  Make sure you run the benchmark from the repository root");
            Vec::new()
        }
    };

    if aircraft.is_empty() {
        eprintln!("  ❌ No aircraft data available, cannot run benchmark");
        std::process::exit(1);
    }

    // Generate realistic mock airports
    let airports = mock_data::generate_mock_airports(airport_count);
    let runways = mock_data::generate_mock_runways(&airports);
    let spatial_rtree = mock_data::generate_spatial_rtree(&airports);

    let route_generator = Arc::new(RouteGenerator::new(
        airports.clone(),
        runways,
        spatial_rtree,
    ));

    (route_generator, aircraft, true)
}

fn main() {
    println!("Flight Planner Performance Benchmark");
    println!("====================================");

    // Parse command line arguments for custom airport count
    let args: Vec<String> = std::env::args().collect();
    let custom_airport_count = if args.len() > 1 {
        match args[1].parse::<usize>() {
            Ok(count) if count > 0 => {
                println!("  ℹ️  Using custom airport count: {}", count);
                Some(count)
            }
            _ => {
                eprintln!("  ⚠️  Invalid airport count '{}', using default", args[1]);
                None
            }
        }
    } else {
        None
    };

    benchmark_search_performance();
    benchmark_route_generation_with_count(custom_airport_count);

    println!("\n✅ Benchmark Complete!");
}
