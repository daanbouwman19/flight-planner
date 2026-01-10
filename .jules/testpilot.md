## 2026-01-10 - [Cleanup] Tests for util_tests.rs
Discovery: Redundant setup logic for 'Airport' struct in 'tests/util_tests.rs' and weak assertions (assert!(x == y)).
Strategy: Extracted a factory method 'create_airport' and used 'assert_eq!' for better readability and debugging.
