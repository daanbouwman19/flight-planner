## 2026-01-10 - [Cleanup] Tests for util_tests.rs
Discovery: Redundant setup logic for 'Airport' struct in 'tests/util_tests.rs' and weak assertions (assert!(x == y)).
Strategy: Extracted a factory method 'create_airport' and used 'assert_eq!' for better readability and debugging.

## 2026-01-13 - [Refactor] Factory Extraction in airport_tests.rs
Discovery: Repeated 11-line 'Aircraft' struct initialization across 7+ tests in 'tests/airport_tests.rs', creating a "Wall of Setup".
Strategy: Implemented 'create_test_aircraft' helper and used Rust's struct update syntax (..factory()) to highlight only relevant test data.

## 2026-01-16 - [Duplication] Consolidated env var overrides
Discovery: Identical `with_env_overrides` helper function and `ENV_LOCK` mutex were duplicated in `tests/database_tests.rs` and `tests/lib_tests.rs`.
Strategy: Moved the helper logic to `tests/common/mod.rs` and updated both test files to import it, reducing duplication and centralizing env var manipulation logic.

## 2026-01-17 - [Refactor] Factory Extraction in data_operations_tests.rs
Discovery: Redundant setup logic for 'History' and 'Aircraft' structs in 'tests/data_operations_tests.rs' creating a "Wall of Setup".
Strategy: Added 'create_test_history' to 'tests/common/mod.rs' and refactored 'data_operations_tests.rs' to use it along with existing 'create_test_aircraft' and 'create_test_airport' factories.

## 2026-01-19 - [Refactor] Centralized Airport Test Setup
Discovery: `tests/airport_tests.rs` contained a duplicate 60-line database setup (schema + data) that was nearly identical to `tests/common/mod.rs` but included Runways.
Strategy: Extended `tests/common/mod.rs::setup_test_db` to include the Runways table and additional test data, enabling `airport_tests.rs` to reuse the shared setup and eliminating the duplicate code.

## 2026-01-20 - [Refactor] Centralize DB Schema Setup
Discovery: `tests/common/mod.rs` contained duplicate 50-line database setup logic (schema + data) in both `setup_test_pool_db` and `setup_test_db`.
Strategy: Extracted `init_aircraft_db` and `init_airport_db` helper functions to centralize the schema creation and initial data insertion, reducing duplication and improving maintainability.
