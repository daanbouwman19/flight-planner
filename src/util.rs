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
#[inline]
pub fn contains_case_insensitive(haystack: &str, query_lower: &str) -> bool {
    // Fast path: if query is empty, always matches
    if query_lower.is_empty() {
        return true;
    }

    // Optimization: if query is ASCII, we can scan bytes directly regardless of whether
    // the haystack is ASCII or not. This avoids allocations for haystacks containing
    // non-ASCII chars when the match is findable via ASCII bytes.
    if query_lower.is_ascii() {
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
                if haystack_bytes[i..i + q_len].eq_ignore_ascii_case(query_bytes) {
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
}
