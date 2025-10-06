use std::{collections::HashMap, sync::Arc, time::Instant};

use rand::{prelude::*, seq::IteratorRandom};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use reqwest::Client;
use tokio::runtime::Runtime;

use crate::{
    gui::{
        data::ListItemRoute,
        state::{FlightRules, WeatherFilterState},
    },
    models::{Aircraft, Airport, Runway},
    modules::{
        airport::get_destination_airports_with_suitable_runway_fast, weather,
    },
    util::{calculate_haversine_distance_nm, METERS_TO_FEET},
};

pub const GENERATE_AMOUNT: usize = 50;
/// Number of random selection attempts before falling back to filtering
const RANDOM_SELECTION_ATTEMPTS: usize = 3;
const AVWX_API_URL: &str = "https://avwx.rest";

/// A struct responsible for generating flight routes.
///
/// `RouteGenerator` holds pre-computed data and caches to enable fast and
/// efficient route generation. It is designed to be created once and reused
/// for generating multiple sets of routes.
pub struct RouteGenerator {
    /// A vector of all available airports.
    pub all_airports: Vec<Arc<Airport>>,
    /// A map from airport ID to a vector of its runways.
    pub all_runways: HashMap<i32, Arc<Vec<Runway>>>,
    /// An R-tree containing all airports for efficient spatial queries.
    pub spatial_airports: rstar::RTree<crate::models::airport::SpatialAirport>,
    /// A cache for the longest runway length of each airport, keyed by airport ID.
    pub longest_runway_cache: HashMap<i32, i32>,
    /// An index of airports categorized by minimum runway length requirements (in feet).
    pub airports_by_runway_length: HashMap<i32, Vec<Arc<Airport>>>,
}

impl RouteGenerator {
    /// Creates a new `RouteGenerator` with optimized caches for fast route generation.
    ///
    /// This constructor pre-processes the airport and runway data to build
    /// efficient lookup structures, such as a cache for the longest runway at each
    /// airport and an index of airports bucketed by runway length.
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
        let mut airport_cache: Vec<AirportCache> = all_airports
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
        // Use optimized buckets based on common aircraft takeoff distances
        let runway_buckets = vec![
            0,     // All airports
            1000,  // Very small aircraft
            2000,  // Light aircraft
            2500,  // Small turboprops
            3000,  // Regional jets
            4000,  // Narrow-body jets
            5000,  // Medium jets
            6000,  // Large jets
            8000,  // Wide-body jets
            10000, // Heavy jets
            12000, // Super heavy
            15000, // Largest aircraft
        ];
        let mut airports_by_runway_length: HashMap<i32, Vec<Arc<Airport>>> = HashMap::new();

        // Pre-sort airports by longest runway length for efficient bucket creation
        airport_cache.sort_by_key(|cache| cache.longest_runway_length);

