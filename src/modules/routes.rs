use std::{sync::Arc, time::Instant};

use rand::prelude::*;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::{
    gui::ui::ListItemRoute,
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

    /// Generates random routes using a generic aircraft list.
    ///
    /// # Arguments
    ///
    /// * `aircraft_list` - A slice of aircraft to generate routes for.
    /// * `amount` - The number of routes to generate.
    fn generate_random_routes_generic(
        &self,
        aircraft_list: &[Arc<Aircraft>],
        amount: usize,
        departure_airport_icao: Option<&str>,
    ) -> Vec<ListItemRoute> {
        let start_time = Instant::now();

        let routes: Vec<ListItemRoute> = (0..amount)
            .into_par_iter()
            .filter_map(|_| -> Option<ListItemRoute> {
                let mut rng = rand::rng();
                let aircraft = aircraft_list.choose(&mut rng)?;

                let departure = departure_airport_icao.map_or_else(
                    || {
                        get_airport_with_suitable_runway_fast(
                            aircraft,
                            &self.all_airports,
                            &self.all_runways,
                        )
                        .ok()
                    },
                    |icao| self.all_airports.iter().find(|a| a.ICAO == icao).cloned(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::ui::ListItemRoute;
    use crate::models::{Aircraft, Airport, Runway};
    use rstar::RTree;
    use std::collections::HashMap;
    use std::sync::Arc;

    type AircraftVec = Vec<Arc<Aircraft>>;
    type AirportVec = Vec<Arc<Airport>>;
    type RunwayMap = HashMap<i32, Arc<Vec<Runway>>>;
    type AirportRTree = RTree<crate::gui::ui::SpatialAirport>;

    fn create_test_data() -> (AircraftVec, AirportVec, RunwayMap, AirportRTree) {
        let aircraft1 = Arc::new(Aircraft {
            id: 1,
            manufacturer: "Boeing".to_string(),
            variant: "737-800".to_string(),
            icao_code: "B738".to_string(),
            flown: 0,
            aircraft_range: 3000,
            category: "A".to_string(),
            cruise_speed: 450,
            date_flown: Some("2024-12-10".to_string()),
            takeoff_distance: Some(2000),
        });
        let aircraft2 = Arc::new(Aircraft {
            id: 2,
            manufacturer: "Airbus".to_string(),
            variant: "A320".to_string(),
            icao_code: "A320".to_string(),
            flown: 0,
            aircraft_range: 2500,
            category: "A".to_string(),
            cruise_speed: 430,
            date_flown: None,
            takeoff_distance: Some(1800),
        });

        let all_aircraft = vec![aircraft1, aircraft2];

        let airport1 = Arc::new(Airport {
            ID: 1,
            Name: "Amsterdam Airport Schiphol".to_string(),
            ICAO: "EHAM".to_string(),
            PrimaryID: None,
            Latitude: 52.3086,
            Longtitude: 4.7639,
            Elevation: -11,
            TransitionAltitude: Some(10000),
            TransitionLevel: None,
            SpeedLimit: Some(230),
            SpeedLimitAltitude: Some(6000),
        });
        let airport2 = Arc::new(Airport {
            ID: 2,
            Name: "Rotterdam The Hague Airport".to_string(),
            ICAO: "EHRD".to_string(),
            PrimaryID: None,
            Latitude: 51.9561,
            Longtitude: 4.4397,
            Elevation: -13,
            TransitionAltitude: Some(5000),
            TransitionLevel: None,
            SpeedLimit: Some(180),
            SpeedLimitAltitude: Some(4000),
        });
        let all_airports = vec![airport1, airport2];

        let runway1 = Runway {
            ID: 1,
            AirportID: 1,
            Ident: "09".to_string(),
            TrueHeading: 92.0,
            Length: 10000,
            Width: 45,
            Surface: "Asphalt".to_string(),
            Latitude: 52.3086,
            Longtitude: 4.7639,
            Elevation: -11,
        };
        let runway2 = Runway {
            ID: 2,
            AirportID: 2,
            Ident: "06".to_string(),
            TrueHeading: 62.0,
            Length: 10000,
            Width: 45,
            Surface: "Asphalt".to_string(),
            Latitude: 51.9561,
            Longtitude: 4.4397,
            Elevation: -13,
        };

        let mut runway_map = HashMap::new();
        runway_map.insert(1, Arc::new(vec![runway1]));
        runway_map.insert(2, Arc::new(vec![runway2]));

        let spatial_airports = RTree::bulk_load(
            all_airports
                .iter()
                .map(|airport| crate::gui::ui::SpatialAirport {
                    airport: Arc::clone(airport),
                })
                .collect(),
        );

        (all_aircraft, all_airports, runway_map, spatial_airports)
    }

    #[test]
    fn test_generate_random_routes() {
        let (all_aircraft, all_airports, all_runways, spatial_airports) = create_test_data();
        let route_generator = RouteGenerator {
            all_airports,
            all_runways,
            spatial_airports,
        };

        let routes = route_generator.generate_random_routes(&all_aircraft, None);
        assert!(!routes.is_empty());
        for route in routes {
            assert!(route.departure.ID != route.destination.ID);
            assert!(route.route_length != "0");
        }
    }

    #[test]
    fn test_generate_random_not_flown_aircraft_routes() {
        let (all_aircraft, all_airports, all_runways, spatial_airports) = create_test_data();
        let route_generator = RouteGenerator {
            all_airports,
            all_runways,
            spatial_airports,
        };

        let routes =
            route_generator.generate_random_not_flown_aircraft_routes(&all_aircraft, None);
        assert!(!routes.is_empty());
        for route in routes {
            assert!(route.departure.ID != route.destination.ID);
            assert!(route.route_length != "0");
        }
    }

    #[test]
    fn test_generate_random_routes_generic() {
        let (all_aircraft, all_airports, all_runways, spatial_airports) = create_test_data();
        let route_generator = RouteGenerator {
            all_airports,
            all_runways,
            spatial_airports,
        };

        let routes: Vec<ListItemRoute> =
            route_generator.generate_random_routes_generic(&all_aircraft, 50, None);
        assert_eq!(routes.len(), 50);
        for route in routes {
            assert!(route.departure.ID != route.destination.ID);
            assert!(route.route_length != "0");
        }
    }
}
