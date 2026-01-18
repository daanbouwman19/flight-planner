use std::{collections::HashMap, sync::Arc};

use rand::prelude::*;

use crate::models::{Aircraft, Airport, Runway};
use crate::util::METERS_TO_FEET;

#[cfg(feature = "gui")]
use {
    crate::{
        gui::data::ListItemRoute, modules::airport::get_random_destination_airport_fast,
        util::calculate_haversine_distance_nm,
    },
    rayon::iter::{IntoParallelIterator, ParallelIterator},
    std::time::Instant,
};

pub const GENERATE_AMOUNT: usize = 50;

/// A struct responsible for generating flight routes.
///
/// `RouteGenerator` holds pre-computed data and caches to enable fast and
/// efficient route generation. It is designed to be created once and reused
/// for generating multiple sets of routes.
pub struct RouteGenerator {
    /// A vector of all available airports.
    /// OPTIMIZATION: This vector is sorted by longest runway length to enable
    /// fast binary search filtering without extra allocations.
    pub all_airports: Vec<Arc<Airport>>,
    /// A map from airport ID to a vector of its runways.
    pub all_runways: HashMap<i32, Arc<Vec<Runway>>>,
    /// An R-tree containing all airports for efficient spatial queries.
    pub spatial_airports: rstar::RTree<crate::models::airport::SpatialAirport>,
    /// A cache for the longest runway length of each airport, keyed by airport ID.
    pub longest_runway_cache: HashMap<i32, i32>,
    /// Parallel vector to all_airports containing the longest runway length for each.
    /// Used for fast binary search filtering.
    pub sorted_runway_lengths: Vec<i32>,
    /// A cache for pre-formatted airport display strings ("Name (ICAO)"), keyed by airport ID.
    /// This avoids repetitive string formatting and allocation during route generation.
    pub airport_display_cache: HashMap<i32, Arc<String>>,
}

impl RouteGenerator {
    /// Creates a new `RouteGenerator` with optimized caches for fast route generation.
    ///
    /// This constructor pre-processes the airport and runway data to build
    /// efficient lookup structures, such as a cache for the longest runway at each
    /// airport and a sorted index of airports by runway length.
    ///
    /// # Arguments
    ///
    /// * `all_airports` - A vector of all airports.
    /// * `all_runways` - A map from airport ID to its runways.
    /// * `spatial_airports` - An R-tree of all airports for spatial queries.
    ///
    /// # Returns
    ///
    /// A new `RouteGenerator` instance.
    pub fn new(
        mut all_airports: Vec<Arc<Airport>>,
        all_runways: HashMap<i32, Arc<Vec<Runway>>>,
        spatial_airports: rstar::RTree<crate::models::airport::SpatialAirport>,
    ) -> Self {
        // Pre-compute airport cache with longest runway lengths
        let mut longest_runway_cache = HashMap::new();

        for airport in &all_airports {
            if let Some(runways) = all_runways.get(&airport.ID) {
                let longest_runway_length = runways.iter().map(|r| r.Length).max().unwrap_or(0);
                longest_runway_cache.insert(airport.ID, longest_runway_length);
            }
        }

        // OPTIMIZATION: Sort airports by runway length to enable binary search.
        // This removes the need for "buckets" and redundant Vec<Arc> storage.
        all_airports.sort_by_key(|a| longest_runway_cache.get(&a.ID).copied().unwrap_or(0));

        // Create parallel vector of runway lengths for binary search
        let sorted_runway_lengths: Vec<i32> = all_airports
            .iter()
            .map(|a| longest_runway_cache.get(&a.ID).copied().unwrap_or(0))
            .collect();

        // Pre-calculate display strings for all airports to avoid allocations during route generation
        let airport_display_cache: HashMap<i32, Arc<String>> = all_airports
            .iter()
            .map(|a| (a.ID, Arc::new(format!("{} ({})", a.Name, a.ICAO))))
            .collect();

        Self {
            all_airports,
            all_runways,
            spatial_airports,
            longest_runway_cache,
            sorted_runway_lengths,
            airport_display_cache,
        }
    }

