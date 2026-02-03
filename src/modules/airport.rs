use crate::database::{DatabaseConnections, DatabasePool};
use crate::errors::AirportSearchError;
#[cfg(feature = "gui")]
use crate::models::airport::{CachedAirport, SpatialAirport};
use crate::models::{Aircraft, Airport, Runway};
use crate::schema::Airports::dsl::{Airports, ID, Latitude, Longtitude};
use crate::traits::{AircraftOperations, AirportOperations};
use crate::util::calculate_haversine_distance_nm;
#[cfg(feature = "gui")]
use crate::util::{calculate_haversine_threshold, check_haversine_within_threshold_cached};
use diesel::prelude::*;
use rand::prelude::*;
#[cfg(feature = "gui")]
use rand::seq::IteratorRandom;
#[cfg(feature = "gui")]
use rstar::{AABB, RTree};
use std::collections::HashMap;
use std::sync::Arc;

const M_TO_FT: f64 = 3.28084;
const MAX_ATTEMPTS: usize = 50;

impl AirportOperations for DatabaseConnections {
    fn get_random_airport(&mut self) -> Result<Airport, AirportSearchError> {
        // Optimization: Use count + offset instead of ORDER BY RANDOM() LIMIT 1.
        // ORDER BY RANDOM() requires scanning and sorting all rows, which is slow for large tables.
        // OFFSET avoids sorting. Benchmark showed ~17x improvement (6.85ms -> 0.39ms) for 20k rows.
        let count: i64 = Airports.count().get_result(&mut self.airport_connection)?;
        if count == 0 {
            return Err(AirportSearchError::NotFound);
        }
        let offset = rand::rng().random_range(0..count);
        let airport: Airport = Airports
            .offset(offset)
            .limit(1)
            .get_result(&mut self.airport_connection)?;

        Ok(airport)
    }

    fn get_destination_airport(
        &mut self,
        aircraft: &Aircraft,
        departure: &Airport,
    ) -> Result<Airport, AirportSearchError> {
        get_destination_airport(self, aircraft, departure)
    }

    fn get_random_airport_for_aircraft(
        &mut self,
        aircraft: &Aircraft,
    ) -> Result<Airport, AirportSearchError> {
        let min_takeoff_distance_m = aircraft.takeoff_distance;
        if min_takeoff_distance_m.is_some() {
            get_random_airport_for_aircraft(&mut self.airport_connection, min_takeoff_distance_m)
        } else {
            self.get_random_airport()
        }
    }

    fn get_runways_for_airport(
        &mut self,
        airport: &Airport,
    ) -> Result<Vec<Runway>, AirportSearchError> {
        use crate::schema::Runways::dsl::{AirportID, Runways};

        let runways = Runways
            .filter(AirportID.eq(airport.ID))
            .load::<Runway>(&mut self.airport_connection)?;

        Ok(runways)
    }

    fn get_destination_airport_with_suitable_runway(
        &mut self,
        departure: &Airport,
        max_distance_nm: i32,
        min_takeoff_distance_m: i32,
    ) -> Result<Airport, AirportSearchError> {
        get_destination_airport_with_suitable_runway(
            &mut self.airport_connection,
            departure,
            max_distance_nm,
            min_takeoff_distance_m,
        )
    }

    fn get_airport_within_distance(
        &mut self,
        departure: &Airport,
        max_distance_nm: i32,
    ) -> Result<Airport, AirportSearchError> {
        get_airport_within_distance(&mut self.airport_connection, departure, max_distance_nm)
    }

    fn get_airports(&mut self) -> Result<Vec<Airport>, AirportSearchError> {
        let airports = Airports.load::<Airport>(&mut self.airport_connection)?;

        Ok(airports)
    }

    fn get_airport_by_icao(&mut self, icao: &str) -> Result<Airport, AirportSearchError> {
        get_airport_by_icao(&mut self.airport_connection, icao)
    }
}

