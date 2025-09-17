use std::{collections::HashMap, sync::Arc, time::Instant};

use rand::{prelude::*, seq::IteratorRandom};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{
    gui::data::ListItemRoute,
    models::{Aircraft, Airport, Runway},
    modules::airport::get_destination_airports_with_suitable_runway_fast,
    util::{METERS_TO_FEET, calculate_haversine_distance_nm},
};

pub const GENERATE_AMOUNT: usize = 50;

pub struct RouteGenerator {
    pub all_airports: Vec<Arc<Airport>>,
    pub all_runways: HashMap<i32, Arc<Vec<Runway>>>,
    pub spatial_airports: rstar::RTree<crate::models::airport::SpatialAirport>,
    /// Fast lookup of longest runway length by airport ID
    pub longest_runway_cache: HashMap<i32, i32>,
    /// Index of airports by minimum runway length requirement (in feet)
    pub airports_by_runway_length: HashMap<i32, Vec<Arc<Airport>>>,
}

impl RouteGenerator {
    /// Creates a new RouteGenerator with optimized caches for fast route generation.
    pub fn new(
        all_airports: Vec<Arc<Airport>>,
        all_runways: HashMap<i32, Arc<Vec<Runway>>>,
        spatial_airports: rstar::RTree<crate::models::airport::SpatialAirport>,
    ) -> Self {
        /// Cached airport data for efficient lookup - local helper struct
        #[derive(Clone)]
        struct AirportCache {
            airport: Arc<Airport>,
            longest_runway_length: i32,
        }

        // Pre-compute airport cache with longest runway lengths
        let mut longest_runway_cache = HashMap::new();
        let airport_cache: Vec<AirportCache> = all_airports
            .iter()
            .filter_map(|airport| {
                let runways = all_runways.get(&airport.ID)?;
                let longest_runway_length = runways.iter().map(|r| r.Length).max().unwrap_or(0);
                longest_runway_cache.insert(airport.ID, longest_runway_length);
                Some(AirportCache {
                    airport: Arc::clone(airport),
                    longest_runway_length,
                })
            })
            .collect();

        // Create index of airports by runway length buckets (in feet)
        // Use common takeoff distance thresholds to create efficient buckets
        let runway_buckets = vec![0, 1000, 2000, 3000, 4000, 5000, 6000, 8000, 10000, 15000];
        let mut airports_by_runway_length: HashMap<i32, Vec<Arc<Airport>>> = HashMap::new();

        for &min_length in &runway_buckets {
            let suitable_airports: Vec<Arc<Airport>> = airport_cache
                .iter()
                .filter(|cache| cache.longest_runway_length >= min_length)
                .map(|cache| Arc::clone(&cache.airport))
                .collect();
            airports_by_runway_length.insert(min_length, suitable_airports);
        }

        Self {
            all_airports,
            all_runways,
            spatial_airports,
            longest_runway_cache,
            airports_by_runway_length,
        }
    }

    /// Fast airport selection using pre-computed indexes instead of random attempts.
    fn get_airport_with_suitable_runway_optimized(
        &self,
        aircraft: &Aircraft,
    ) -> Option<Arc<Airport>> {
        let required_length_ft = aircraft
            .takeoff_distance
            .map(|d| (f64::from(d) * METERS_TO_FEET).round() as i32)
            .unwrap_or(0);

        // Find the appropriate bucket for this aircraft's requirements
        let bucket_key = self
            .airports_by_runway_length
            .keys()
            .filter(|&&bucket| bucket <= required_length_ft)
            .max()
            .copied()
            .unwrap_or(0);

        // Get suitable airports from the pre-computed index
        let suitable_airports = self.airports_by_runway_length.get(&bucket_key)?;

        if suitable_airports.is_empty() {
            return None;
        }

        let mut rng = rand::rng();
        // Filter airports to ensure they actually meet the required runway length
        suitable_airports
            .iter()
            .filter(|airport| {
                self.longest_runway_cache
                    .get(&airport.ID)
                    .is_some_and(|&length| length >= required_length_ft)
            })
            .choose(&mut rng)
            .map(Arc::clone)
    }

    /// Generates random routes for aircraft that have not been flown yet.
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

    /// Generates a list of random routes.
    pub fn generate_random_routes(
        &self,
        all_aircraft: &[Arc<Aircraft>],
        departure_airport_icao: Option<&str>,
    ) -> Vec<ListItemRoute> {
        self.generate_random_routes_generic(all_aircraft, GENERATE_AMOUNT, departure_airport_icao)
    }

    /// Generates routes for a specific aircraft.
    pub fn generate_routes_for_aircraft(
        &self,
        aircraft: &Arc<Aircraft>,
        departure_airport_icao: Option<&str>,
    ) -> Vec<ListItemRoute> {
        let aircraft_slice = &[Arc::clone(aircraft)];
        self.generate_random_routes_generic(aircraft_slice, GENERATE_AMOUNT, departure_airport_icao)
    }

    /// Generate random routes for aircraft.
    ///
    /// * `aircraft_list` - A slice of aircraft to generate routes for.
    /// * `amount` - The number of routes to generate.
    /// * `departure_airport_icao` - Optional departure airport ICAO code.
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

        let routes: Vec<ListItemRoute> = (0..amount)
            .into_par_iter()
            .filter_map(|_| -> Option<ListItemRoute> {
                let mut rng = rand::rng();
                let aircraft = aircraft_list.choose(&mut rng)?;

                let departure = departure_airport.as_ref().map_or_else(
                    || self.get_airport_with_suitable_runway_optimized(aircraft),
                    |airport| Some(Arc::clone(airport)),
                );

                let departure = departure?;

                // Use cached longest runway length for departure
                let departure_longest_runway_length = self
                    .longest_runway_cache
                    .get(&departure.ID)
                    .copied()
                    .unwrap_or(0);

                let airports_iter = get_destination_airports_with_suitable_runway_fast(
                    aircraft,
                    &departure,
                    &self.spatial_airports,
                    &self.all_runways,
                );

                // Choose a random destination from the iterator
                let destination = airports_iter.choose(&mut rng)?;

                // Use cached longest runway length for destination
                let destination_longest_runway_length = self
                    .longest_runway_cache
                    .get(&destination.ID)
                    .copied()
                    .unwrap_or(0);

                let route_length =
                    calculate_haversine_distance_nm(&departure, destination.as_ref());

                Some(ListItemRoute {
                    departure: Arc::clone(&departure),
                    destination: Arc::clone(destination),
                    aircraft: Arc::clone(aircraft),
                    departure_runway_length: departure_longest_runway_length.to_string(),
                    destination_runway_length: destination_longest_runway_length.to_string(),
                    route_length: route_length.to_string(),
                })
            })
            .collect();

        let duration = start_time.elapsed();
        log::info!("Generated {} routes in {:?}", routes.len(), duration);

        routes
    }
}
