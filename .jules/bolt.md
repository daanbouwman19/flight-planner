## 2024-05-24 - Zero-Allocation Search in UTF-8 Strings
**Learning:** Checking `is_ascii()` on a haystack is O(N) and redundant if checking for an ASCII needle via byte comparison. An ASCII byte in a query can never match a UTF-8 continuation byte or a leading byte of a multi-byte character.
**Action:** When searching for an ASCII needle in a potentially non-ASCII haystack, skip `haystack.is_ascii()` and run the byte-scan loop directly. Only fall back to `to_lowercase()` (allocation) if the byte scan fails AND the haystack contains non-ASCII characters (to handle edge cases like Kelvin sign 'K' -> 'k').

## 2026-01-14 - Loop Fusion and Iterator Parity
**Learning:** When manually fusing `min_by_key` and `max_by_key` into a single loop for O(N) efficiency, remember that `min_by` typically takes the *first* match (`<`) while `max_by` takes the *last* match (`>=`) in Rust's standard library. Failing to respect this can break tie-breaking logic.
**Action:** Use strict inequality for min and inclusive inequality for max when manually unrolling iterator logic.

## 2026-01-17 - Hybrid Spatial Search vs Rejection Sampling
**Learning:** For "find random item within large radius" queries, rejection sampling on the global dataset is often O(1) expected time and significantly faster than spatial index queries (which involve traversing the tree and iterating results, O(N_in_range)).
**Action:** Implemented a hybrid approach: Use rejection sampling when search radius is large (>2000 NM), falling back to R-tree spatial query for small radii or if rejection sampling fails.

## 2026-01-24 - Haversine Threshold Optimization
**Learning:** Computing the exact Haversine distance (`2 * atan2(sqrt(a), sqrt(1-a)) * R`) is expensive inside tight loops due to `atan2`, `sqrt`, and multiplications. Since the haversine term `a = sin^2(c/2)` is monotonic with distance, we can pre-calculate the threshold for `a` and avoid the inverse trig functions during bulk comparisons.
**Action:** Pre-calculate `sin^2(max_dist/2R)` once, then compare `a <= threshold` in the loop. This saves ~30% per iteration in spatial query filters.