impl AirportOperations for DatabasePool {
    fn get_random_airport(&mut self) -> Result<Airport, AirportSearchError> {
        let conn = &mut self.airport_pool.get()?;
        let count: i64 = Airports.count().get_result(conn)?;
        if count == 0 {
            return Err(AirportSearchError::NotFound);
        }
        let offset = rand::rng().random_range(0..count);
        let airport: Airport = Airports.offset(offset).limit(1).get_result(conn)?;

        Ok(airport)
    }

    fn get_destination_airport(
        &mut self,
        aircraft: &Aircraft,
        departure: &Airport,
    ) -> Result<Airport, AirportSearchError>
    where
        Self: AircraftOperations,
    {
        get_destination_airport(self, aircraft, departure)
    }

    fn get_random_airport_for_aircraft(
        &mut self,
        aircraft: &Aircraft,
    ) -> Result<Airport, AirportSearchError> {
        let min_takeoff_distance_m = aircraft.takeoff_distance;
        if min_takeoff_distance_m.is_some() {
            let conn = &mut self.airport_pool.get()?;
            get_random_airport_for_aircraft(conn, min_takeoff_distance_m)
        } else {
            self.get_random_airport()
        }
    }

    fn get_runways_for_airport(
        &mut self,
        airport: &Airport,
    ) -> Result<Vec<Runway>, AirportSearchError> {
        use crate::schema::Runways::dsl::{AirportID, Runways};
        let conn = &mut self.airport_pool.get()?;

        let runways = Runways
            .filter(AirportID.eq(airport.ID))
            .load::<Runway>(conn)?;

        Ok(runways)
    }

    fn get_destination_airport_with_suitable_runway(
        &mut self,
        departure: &Airport,
        max_distance_nm: i32,
        min_takeoff_distance_m: i32,
    ) -> Result<Airport, AirportSearchError> {
        let conn = &mut self.airport_pool.get()?;
        get_destination_airport_with_suitable_runway(
            conn,
            departure,
            max_distance_nm,
            min_takeoff_distance_m,
        )
    }

    fn get_airport_within_distance(
        &mut self,
        departure: &Airport,
        max_distance_nm: i32,
    ) -> Result<Airport, AirportSearchError> {
        let conn = &mut self.airport_pool.get()?;
        get_airport_within_distance(conn, departure, max_distance_nm)
    }

    fn get_airports(&mut self) -> Result<Vec<Airport>, AirportSearchError> {
        let conn = &mut self.airport_pool.get()?;
        let airports = Airports.load::<Airport>(conn)?;

        Ok(airports)
    }

    fn get_airport_by_icao(&mut self, icao: &str) -> Result<Airport, AirportSearchError> {
        let conn = &mut self.airport_pool.get()?;
        get_airport_by_icao(conn, icao)
    }
}

/// Formats an `Airport` struct into a human-readable string.
///
/// # Arguments
///
/// * `airport` - A reference to the `Airport` struct to format.
///
/// # Returns
///
/// A `String` containing the formatted airport details.
pub fn format_airport(airport: &Airport) -> String {
    format!(
        "{} ({}), altitude: {}",
        airport.Name, airport.ICAO, airport.Elevation
    )
}

