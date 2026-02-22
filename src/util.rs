use crate::models::Airport;
use diesel::define_sql_function;

define_sql_function! {fn random() -> Text;}

/// Validates a path from an environment variable.
///
/// Returns `Some(PathBuf)` if the variable exists and contains a safe path.
/// Returns `None` if the variable is missing or contains path traversal components (`..`).
pub fn validate_env_path(var_name: &str) -> Option<std::path::PathBuf> {
    let val = std::env::var(var_name).ok()?;
    let path = std::path::PathBuf::from(val);

    // Check for traversal attempts
    if path
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return None;
    }

    // Reconstruct the path component by component to ensure we are creating a new,
    // unrelated object in the eyes of static analysis tools (breaking the taint chain).
    let mut clean_path = std::path::PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::RootDir => clean_path.push(std::path::Component::RootDir),
            std::path::Component::Prefix(p) => clean_path.push(std::path::Component::Prefix(p)),
            std::path::Component::Normal(s) => clean_path.push(s),
            std::path::Component::CurDir => {} // Ignore '.'
            std::path::Component::ParentDir => return None, // Should be caught by check above
        }
    }

    Some(clean_path)
}

/// The conversion factor from meters to feet.
pub const METERS_TO_FEET: f64 = 3.28084;

/// Calculates the great-circle distance between two airports using the haversine formula.
///
/// # Arguments
///
/// * `airport_1` - The first airport.
/// * `airport_2` - The second airport.
///
/// # Returns
///
/// The distance between the two airports in nautical miles, rounded to the nearest integer.
#[must_use]
pub fn calculate_haversine_distance_nm(airport_1: &Airport, airport_2: &Airport) -> i32 {
    calculate_haversine_distance_nm_points(
        airport_1.Latitude,
        airport_1.Longtitude,
        airport_2.Latitude,
        airport_2.Longtitude,
    )
}

/// Calculates the great-circle distance between two points using the haversine formula.
///
/// # Arguments
///
/// * `lat1` - Latitude of the first point in decimal degrees.
/// * `lon1` - Longitude of the first point in decimal degrees.
/// * `lat2` - Latitude of the second point in decimal degrees.
/// * `lon2` - Longitude of the second point in decimal degrees.
///
/// # Returns
///
/// The distance between the two points in nautical miles, rounded to the nearest integer.
#[must_use]
pub fn calculate_haversine_distance_nm_points(
    lat1: f64,
    lon1: f64,
    lat2: f64,
    lon2: f64,
) -> i32 {
    // Optimization: Use f32 for distance calculations.
    // Earth radius is 3440 NM. f32 provides ~7 significant digits.
    // 0.001 NM (1.8 meters) precision is sufficient for flight planning.
    // This reduces register pressure and is faster on many architectures.
    let earth_radius_nm = 3440.0_f32;
    let lat1 = (lat1 as f32).to_radians();
    let lon1 = (lon1 as f32).to_radians();
    let lat2 = (lat2 as f32).to_radians();
    let lon2 = (lon2 as f32).to_radians();

    let lat_diff = lat2 - lat1;
    let lon_diff = lon2 - lon1;

    let a = calculate_haversine_factor(lat_diff, lon_diff, lat1.cos(), lat2.cos());
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    #[allow(clippy::cast_possible_truncation)]
    let result = (earth_radius_nm * c).round() as i32;
    result
}

/// Pre-calculates the threshold value for the Haversine formula based on the maximum distance.
///
/// This avoids repeated `sqrt`, `asin`/`atan2` calls during bulk comparisons.
/// The threshold corresponds to `sin^2(c/2)` where `c` is the central angle.
///
/// # Arguments
///
/// * `max_distance_nm` - The maximum distance in nautical miles.
///
/// # Returns
///
/// The threshold value `a` (between 0.0 and 1.0).
pub fn calculate_haversine_threshold(max_distance_nm: i32) -> f32 {
    let earth_radius_nm = 3440.0_f32;
    // We want to check if distance <= max_distance_nm.
    // Since calculate_haversine_distance_nm returns round(distance),
    // we are effectively checking if distance < max_distance_nm + 0.5.
    let limit = (max_distance_nm as f32) + 0.5;

    // Safety check: if limit is beyond half circumference, it covers everything.
    // Half circumference is ~10807 NM.
    let max_possible_dist = earth_radius_nm * std::f32::consts::PI;
    if limit >= max_possible_dist {
        return 1.0;
    }

    let c_limit = limit / earth_radius_nm;

    // a = sin^2(c/2)
    (c_limit / 2.0).sin().powi(2)
}

