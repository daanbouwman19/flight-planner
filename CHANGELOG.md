# Changelog

## [v1.0.20] - chore: Version bump

### Miscellaneous
- Bumped version to v1.0.20.

---

For full details, see the diff: [v1.0.19...v1.0.20](https://github.com/daanbouwman19/flight-planner/compare/v1.0.19...v1.0.20)

## [v1.0.19] - feat(perf), refactor: Improved performance and code organization

### Changed
- **Performance Improvements:**
    - Refactored airport and route selection functions to return vectors instead of iterators for improved usability and performance.
    - Small performance improvements in `src/modules/routes.rs`.
- **Refactoring:**
    - Refactored airport suitability tests to include random number generator for enhanced randomness.
    - Simplified desktop icon in install scripts.
    - Refactored `src/lib.rs` layout, organized module definitions, and removed redundant comments.

### Fixed
- **GUI Warning Handling:** Enhanced GUI warning handling with fallback to console output.
- **Log Format:** Standardized route generation duration log format.

### Miscellaneous
- Removed unused `SliceRandom` import from routes module and tests.
- Bumped version to v1.0.19.

---

For full details, see the diff: [v1.0.18...v1.0.19](https://github.com/daanbouwman19/flight-planner/compare/v1.0.18...v1.0.19)

## [v1.0.18](https://github.com/daanbouwman19/flight-planner/compare/v1.0.17...v1.0.18)

### Dependencies
- Updated various dependencies to their latest versions.

---

For full details, see the diff: [v1.0.17...v1.0.18](https://github.com/daanbouwman19/flight-planner/compare/v1.0.17...v1.0.18)

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