fn get_destination_airport<T: AirportOperations>(
    db: &mut T,
    aircraft: &Aircraft,
    departure: &Airport,
) -> Result<Airport, AirportSearchError> {
    let max_aircraft_range_nm = aircraft.aircraft_range;
    let min_takeoff_distance_m = aircraft.takeoff_distance;

    for _ in 0..MAX_ATTEMPTS {
        let result = match min_takeoff_distance_m {
            Some(min_takeoff_distance) => db.get_destination_airport_with_suitable_runway(
                departure,
                max_aircraft_range_nm,
                min_takeoff_distance,
            ),
            None => db.get_airport_within_distance(departure, max_aircraft_range_nm),
        };

        match result {
            Ok(airport) => return Ok(airport),
            Err(AirportSearchError::DistanceExceeded) => continue,
            Err(AirportSearchError::NotFound) => return Err(AirportSearchError::NotFound),
            Err(e) => return Err(e),
        }
    }

    Err(AirportSearchError::NotFound)
}
fn get_destination_airport_with_suitable_runway(
    db: &mut SqliteConnection,
    departure: &Airport,
    max_distance_nm: i32,
    min_takeoff_distance_m: i32,
) -> Result<Airport, AirportSearchError> {
    use crate::schema::Runways;
    let origin_lat = departure.Latitude;
    let origin_lon = departure.Longtitude;

    let max_difference_degrees = f64::from(max_distance_nm) / 60.0;
    let min_lat = origin_lat - max_difference_degrees;
    let max_lat = origin_lat + max_difference_degrees;
    let min_lon = origin_lon - max_difference_degrees;
    let max_lon = origin_lon + max_difference_degrees;

    #[allow(clippy::cast_possible_truncation)]
    let min_takeoff_distance_ft: i32 = (f64::from(min_takeoff_distance_m) * M_TO_FT).round() as i32;

    // Optimization: Use EXISTS + COUNT + OFFSET instead of JOIN + DISTINCT + ORDER BY RANDOM()
    // 1. EXISTS avoids duplicating rows (which inner_join does).
    // 2. COUNT + OFFSET avoids the expensive sort of ORDER BY RANDOM().
    let query = Airports
        .filter(diesel::dsl::exists(
            Runways::table
                .filter(Runways::AirportID.eq(ID))
                .filter(Runways::Length.ge(min_takeoff_distance_ft)),
        ))
        .filter(Latitude.ge(min_lat))
        .filter(Latitude.le(max_lat))
        .filter(Longtitude.ge(min_lon))
        .filter(Longtitude.le(max_lon))
        .filter(ID.ne(departure.ID));

    // Optimization: Count once, then try multiple times to find a match.
    // The query returns airports in a bounding box (square). We need to check exact Haversine distance (circle).
    // The ratio of circle area to square area is pi/4 ~= 0.785.
    // So ~21.5% of airports in the box are outside the circle.
    // With 10 attempts, the probability of failure (only picking corners) is (0.215)^10 ~= 2e-7.
    let count: i64 = query.count().get_result(db)?;

    if count == 0 {
        return Err(AirportSearchError::NotFound);
    }

    for _ in 0..10 {
        let offset = rand::rng().random_range(0..count);
        let airport = query.offset(offset).limit(1).get_result::<Airport>(db)?;

        let distance = calculate_haversine_distance_nm(departure, &airport);

        if distance < max_distance_nm {
            return Ok(airport);
        }
    }

    Err(AirportSearchError::DistanceExceeded)
}

fn get_airport_within_distance(
    db: &mut SqliteConnection,
    departure: &Airport,
    max_distance_nm: i32,
) -> Result<Airport, AirportSearchError> {
    let origin_lat = departure.Latitude;
    let origin_lon = departure.Longtitude;

    let max_difference_degrees = f64::from(max_distance_nm) / 60.0;
    let min_lat = origin_lat - max_difference_degrees;
    let max_lat = origin_lat + max_difference_degrees;
    let min_lon = origin_lon - max_difference_degrees;
    let max_lon = origin_lon + max_difference_degrees;

    let query = Airports
        .filter(Latitude.ge(min_lat))
        .filter(Latitude.le(max_lat))
        .filter(Longtitude.ge(min_lon))
        .filter(Longtitude.le(max_lon))
        .filter(ID.ne(departure.ID));

    // Optimization: Count once, then try multiple times to find a match.
    // The query returns airports in a bounding box (square). We need to check exact Haversine distance (circle).
    // The ratio of circle area to square area is pi/4 ~= 0.785.
    // So ~21.5% of airports in the box are outside the circle.
    // With 10 attempts, the probability of failure (only picking corners) is (0.215)^10 ~= 2e-7.
    let count: i64 = query.count().get_result(db)?;

    if count == 0 {
        return Err(AirportSearchError::NotFound);
    }

    for _ in 0..10 {
        let offset = rand::rng().random_range(0..count);
        let airport = query.offset(offset).limit(1).get_result::<Airport>(db)?;

        let distance = calculate_haversine_distance_nm(departure, &airport);

        if distance < max_distance_nm {
            return Ok(airport);
        }
    }

    Err(AirportSearchError::DistanceExceeded)
}

