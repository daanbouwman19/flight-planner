use std::{sync::Arc, time::Instant};

use rand::prelude::*;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{
    gui::data::ListItemRoute,
    models::{Aircraft, Airport, Runway},
    modules::airport::{
        get_airport_with_suitable_runway_fast, get_destination_airports_with_suitable_runway_fast,
    },
    util::calculate_haversine_distance_nm,
};

const GENERATE_AMOUNT: usize = 50;

pub struct RouteGenerator {
    pub all_airports: Vec<Arc<Airport>>,
    pub all_runways: std::collections::HashMap<i32, Arc<Vec<Runway>>>,
    pub spatial_airports: rstar::RTree<crate::gui::ui::SpatialAirport>,
}

impl RouteGenerator {
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
                    || {
                        get_airport_with_suitable_runway_fast(
                            aircraft,
                            &self.all_airports,
                            &self.all_runways,
                        )
                        .ok()
                    },
                    |airport| Some(Arc::clone(airport)),
                );

                let departure = departure?;
                let departure_runways = self.all_runways.get(&departure.ID)?;
                let longest_runway = departure_runways.iter().max_by_key(|r| r.Length)?;

                let mut airports = get_destination_airports_with_suitable_runway_fast(
                    aircraft,
                    &departure,
                    &self.spatial_airports,
                    &self.all_runways,
                );
                airports.shuffle(&mut rng);

                let destination_arc = airports.pop()?;
                let destination_runways = self.all_runways.get(&destination_arc.ID)?;
                let destination_runways = destination_runways.clone();

                let route_length =
                    calculate_haversine_distance_nm(&departure, destination_arc.as_ref());

                Some(ListItemRoute {
                    departure: Arc::clone(&departure),
                    destination: Arc::clone(destination_arc),
                    aircraft: Arc::clone(aircraft),
                    departure_runway_length: longest_runway.Length.to_string(),
                    destination_runway_length: destination_runways
                        .iter()
                        .max_by_key(|r| r.Length)
                        .unwrap()
                        .Length
                        .to_string(),
                    route_length: route_length.to_string(),
                })
            })
            .collect();

        let duration = start_time.elapsed();
        log::info!("Generated {} routes in {:?}", routes.len(), duration);

        routes
    }
}
