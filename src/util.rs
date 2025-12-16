use crate::models::Airport;
use diesel::define_sql_function;

define_sql_function! {fn random() -> Text;}

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
    let earth_radius_nm = 3440.0;
    let lat1 = airport_1.Latitude.to_radians();
    let lon1 = airport_1.Longtitude.to_radians();
    let lat2 = airport_2.Latitude.to_radians();
    let lon2 = airport_2.Longtitude.to_radians();

    let lat = lat2 - lat1;
    let lon = lon2 - lon1;

    let a = (lat1.cos() * lat2.cos()).mul_add((lon / 2.0).sin().powi(2), (lat / 2.0).sin().powi(2));
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    #[allow(clippy::cast_possible_truncation)]
    return (earth_radius_nm * c).round() as i32;
}

/// Optimized case-insensitive substring search that minimizes allocations.
/// For ASCII text (the vast majority of cases), uses zero-allocation comparison.
/// For Unicode text, falls back to correct but allocating comparison.
/// Assumes `query_lower` is already lowercase for optimal performance.
pub fn contains_case_insensitive(haystack: &str, query_lower: &str) -> bool {
    // Fast path: if query is empty, always matches
    if query_lower.is_empty() {
        return true;
    }

    // Optimization: if both haystack and query are pure ASCII, use fast non-allocating path
    if haystack.is_ascii() && query_lower.is_ascii() {
        // Convert to bytes for efficient ASCII comparison
        let haystack_bytes = haystack.as_bytes();
        let query_bytes = query_lower.as_bytes();

        if query_bytes.len() > haystack_bytes.len() {
            return false;
        }

        // Idiomatic sliding window search using `windows` and `eq_ignore_ascii_case`
        haystack_bytes
            .windows(query_bytes.len())
            .any(|window| window.eq_ignore_ascii_case(query_bytes))
    } else {
        // Unicode fallback: correct but allocating for complex cases like Turkish İ
        haystack.to_lowercase().contains(query_lower)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    }
}