fn get_random_airport_for_aircraft(
    db: &mut SqliteConnection,
    min_takeoff_distance_m: Option<i32>,
) -> Result<Airport, AirportSearchError> {
    use crate::schema::{Airports::dsl::Airports, Runways};

    if let Some(min_takeoff_distance) = min_takeoff_distance_m {
        #[allow(clippy::cast_possible_truncation)]
        let min_takeoff_distance_ft = (f64::from(min_takeoff_distance) * M_TO_FT).round() as i32;

        // Optimization: Use EXISTS + COUNT + OFFSET instead of JOIN + DISTINCT + ORDER BY RANDOM()
        // 1. EXISTS avoids duplicating rows (which inner_join does), so we don't need distinct().
        // 2. COUNT + OFFSET avoids the expensive sort of ORDER BY RANDOM().
        let query = Airports.filter(diesel::dsl::exists(
            Runways::table
                .filter(Runways::AirportID.eq(ID))
                .filter(Runways::Length.ge(min_takeoff_distance_ft)),
        ));

        let count: i64 = query.count().get_result(db)?;

        if count == 0 {
            return Err(AirportSearchError::NoSuitableRunway);
        }

        let offset = rand::rng().random_range(0..count);
        let airport = query.offset(offset).limit(1).get_result::<Airport>(db)?;

        Ok(airport)
    } else {
        Err(AirportSearchError::NoSuitableRunway)
    }
}

/// Finds a random suitable destination airport for a given aircraft and departure airport.
///
/// This function uses a hybrid approach for optimal performance:
/// 1. For large search ranges (> 2000 NM), it attempts rejection sampling first. This is O(1)
///    expected time and avoids iterating over thousands of candidates in the spatial index.
/// 2. If rejection sampling fails or range is small, it falls back to an R-tree spatial query.
///
/// # Arguments
///
/// * `aircraft` - The aircraft for which to find destinations.
/// * `departure` - The departure airport.
/// * `suitable_airports` - Optional slice of all airports that meet the runway requirement.
/// * `spatial_airports` - An R-tree of all airports for fast spatial queries.
/// * `rng` - Random number generator.
///
/// # Returns
///
/// An `Option` containing a reference to a suitable destination airport, or `None`.
#[cfg(feature = "gui")]
pub fn get_random_destination_airport_fast<'a, R: Rng + ?Sized>(
    aircraft: &'a Aircraft,
    departure: &'a CachedAirport,
    suitable_airports: Option<&'a [CachedAirport]>,
    spatial_airports: &'a RTree<SpatialAirport>,
    rng: &mut R,
) -> Option<&'a CachedAirport> {
    let max_distance_nm = aircraft.aircraft_range;

    // Pre-calculate the Haversine threshold to avoid expensive sqrt/atan2 calls in the loop.
    // This reduces the distance check to a few trig ops and a comparison.
    let distance_threshold = calculate_haversine_threshold(max_distance_nm);

    #[allow(clippy::cast_possible_truncation)]
    let takeoff_distance_ft: Option<i32> = aircraft
        .takeoff_distance
        .map(|d| (f64::from(d) * M_TO_FT).round() as i32);

    // HYBRID STRATEGY:
    // If we have a list of suitable candidates (runway-filtered) and the search radius
    // is large (> 500 NM), the probability of a random candidate being in range is high.
    // Rejection sampling is much faster here (O(1) vs O(N_in_range) for spatial query iteration).
    //
    // Updated optimization (Bolt): Lowered threshold to 500 NM to cover medium-range aircraft.
    // Increased attempts to 128 to maintain high success rate with the expanded (lower probability) range.
    const REJECTION_SAMPLING_THRESHOLD_NM: i32 = 500;
    const REJECTION_SAMPLING_ATTEMPTS: usize = 128;

    if max_distance_nm >= REJECTION_SAMPLING_THRESHOLD_NM
        && let Some(candidates) = suitable_airports
        && !candidates.is_empty()
    {
        // For cached comparison, we rely on f32 radians.
        // We can do a quick lat check using radians if we want, but check_haversine_within_threshold_cached is fast enough.
        // But the lat_diff optimization was using f64 degrees. Let's adapt it to f32 radians for consistency and speed.
        let max_lat_diff_rad = (f64::from(max_distance_nm) / 60.0).to_radians() as f32;

        for _ in 0..REJECTION_SAMPLING_ATTEMPTS {
            // Pick a random airport from the pre-filtered list (guaranteed to meet runway reqs if bucket is strict,
            // or we check it below to be safe)
            if let Some(candidate) = candidates.choose(rng) {
                if candidate.inner.ID == departure.inner.ID {
                    continue;
                }

                // Quick check for latitude difference to avoid expensive Haversine
                // This optimization skips approximately 30-40% of Haversine calls for global candidates.
                // Using f32 radians directly.
                let lat_diff = (departure.lat_rad - candidate.lat_rad).abs();
                if lat_diff > max_lat_diff_rad {
                    continue;
                }

                // Check distance using optimized threshold check with cached values
                if check_haversine_within_threshold_cached(departure, candidate, distance_threshold)
                {
                    return Some(candidate);
                }
            }
        }
    }

    // FALLBACK: Spatial Query
    // Best for small ranges where rejection sampling would miss frequently.
    let search_radius_deg = f64::from(max_distance_nm) / 60.0;

    let min_point = [
        departure.inner.Latitude - search_radius_deg,
        departure.inner.Longtitude - search_radius_deg,
    ];
    let max_point = [
        departure.inner.Latitude + search_radius_deg,
        departure.inner.Longtitude + search_radius_deg,
    ];
    let search_envelope = AABB::from_corners(min_point, max_point);

    // Direct iterator choice - avoids allocating Vec of thousands of candidates
    spatial_airports
        .locate_in_envelope(&search_envelope)
        .filter_map(move |spatial_airport| {
            let candidate_cached = &spatial_airport.airport;
            if candidate_cached.inner.ID == departure.inner.ID {
                return None;
            }

            // Quick runway check using pre-computed data directly from spatial index
            // Optimization: Avoids HashMap lookup for each candidate in the loop
            let max_len = spatial_airport.longest_runway_length;

            let has_suitable_runway = match takeoff_distance_ft {
                Some(required_distance) => max_len >= required_distance,
                None => max_len > 0,
            };

            if !has_suitable_runway {
                return None;
            }

            // Verify actual Haversine distance.
            // Spatial query envelope is a square box, so corners are further than search_radius.
            // Also, longitude degrees shrink with latitude, so the box might be inaccurate in longitude.
            if !check_haversine_within_threshold_cached(
                departure,
                candidate_cached,
                distance_threshold,
            ) {
                return None;
            }

            Some(candidate_cached)
        })
        .choose(rng)
}

