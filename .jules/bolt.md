## 2026-02-21 - Cached Display Strings in Rust
**Learning:** Pre-formatting and caching display strings in long-lived structures (like `CachedAirport`) significantly reduces allocations during high-frequency operations (like route generation), especially when using `Arc<String>`.
**Action:** Look for other frequently accessed computed properties that can be cached in `Cached*` structs.
