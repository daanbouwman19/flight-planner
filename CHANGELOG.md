# Changelog

## [v1.0.13](https://github.com/daanbouwman19/flight-planner/compare/v1.0.12...v1.0.13)

### Added
- **Performance Benchmark:** Comprehensive benchmark added under `examples/benchmark.rs` for measuring search and route generation performance.
- **New Tests:**
  - `tests/route_generation_correctness_test.rs`: Verifies runway length correctness in route generation.
  - `tests/search_integration_tests.rs`: Integration tests for search refactor.
  - `tests/table_items_tests.rs`: Extensive tests for Unicode/case-insensitive search.
- **Optimized Search:**
  - Parallel search processing for large datasets using Rayon.
  - Instant search for very short queries (≤2 chars).
  - Result limits (`MAX_SEARCH_RESULTS`) and parallelization threshold for UI performance.

### Changed
- **Search Algorithm:**
  - Refactored to use a non-allocating, Unicode-aware case-insensitive substring search (`contains_case_insensitive`).
  - `TableItem` search logic split into optimized and regular paths.
  - Debouncing duration reduced from 300ms → 50ms.

- **Route Generation:**
  - `RouteGenerator` now uses cached runway lengths and indexed airports for fast selection.
  - Improved correctness: ensures both departure and destination airports meet aircraft runway requirements.

- **AppService Construction:**
  - Now uses `RouteGenerator::new` with optimized caches.

- **UI Search Handling:**
  - Instant search for short queries, standard debouncing otherwise.
  - Improved event handling for search and clear search actions.

### Fixed
- **Unicode Case Sensitivity:**
  - Search now correctly matches airport names and other fields with Unicode characters.
  - Regression tests added for edge cases (e.g., German ß, Turkish İ, Cyrillic).

- **Runway Length Filtering:**
  - Route generation now properly filters airports based on required runway length for each aircraft.

### Miscellaneous
- **Code Style:** Numerous formatting improvements for clarity and maintainability, especially in test files and helpers.
- **Internal Refactoring:** Several modules updated to use new route generator and search logic.

---

For full details, see the diff: [v1.0.12...v1.0.13](https://github.com/daanbouwman19/flight-planner/compare/v1.0.12...v1.0.13)
