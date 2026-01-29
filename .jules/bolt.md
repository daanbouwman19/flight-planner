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

## 2026-01-25 - Expanding Rejection Sampling Range
**Learning:** For medium-range spatial queries (500-2000 NM), rejection sampling on the global dataset (with higher retry limit) is significantly faster than R-tree spatial queries (O(1) vs O(N_in_range)). The "gap" between small local queries and large global queries was handling medium ranges inefficiently.
**Action:** Lowered `REJECTION_SAMPLING_THRESHOLD_NM` from 2000 to 500 and increased attempts to 128. This resulted in a 4x speedup (4ms -> 1ms) for mixed-range route generation.

## 2026-01-28 - Pre-calculated Trigonometry for Haversine
**Learning:** `sin`, `cos`, and `to_radians` are expensive in tight loops (like spatial queries or rejection sampling). Pre-calculating these values for static data (airports) and storing them alongside the object can significantly reduce CPU usage during distance checks.
**Action:** Wrap static spatial objects in a `Cached` struct that computes and stores necessary trigonometric values upon initialization.

## 2026-01-29 - Latitude Band Sampling
**Learning:** Rejection Sampling is efficient for global searches (high acceptance), and R-Trees are efficient for tiny local searches (small N). However, for "medium" ranges (200-500 NM), Rejection Sampling fails too often (0.5% acceptance), and R-Trees iterate too many candidates (1000+).
**Action:** Implemented "Latitude Band Sampling": Binary search a pre-sorted latitude index to select a candidate strip (narrowing world area by ~90%), then perform rejection sampling within that strip. This yields a ~10x higher acceptance rate than global sampling and is ~4.5x faster than R-Tree for medium-range route generation.
