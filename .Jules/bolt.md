## 2024-05-23 - Reservoir Sampling vs. Collect-then-Choose
**Learning:** When selecting a single random item from a large iterator (e.g., filtered results), collecting pointers into a `Vec` and picking one index is often faster than using `IteratorRandom::choose` (reservoir sampling). Reservoir sampling calls the RNG for every element in the iterator, whereas collecting allocates once and calls RNG once. If allocation is cheap (pointers) and the iterator is large, `Vec` wins.
**Action:** Use `collect::<Vec<_>>().choose(rng)` instead of `IteratorRandom::choose(rng)` when selecting a single item from a large, cheap-to-move iterator, unless memory is strictly constrained.

## 2024-05-23 - Data Locality and Indirection in Hot Loops
**Learning:** In the route generation hot loop, accessing `runways_by_airport` (`HashMap<i32, Arc<Vec<Runway>>>`) involved double indirection and cache-unfriendly traversal of Runway structs. Replacing this with a pre-computed `longest_runway_cache` (`HashMap<i32, i32>`) reduced execution time by ~65%.
**Action:** Pre-compute minimal necessary data (like scalars) for hot loops instead of traversing complex object graphs, even if it requires a dedicated cache.