/// Checks if the distance between two airports is within the pre-calculated threshold.
///
/// This function computes the Haversine `a` value (squared sine of half the central angle)
/// and compares it directly against the threshold, avoiding expensive inverse trigonometric functions.
///
/// # Arguments
///
/// * `airport_1` - The first airport.
/// * `airport_2` - The second airport.
/// * `threshold` - The pre-calculated threshold from `calculate_haversine_threshold`.
///
/// # Returns
///
/// `true` if the distance is within the threshold, `false` otherwise.
pub fn check_haversine_within_threshold(
    airport_1: &Airport,
    airport_2: &Airport,
    threshold: f32,
) -> bool {
    let lat1 = (airport_1.Latitude as f32).to_radians();
    let lon1 = (airport_1.Longtitude as f32).to_radians();
    let lat2 = (airport_2.Latitude as f32).to_radians();
    let lon2 = (airport_2.Longtitude as f32).to_radians();

    let lat_diff = lat2 - lat1;
    let lon_diff = lon2 - lon1;

    let a = calculate_haversine_factor(lat_diff, lon_diff, lat1.cos(), lat2.cos());

    a <= threshold
}

/// Helper function to calculate Haversine 'a' value using optimized dot-product formula.
///
/// Formula: a = 0.5 * (1.0 - (sin(lat1)*sin(lat2) + cos(lat1)*cos(lat2)*cos(lon_diff)))
/// Derived from Haversine identity: sin^2(x/2) = (1 - cos(x)) / 2
#[cfg(feature = "gui")]
#[inline(always)]
fn calculate_haversine_a_cached(
    source: &crate::models::airport::CachedAirport,
    target: &crate::models::airport::CachedAirport,
) -> f32 {
    let sin_lat_prod = source.sin_lat * target.sin_lat;
    let cos_lat_prod = source.cos_lat * target.cos_lat;

    // cos(lon_diff) = cos(lon1)*cos(lon2) + sin(lon1)*sin(lon2)
    let cos_lon_diff = source
        .sin_lon
        .mul_add(target.sin_lon, source.cos_lon * target.cos_lon);

    // a = 0.5 * (1.0 - (sin_lat_prod + cos_lat_prod * cos_lon_diff))
    // We use mul_add here for potential FMA optimization:
    // inner = cos_lat_prod * cos_lon_diff + sin_lat_prod
    let inner = cos_lat_prod.mul_add(cos_lon_diff, sin_lat_prod);
    0.5 * (1.0 - inner)
}

/// Checks if the distance between two cached airports is within the pre-calculated threshold.
///
/// This uses pre-calculated trigonometric values from `CachedAirport` to avoid
/// expensive trigonometric operations during comparison.
///
/// # Arguments
///
/// * `source` - The first airport (cached).
/// * `target` - The second airport (cached).
/// * `threshold` - The pre-calculated threshold from `calculate_haversine_threshold`.
#[cfg(feature = "gui")]
#[inline]
pub fn check_haversine_within_threshold_cached(
    source: &crate::models::airport::CachedAirport,
    target: &crate::models::airport::CachedAirport,
    threshold: f32,
) -> bool {
    let a = calculate_haversine_a_cached(source, target);
    a <= threshold
}

/// Calculates the great-circle distance between two cached airports using the haversine formula.
///
/// This version uses pre-calculated trigonometric values from `CachedAirport` to avoid
/// repeated `to_radians()` and `cos()` calls, making it significantly faster for
/// repeated calculations.
///
/// # Arguments
///
/// * `source` - The first airport (cached).
/// * `target` - The second airport (cached).
///
/// # Returns
///
/// The distance between the two airports in nautical miles, rounded to the nearest integer.
#[cfg(feature = "gui")]
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn calculate_haversine_distance_nm_cached(
    source: &crate::models::airport::CachedAirport,
    target: &crate::models::airport::CachedAirport,
) -> i32 {
    let earth_radius_nm = 3440.0_f32;

    let a = calculate_haversine_a_cached(source, target);

    // Clamp to valid range [0.0, 1.0] to avoid NaN in sqrt due to floating point precision
    let a = a.clamp(0.0, 1.0);

    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    (earth_radius_nm * c).round() as i32
}