fn get_airport_by_icao(
    db: &mut SqliteConnection,
    icao: &str,
) -> Result<Airport, AirportSearchError> {
    use crate::schema::Airports::dsl::ICAO;

    let airport = Airports.filter(ICAO.eq(icao)).first::<Airport>(db)?;
    Ok(airport)
}

pub fn get_airport_with_suitable_runway_fast<'a, R: Rng + ?Sized>(
    aircraft: &'a Aircraft,
    all_airports: &'a [Arc<Airport>],
    runways_by_airport: &'a HashMap<i32, Arc<Vec<Runway>>>,
    rng: &mut R,
) -> Result<Arc<Airport>, AirportSearchError> {
    for _ in 0..MAX_ATTEMPTS {
        let Some(airport) = all_airports.choose(rng) else {
            continue;
        };

        let Some(runways) = runways_by_airport.get(&airport.ID) else {
            log::warn!("No runway data found for airport ID: {}", airport.ID);
            continue;
        };

        let Some(longest_runway) = runways.iter().max_by_key(|r: &&Runway| r.Length) else {
            log::warn!("Empty runway list for airport ID: {}", airport.ID);
            continue;
        };

        if aircraft.takeoff_distance.is_none_or(|takeoff_distance_m| {
            #[allow(clippy::cast_possible_truncation)]
            let required_distance_ft = (f64::from(takeoff_distance_m) * M_TO_FT).round() as i32;
            longest_runway.Length >= required_distance_ft
        }) {
            return Ok(Arc::clone(airport));
        }
    }
    Err(AirportSearchError::NotFound)
}
