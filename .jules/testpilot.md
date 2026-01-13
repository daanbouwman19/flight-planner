## 2026-01-10 - [Cleanup] Tests for util_tests.rs
Discovery: Redundant setup logic for 'Airport' struct in 'tests/util_tests.rs' and weak assertions (assert!(x == y)).
Strategy: Extracted a factory method 'create_airport' and used 'assert_eq!' for better readability and debugging.

## 2026-01-13 - [Refactor] Factory Extraction in airport_tests.rs
Discovery: Repeated 11-line 'Aircraft' struct initialization across 7+ tests in 'tests/airport_tests.rs', creating a "Wall of Setup".
Strategy: Implemented 'create_test_aircraft' helper and used Rust's struct update syntax (..factory()) to highlight only relevant test data.