    /// Selects a random airport with a runway suitable for the given aircraft.
    ///
    /// This method uses pre-computed indexes and binary search for very
    /// fast lookups, avoiding the need for iterating or filtering large lists
    /// of airports.
    ///
    /// # Arguments
    ///
    /// * `aircraft` - The aircraft for which a suitable airport is needed.
    ///
    /// # Returns
    ///
    /// An `Option` containing an `Arc<Airport>` if a suitable airport is found,
    /// otherwise `None`.
    pub fn get_airport_with_suitable_runway_optimized<R: Rng + ?Sized>(
        &self,
        aircraft: &Aircraft,
        rng: &mut R,
    ) -> Option<Arc<Airport>> {
        let required_length_ft = aircraft
            .takeoff_distance
            .map(|d| (f64::from(d) * METERS_TO_FEET).round() as i32)
            .unwrap_or(0);

        // Binary search to find the start index where runway_length >= required_length
        // partition_point returns the index of the first element satisfying the predicate (false condition for <)
        let start_idx = self
            .sorted_runway_lengths
            .partition_point(|&len| len < required_length_ft);

        // Get slice of suitable airports
        // Since both vectors are sorted by runway length, all airports from start_idx onwards are valid
        let suitable_airports = &self.all_airports[start_idx..];

        if suitable_airports.is_empty() {
            return None;
        }

        // Just pick one at random. No further filtering needed!
        suitable_airports.choose(rng).map(|a| Arc::clone(a))
    }

    /// Generates random routes for aircraft that have not yet been flown.
    ///
    /// This function filters the provided list of aircraft to include only those
    /// that are marked as not flown, and then generates a set of random routes for them.
    ///
    /// # Arguments
    ///
    /// * `all_aircraft` - A slice of all available aircraft.
    /// * `departure_airport_icao` - An optional ICAO code for a fixed departure airport.
    ///
    /// # Returns
    ///
    /// A `Vec<ListItemRoute>` containing the generated routes.
    #[cfg(feature = "gui")]
    pub fn generate_random_not_flown_aircraft_routes(
        &self,
        all_aircraft: &[Arc<Aircraft>],
        departure_airport_icao: Option<&str>,
    ) -> Vec<ListItemRoute> {
        let not_flown_aircraft: Vec<_> = all_aircraft
            .iter()
            .filter(|aircraft| aircraft.flown == 0)
            .cloned()
            .collect();

        self.generate_random_routes_generic(
            &not_flown_aircraft,
            GENERATE_AMOUNT,
            departure_airport_icao,
        )
    }

    /// Generates a list of random routes for any aircraft.
    ///
    /// # Arguments
    ///
    /// * `all_aircraft` - A slice of all available aircraft to choose from.
    /// * `departure_airport_icao` - An optional ICAO code for a fixed departure airport.
    ///
    /// # Returns
    ///
    /// A `Vec<ListItemRoute>` containing the generated routes.
    #[cfg(feature = "gui")]
    pub fn generate_random_routes(
        &self,
        all_aircraft: &[Arc<Aircraft>],
        departure_airport_icao: Option<&str>,
    ) -> Vec<ListItemRoute> {
        self.generate_random_routes_generic(all_aircraft, GENERATE_AMOUNT, departure_airport_icao)
    }

    /// Generates a list of random routes for a specific aircraft.
    ///
    /// # Arguments
    ///
    /// * `aircraft` - The specific aircraft for which to generate routes.
    /// * `departure_airport_icao` - An optional ICAO code for a fixed departure airport.
    ///
    /// # Returns
    ///
    /// A `Vec<ListItemRoute>` containing the generated routes.
    #[cfg(feature = "gui")]
    pub fn generate_routes_for_aircraft(
        &self,
        aircraft: &Arc<Aircraft>,
        departure_airport_icao: Option<&str>,
    ) -> Vec<ListItemRoute> {
        let aircraft_slice = &[Arc::clone(aircraft)];
        self.generate_random_routes_generic(aircraft_slice, GENERATE_AMOUNT, departure_airport_icao)
    }

