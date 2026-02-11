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

## 2026-02-05 - Batching Random DB Fetches
**Learning:** To pick "random" rows from a large dataset filtered by a query, using `ORDER BY RANDOM()` is slow. Using repeated `OFFSET random()` queries inside a loop creates an N+1 performance bottleneck.
**Action:** Instead of N queries for 1 item each, perform 1 query for N items by picking a random start offset and fetching a block. This reduces DB roundtrips by N-1 (e.g. 90% reduction for N=10), significantly improving latency for "random" selection features.

## 2026-02-04 - Startup Cache vs Runtime Compute
**Learning:** Pre-calculating and caching formatted strings for the entire dataset (40k+ items) to save allocations during runtime operations is a poor tradeoff if the runtime operation only accesses a tiny fraction of the data. It significantly bloats memory and increases startup time.
**Action:** Removed `airport_display_cache` and switched to on-demand formatting. This saved ~2MB of RAM and reduced startup time by ~10% (from ~270ms to ~250ms for 100k items) while adding negligible cost to route generation.

## 2026-02-05 - Trig-Free Haversine Threshold Check
**Learning:** Even with pre-calculated cosines, calling `sin().powi(2)` in the Haversine formula is expensive inside tight loops. Using the dot product of pre-calculated sine/cosine vectors allows computing `sin^2(diff/2)` using only multiplications and additions (`0.5 * (1 - dot_product)`).
**Action:** Expanded `CachedAirport` to store `sin_lat`, `sin_lon`, `cos_lon` and replaced the Haversine threshold check with a dot-product formula. This yielded an 8% speedup in route generation.

## 2026-02-07 - Redundant HashMap Lookups in Sort/Map
**Learning:** When sorting or mapping a collection of objects that already contain a cached value (e.g. `longest_runway_length`), avoid looking up that value in an external `HashMap` inside the closure. Direct field access is O(1) and significantly faster than hashing, especially in tight loops or large collections.
**Action:** Replaced `longest_runway_cache.get(...)` with `a.longest_runway_length` in `RouteGenerator` initialization, avoiding ~80k hash map lookups.

## 2026-02-09 - Caching Repeated Derived Properties in Route Generation
**Learning:** When repeatedly picking random items (aircraft) from a small subset to generate many outputs (routes), re-computing derived properties (binary search indices, formatted strings) is redundant and costly. Pre-calculating these properties for the specific subset avoids O(N*logM) binary searches and O(N) allocations.
**Action:** Implemented `CandidateAircraft` struct to pre-calculate `start_idx` and `aircraft_info` when generating multiple routes from a small aircraft list, replacing repeated binary searches and string formatting with cheap pointer copies.