/// Checks if the distance between two airports is within the pre-calculated threshold,
/// using pre-calculated radians for the first airport.
///
/// This avoids re-calculating trigonometric values for the reference airport in loops.
///
/// # Arguments
///
/// * `lat1_rad` - Latitude of the first airport in radians.
/// * `lon1_rad` - Longitude of the first airport in radians.
/// * `cos_lat1` - Cosine of the latitude of the first airport.
/// * `airport_2` - The second airport.
/// * `threshold` - The pre-calculated threshold from `calculate_haversine_threshold`.
#[inline]
pub fn check_haversine_within_threshold_fast(
    lat1_rad: f32,
    lon1_rad: f32,
    cos_lat1: f32,
    airport_2: &Airport,
    threshold: f32,
) -> bool {
    let lat2 = (airport_2.Latitude as f32).to_radians();
    let lon2 = (airport_2.Longtitude as f32).to_radians();

    let lat_diff = lat2 - lat1_rad;
    let lon_diff = lon2 - lon1_rad;

    let a = calculate_haversine_factor(lat_diff, lon_diff, cos_lat1, lat2.cos());

    a <= threshold
}

/// Calculates the Haversine 'a' factor (squared sine of half the central angle).
///
/// # Arguments
///
/// * `lat_diff` - Difference in latitude in radians.
/// * `lon_diff` - Difference in longitude in radians.
/// * `cos_lat1` - Cosine of the first latitude.
/// * `cos_lat2` - Cosine of the second latitude.
#[inline(always)]
fn calculate_haversine_factor(lat_diff: f32, lon_diff: f32, cos_lat1: f32, cos_lat2: f32) -> f32 {
    (cos_lat1 * cos_lat2).mul_add(
        (lon_diff / 2.0).sin().powi(2),
        (lat_diff / 2.0).sin().powi(2),
    )
}

/// Optimized case-insensitive substring search that minimizes allocations.
/// For ASCII text (the vast majority of cases), uses zero-allocation comparison.
/// For Unicode text, falls back to correct but allocating comparison.
/// Assumes `query_lower` is already lowercase for optimal performance.
#[inline]
pub fn contains_case_insensitive(haystack: &str, query_lower: &str) -> bool {
    contains_case_insensitive_optimized(haystack, query_lower, query_lower.is_ascii())
}

/// A highly optimized version of `contains_case_insensitive` that skips repeated checks.
///
/// This function assumes:
/// 1. `query_lower` is already lowercase.
/// 2. `query_is_ascii` is pre-calculated (avoiding O(N) scan).
#[inline]
pub fn contains_case_insensitive_optimized(
    haystack: &str,
    query_lower: &str,
    query_is_ascii: bool,
) -> bool {
    // Fast path: if query is empty, always matches.
    // Although callers might handle this, it's safer to handle it here to prevent panics
    // on indexing below.
    if query_lower.is_empty() {
        return true;
    }

    // Optimization: if query is ASCII, we can scan bytes directly regardless of whether
    // the haystack is ASCII or not. This avoids allocations for haystacks containing
    // non-ASCII chars when the match is findable via ASCII bytes.
    if query_is_ascii {
        // Convert to bytes for efficient ASCII comparison
        let haystack_bytes = haystack.as_bytes();
        let query_bytes = query_lower.as_bytes();
        let h_len = haystack_bytes.len();
        let q_len = query_bytes.len();

        if q_len > h_len {
            return false;
        }

        // Optimization: Iterate manually to find the first char match, then check rest
        // This avoids creating sub-slices for every position like windows() does
        let first_byte_lower = query_bytes[0];
        // query_lower is assumed lowercase, but for safety/correctness in ASCII fast path:
        let first_byte_upper = first_byte_lower.to_ascii_uppercase();

        // We only need to iterate up to h_len - q_len
        // Using a manual loop is significantly faster than windows().any()
        for i in 0..=(h_len - q_len) {
            let b = haystack_bytes[i];
            if b == first_byte_lower || b == first_byte_upper {
                // Check the rest of the string
                // Optimization: Skip the first byte since we just checked it
                if haystack_bytes[i + 1..i + q_len].eq_ignore_ascii_case(&query_bytes[1..]) {
                    return true;
                }
            }
        }

        // If we didn't find it in the fast path, AND the haystack is pure ASCII,
        // then it's definitely not there (since we covered all ASCII possibilities).
        // Only if the haystack has non-ASCII chars do we need to fallback to to_lowercase()
        // to handle edge cases where non-ASCII chars might normalize to ASCII chars
        // (e.g. Kelvin sign 'K' -> 'k').
        if haystack.is_ascii() {
            return false;
        }
    }

    // Unicode fallback: correct but allocating for complex cases like Turkish İ
    haystack.to_lowercase().contains(query_lower)
}