    /// The generic engine for generating a specified number of random routes.
    ///
    /// This function serves as the core logic for route generation. It takes a list
    /// of aircraft, a desired number of routes, and an optional fixed departure
    /// airport. It then generates routes in parallel for maximum performance.
    ///
    /// # Arguments
    ///
    /// * `aircraft_list` - A slice of `Arc<Aircraft>` to be used for generating routes.
    ///   A random aircraft from this list is chosen for each route.
    /// * `amount` - The number of routes to generate.
    /// * `departure_airport_icao` - If `Some`, all generated routes will depart from
    ///   the specified airport ICAO. If `None`, a random suitable departure airport
    ///   is chosen for each route.
    ///
    /// # Returns
    ///
    /// A `Vec<ListItemRoute>` containing the generated routes.
    #[cfg(feature = "gui")]
    pub fn generate_random_routes_generic(
        &self,
        aircraft_list: &[Arc<Aircraft>],
        amount: usize,
        departure_airport_icao: Option<&str>,
    ) -> Vec<ListItemRoute> {
        let start_time = Instant::now();

        // Validate and lookup departure airport once before the parallel loop
        let departure_airport: Option<Arc<Airport>> = if let Some(icao) = departure_airport_icao {
            let icao_upper = icao.to_uppercase();
            if let Some(airport) = self.all_airports.iter().find(|a| a.ICAO == icao_upper) {
                Some(Arc::clone(airport))
            } else {
                log::warn!("Departure airport with ICAO '{icao}' not found in database");
                return Vec::new();
            }
        } else {
            None
        };

        // Cache aircraft display strings to avoid repeated formatting/allocation
        let aircraft_display_cache: HashMap<i32, Arc<String>> = aircraft_list
            .iter()
            .map(|a| (a.id, Arc::new(format!("{} {}", a.manufacturer, a.variant))))
            .collect();

        // Use parallel processing for optimal performance
        let routes: Vec<ListItemRoute> = (0..amount)
            .into_par_iter()
            .filter_map(|_| -> Option<ListItemRoute> {
                let mut rng = rand::rng();
                self.generate_single_route(
                    aircraft_list,
                    &departure_airport,
                    &mut rng,
                    &aircraft_display_cache,
                )
            })
            .collect();

        let duration = start_time.elapsed();
        log::info!(
            "Generated {} routes in {}ms",
            routes.len(),
            duration.as_millis()
        );

        routes
    }

    /// Generate a single route (parallel-safe version)
    #[cfg(feature = "gui")]
    fn generate_single_route<R: Rng + ?Sized>(
        &self,
        aircraft_list: &[Arc<Aircraft>],
        departure_airport: &Option<Arc<Airport>>,
        rng: &mut R,
        aircraft_display_cache: &HashMap<i32, Arc<String>>,
    ) -> Option<ListItemRoute> {
        let aircraft = aircraft_list.choose(rng)?;

        let departure = departure_airport.as_ref().map_or_else(
            || self.get_airport_with_suitable_runway_optimized(aircraft, rng),
            |airport| Some(Arc::clone(airport)),
        );

        let departure = departure?;

        // Use cached longest runway length for departure (avoid redundant lookup)
        let departure_longest_runway_length = self
            .longest_runway_cache
            .get(&departure.ID)
            .copied()
            .unwrap_or(0);

        let required_length_ft = aircraft
            .takeoff_distance
            .map(|d| (f64::from(d) * METERS_TO_FEET).round() as i32)
            .unwrap_or(0);

        // Find start index of suitable airports using binary search
        let start_idx = self
            .sorted_runway_lengths
            .partition_point(|&len| len < required_length_ft);

        // Exact slice of suitable airports
        let suitable_airports = if start_idx < self.all_airports.len() {
            Some(&self.all_airports[start_idx..])
        } else {
            None
        };

        // Get single destination candidate directly from iterator (avoids Vec allocation)
        let destination_arc_ref = get_random_destination_airport_fast(
            aircraft,
            &departure,
            suitable_airports,
            &self.spatial_airports,
            &self.longest_runway_cache,
            rng,
        )?;

        // Use cached longest runway length for destination (avoid redundant lookup)
        let destination_longest_runway_length = self
            .longest_runway_cache
            .get(&destination_arc_ref.ID)
            .copied()
            .unwrap_or(0);

        // Calculate distance only once
        let route_length =
            calculate_haversine_distance_nm(&departure, destination_arc_ref.as_ref()) as f64;

        // Retrieve pre-formatted strings from caches
        let aircraft_info = aircraft_display_cache
            .get(&aircraft.id)
            .cloned()
            .unwrap_or_else(|| Arc::new(format!("{} {}", aircraft.manufacturer, aircraft.variant)));

        // Retrieve pre-formatted strings from global airport cache to avoid allocation
        let departure_info = self
            .airport_display_cache
            .get(&departure.ID)
            .cloned()
            .unwrap_or_else(|| Arc::new(format!("{} ({})", departure.Name, departure.ICAO)));

        let destination_info = self
            .airport_display_cache
            .get(&destination_arc_ref.ID)
            .cloned()
            .unwrap_or_else(|| {
                Arc::new(format!(
                    "{} ({})",
                    destination_arc_ref.Name, destination_arc_ref.ICAO
                ))
            });

        Some(ListItemRoute {
            departure: Arc::clone(&departure),
            destination: Arc::clone(destination_arc_ref),
            aircraft: Arc::clone(aircraft),
            departure_runway_length: departure_longest_runway_length,
            destination_runway_length: destination_longest_runway_length,
            route_length,
            aircraft_info,
            departure_info,
            destination_info,
            distance_str: format!("{route_length:.1} NM"),
            created_at: Instant::now(),
        })
    }
}