        for &min_length in &runway_buckets {
            // Use binary search for efficient filtering since data is sorted
            let start_idx = airport_cache
                .binary_search_by_key(&min_length, |cache| cache.longest_runway_length)
                .unwrap_or_else(|i| i);

            let suitable_airports: Vec<Arc<Airport>> = airport_cache[start_idx..]
                .iter()
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

    /// Selects a random airport with a runway suitable for the given aircraft.
    ///
    /// This method uses pre-computed indexes and runway length buckets for very
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
    pub fn get_airport_with_suitable_runway_optimized(
        &self,
        aircraft: &Aircraft,
    ) -> Option<Arc<Airport>> {
        let required_length_ft = aircraft
            .takeoff_distance
            .map(|d| (f64::from(d) * METERS_TO_FEET).round() as i32)
            .unwrap_or(0);

        // Find the best bucket: largest bucket <= required length
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

        // Since we're using buckets, we still need to verify the exact runway requirement
        // But this is much faster than the previous approach
        let mut rng = rand::rng();

        // For performance, first try a few random selections from the bucket
        // before falling back to filtering the entire list
        for _ in 0..RANDOM_SELECTION_ATTEMPTS {
            if let Some(airport) = suitable_airports.choose(&mut rng)
                && let Some(&runway_length) = self.longest_runway_cache.get(&airport.ID)
                && runway_length >= required_length_ft
            {
                return Some(Arc::clone(airport));
            }
        }

        // Fallback: filter and choose (for when bucket boundaries don't align perfectly)
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
    pub fn generate_random_not_flown_aircraft_routes(
        &self,
        all_aircraft: &[Arc<Aircraft>],
        departure_airport_icao: Option<&str>,
        weather_filter: &WeatherFilterState,
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
            weather_filter,
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
    pub fn generate_random_routes(
        &self,
        all_aircraft: &[Arc<Aircraft>],
        departure_airport_icao: Option<&str>,
        weather_filter: &WeatherFilterState,
    ) -> Vec<ListItemRoute> {
        self.generate_random_routes_generic(
            all_aircraft,
            GENERATE_AMOUNT,
            departure_airport_icao,
            weather_filter,
        )
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
    pub fn generate_routes_for_aircraft(
        &self,
        aircraft: &Arc<Aircraft>,
        departure_airport_icao: Option<&str>,
        weather_filter: &WeatherFilterState,
    ) -> Vec<ListItemRoute> {
        let aircraft_slice = &[Arc::clone(aircraft)];
        self.generate_random_routes_generic(
            aircraft_slice,
            GENERATE_AMOUNT,
            departure_airport_icao,
            weather_filter,
        )
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
    pub fn generate_random_routes_generic(
        &self,
        aircraft_list: &[Arc<Aircraft>],
        amount: usize,
        departure_airport_icao: Option<&str>,
        weather_filter: &WeatherFilterState,
    ) -> Vec<ListItemRoute> {
        let start_time = Instant::now();

        // Validate and lookup departure airport once
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

        // Generate a pool of potential routes. We generate more than needed to ensure
        // we have enough valid options after weather filtering.
        let pool_size = amount * 5; // Generate 5x the required routes
        let mut potential_routes: Vec<(Arc<Aircraft>, Arc<Airport>, Arc<Airport>)> = (0..pool_size)
            .into_par_iter()
            .filter_map(|_| self.generate_single_route_tuple(aircraft_list, &departure_airport))
            .collect();

        // If weather filtering is enabled, filter the routes
        if weather_filter.enabled {
            let rt = Runtime::new().unwrap();
            potential_routes =
                rt.block_on(self.filter_routes_by_weather(potential_routes, weather_filter));
        }

        // Randomly select the final number of routes from the (potentially filtered) pool
        let mut rng = rand::rng();
        let final_routes: Vec<_> = potential_routes
            .into_iter()
            .choose_multiple(&mut rng, amount)
            .into_iter()
            .map(|(aircraft, departure, destination)| {
                self.create_list_item_route(aircraft, departure, destination)
            })
            .collect();

        let duration = start_time.elapsed();
        log::info!("Generated {} routes in {:?}", final_routes.len(), duration);

        final_routes
    }

    /// Generates a single route tuple (aircraft, departure, destination).
    fn generate_single_route_tuple(
        &self,
        aircraft_list: &[Arc<Aircraft>],
        departure_airport: &Option<Arc<Airport>>,
    ) -> Option<(Arc<Aircraft>, Arc<Airport>, Arc<Airport>)> {
        let mut rng = rand::rng();
        let aircraft = aircraft_list.choose(&mut rng)?;

        let departure = departure_airport.as_ref().map_or_else(
            || self.get_airport_with_suitable_runway_optimized(aircraft),
            |airport| Some(Arc::clone(airport)),
        )?;

        let airports_iter = get_destination_airports_with_suitable_runway_fast(
            aircraft,
            &departure,
            &self.spatial_airports,
            &self.all_runways,
        );

        let destination = airports_iter.choose(&mut rng)?;

        Some((
            Arc::clone(aircraft),
            departure.clone(),
            destination.clone(),
        ))
    }

    /// Filters a list of potential routes based on weather conditions.
    async fn filter_routes_by_weather(
        &self,
        routes: Vec<(Arc<Aircraft>, Arc<Airport>, Arc<Airport>)>,
        weather_filter: &WeatherFilterState,
    ) -> Vec<(Arc<Aircraft>, Arc<Airport>, Arc<Airport>)> {
        let api_key = match std::env::var("AVWX_API_KEY") {
            Ok(key) => key,
            Err(_) => {
                log::error!("AVWX_API_KEY environment variable not set. Cannot filter by weather.");
                return routes;
            }
        };

        let client = Client::new();
        let unique_destinations: HashMap<_, _> = routes
            .iter()
            .map(|(_, _, dest)| (dest.ICAO.clone(), Arc::clone(dest)))
            .collect();

        let weather_futs = unique_destinations.values().map(|airport| {
            let client_clone = client.clone();
            let api_key_clone = api_key.clone();
            async move {
                (
                    airport.ICAO.clone(),
                    weather::get_weather_data(AVWX_API_URL, &airport.ICAO, &client_clone, &api_key_clone)
                        .await,
                )
            }
        });

        let weather_results: HashMap<String, _> =
            futures::future::join_all(weather_futs).await.into_iter().collect();

        let max_wind_kts: Option<u32> = weather_filter.max_wind_speed.parse().ok();
        let min_wind_kts: Option<u32> = weather_filter.min_wind_speed.parse().ok();
        let min_vis_mi: Option<f32> = weather_filter.min_visibility.parse().ok();

        routes
            .into_iter()
            .filter(|(_, _, dest)| {
                if let Some(Ok(metar)) = weather_results.get(&dest.ICAO) {
                    if let Some(max_wind) = max_wind_kts {
                        if let Some(wind) = &metar.wind {
                            if wind.speed_kts > max_wind {
                                return false;
                            }
                        }
                    }
                    if let Some(min_wind) = min_wind_kts {
                        if let Some(wind) = &metar.wind {
                            if wind.speed_kts < min_wind {
                                return false;
                            }
                        } else {
                            return false; // No wind data, can't meet min requirement
                        }
                    }
                    if let Some(min_vis) = min_vis_mi {
                        if let Some(vis) = &metar.visibility {
                            if vis.miles < min_vis {
                                return false;
                            }
                        }
                    }
                    if weather_filter.flight_rules != FlightRules::Any
                        && metar.flight_rules != weather_filter.flight_rules.to_string()
                    {
                        return false;
                    }
                    true // All checks passed
                } else {
                    // Could not get weather, exclude from results
                    log::debug!("Excluding {} due to missing weather data", dest.ICAO);
                    false
                }
            })
            .collect()
    }

    /// Creates a `ListItemRoute` from its constituent parts.
    fn create_list_item_route(
        &self,
        aircraft: Arc<Aircraft>,
        departure: Arc<Airport>,
        destination: Arc<Airport>,
    ) -> ListItemRoute {
        let departure_longest_runway_length =
            self.longest_runway_cache.get(&departure.ID).copied().unwrap_or(0);
        let destination_longest_runway_length = self
            .longest_runway_cache
            .get(&destination.ID)
            .copied()
            .unwrap_or(0);
        let route_length = calculate_haversine_distance_nm(&departure, &destination);

        ListItemRoute {
            departure,
            destination,
            aircraft,
            departure_runway_length: departure_longest_runway_length.to_string(),
            destination_runway_length: destination_longest_runway_length.to_string(),
            route_length: route_length.to_string(),
        }
    }
}