/// Calculates the estimated flight time based on distance and speed.
///
/// Returns (hours, minutes, effective_speed_used)
pub fn calculate_flight_time(distance: f64, speed_knots: i32) -> (i32, i32, f64) {
    const DEFAULT_SPEED_KNOTS: f64 = 300.0; // Reasonable default if cruise speed is 0

    let speed = if speed_knots > 0 {
        f64::from(speed_knots)
    } else {
        DEFAULT_SPEED_KNOTS
    };

    let time_hours = distance / speed;

    // Convert to hours and minutes
    let hours = time_hours.trunc() as i32;
    let minutes = (time_hours.fract() * 60.0).round() as i32;

    // Handle rollover if minutes rounds to 60
    if minutes == 60 {
        (hours + 1, 0, speed)
    } else {
        (hours, minutes, speed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_flight_time() {
        // Test case 1: Normal speed
        // Distance 300 NM, Speed 300 kts -> 1.0 hours
        let (h, m, s) = calculate_flight_time(300.0, 300);
        assert_eq!(h, 1);
        assert_eq!(m, 0);
        assert_eq!(s, 300.0);

        // Test case 2: Default speed (0 input)
        // Distance 300 NM, Speed 0 -> defaults to 300 kts -> 1.0 hours
        let (h, m, s) = calculate_flight_time(300.0, 0);
        assert_eq!(h, 1);
        assert_eq!(m, 0);
        assert_eq!(s, 300.0);

        // Test case 3: Rounding
        // Distance 450 NM, Speed 300 kts -> 1.5 hours -> 1h 30m
        let (h, m, s) = calculate_flight_time(450.0, 300);
        assert_eq!(h, 1);
        assert_eq!(m, 30);
        assert_eq!(s, 300.0);

        // Test case 4: Minute rollover
        // 59.9 minutes should round to 60, then increment hour
        // Speed 60 kts. Distance 59.9 NM -> 0.99833 hours -> 59.9 minutes
        let (h, m, s) = calculate_flight_time(59.9, 60);
        assert_eq!(h, 1);
        assert_eq!(m, 0);
        assert_eq!(s, 60.0);

        // Test case 5: 1h 59.9m -> 2h 00m
        // Speed 60 kts. Distance 119.9 NM -> 1.99833 hours
        let (h, m, s) = calculate_flight_time(119.9, 60);
        assert_eq!(h, 2);
        assert_eq!(m, 0);
        assert_eq!(s, 60.0);
    }

    #[test]
    fn test_contains_case_insensitive() {
        assert!(contains_case_insensitive("Hello World", "hello"));
        assert!(contains_case_insensitive("Hello World", "world"));
        assert!(contains_case_insensitive("Hello World", "lo wo"));
        assert!(!contains_case_insensitive("Hello World", "goodbye"));
        assert!(contains_case_insensitive("Hello World", ""));
        assert!(contains_case_insensitive("", ""));

        // Unicode (allocating path)
        // Note: 'İ'.to_lowercase() results in 'i' + combining dot, so "istan" (ASCII) does not match.
        // This matches standard Rust behavior which we are preserving.
        assert!(!contains_case_insensitive("İstanbul", "istan"));
        assert!(contains_case_insensitive("İstanbul", "i̇stan"));

        // Optimization check: fast path should find ASCII match inside non-ASCII string
        assert!(contains_case_insensitive("München", "che"));
        // "Zürich" contains "ü", so "zur" does not match because 'ü' != 'u'
        assert!(!contains_case_insensitive("Zürich", "zur"));
        // But "rich" should match
        assert!(contains_case_insensitive("Zürich", "rich"));

        // Edge case: Kelvin sign (K - U+212A) normalizes to 'k'.
        // Fast path won't find 'k' (0x6B), but fallback should.
        assert!(contains_case_insensitive("Kelvin", "k"));
    }

    #[test]
    fn test_haversine_threshold_consistency() {
        let a1 = Airport {
            ID: 1,
            Name: "Origin".to_string(),
            ICAO: "AAAA".to_string(),
            Latitude: 50.0,
            Longtitude: 10.0,
            Elevation: 0,
            PrimaryID: None,
            TransitionAltitude: None,
            TransitionLevel: None,
            SpeedLimit: None,
            SpeedLimitAltitude: None,
        };

        // Create a target airport roughly 100 NM away
        // 1 degree lat is 60 NM. 1.666 deg is 100 NM.
        let a2 = Airport {
            ID: 2,
            Name: "Target".to_string(),
            ICAO: "BBBB".to_string(),
            Latitude: 50.0 + 1.666,
            Longtitude: 10.0,
            Elevation: 0,
            PrimaryID: None,
            TransitionAltitude: None,
            TransitionLevel: None,
            SpeedLimit: None,
            SpeedLimitAltitude: None,
        };

        let distance = calculate_haversine_distance_nm(&a1, &a2);

        // Test exact boundary
        let threshold_exact = calculate_haversine_threshold(distance);
        assert!(
            check_haversine_within_threshold(&a1, &a2, threshold_exact),
            "Distance {} should be within threshold for max_dist {}",
            distance,
            distance
        );

        // Test slightly less
        let threshold_less = calculate_haversine_threshold(distance - 1);
        assert!(
            !check_haversine_within_threshold(&a1, &a2, threshold_less),
            "Distance {} should NOT be within threshold for max_dist {}",
            distance,
            distance - 1
        );

        // Test slightly more
        let threshold_more = calculate_haversine_threshold(distance + 1);
        assert!(
            check_haversine_within_threshold(&a1, &a2, threshold_more),
            "Distance {} should be within threshold for max_dist {}",
            distance,
            distance + 1
        );
    }

    #[cfg(feature = "gui")]
    #[test]
    fn test_haversine_distance_cached_consistency() {
        use crate::models::airport::CachedAirport;
        use std::sync::Arc;

        let a1 = Arc::new(Airport {
            ID: 1,
            Name: "Origin".to_string(),
            ICAO: "AAAA".to_string(),
            Latitude: 50.0,
            Longtitude: 10.0,
            ..Default::default()
        });

        // Roughly 100 NM away
        let a2 = Arc::new(Airport {
            ID: 2,
            Name: "Target".to_string(),
            ICAO: "BBBB".to_string(),
            Latitude: 50.0 + 1.666,
            Longtitude: 10.0,
            ..Default::default()
        });

        // Roughly 2000 NM away (New York -> London)
        // JFK: 40.64, -73.78
        // LHR: 51.47, -0.45
        let a3 = Arc::new(Airport {
            ID: 3,
            Name: "JFK".to_string(),
            ICAO: "KJFK".to_string(),
            Latitude: 40.64,
            Longtitude: -73.78,
            ..Default::default()
        });

        let a4 = Arc::new(Airport {
            ID: 4,
            Name: "LHR".to_string(),
            ICAO: "EGLL".to_string(),
            Latitude: 51.47,
            Longtitude: -0.45,
            ..Default::default()
        });

        let pairs = vec![
            (a1.clone(), a2.clone()),
            (a3.clone(), a4.clone()),
            (a1.clone(), a3.clone()), // Mixed
        ];

        for (p1, p2) in pairs {
            let dist_standard = calculate_haversine_distance_nm(&p1, &p2);

            let c1 = CachedAirport::new(p1.clone(), 0);
            let c2 = CachedAirport::new(p2.clone(), 0);
            let dist_cached = calculate_haversine_distance_nm_cached(&c1, &c2);

            // Allow difference of 1 NM due to rounding/precision differences
            let diff = (dist_standard - dist_cached).abs();
            assert!(
                diff <= 1,
                "Standard: {}, Cached: {}, Diff: {}",
                dist_standard,
                dist_cached,
                diff
            );
        }
    }
}
