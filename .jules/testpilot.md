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

## 2026-01-22 - [Refactor] Centralize DB Schema Setup
Discovery: `tests/common/mod.rs` contained duplicate 50-line database setup logic (schema + data) in both `setup_test_pool_db` and `setup_test_db`.
Strategy: Extracted `init_aircraft_db` and `init_airport_db` helper functions to centralize the schema creation and initial data insertion, reducing duplication and improving maintainability.

## 2026-01-24 - [Refactor] Parameterized Tests in util_tests.rs
Discovery: Repetitive test logic for `calculate_haversine_distance_nm` in `tests/util_tests.rs` with 4 separate tests doing essentially the same thing.
Strategy: Consolidated the tests into a single `test_calculate_haversine_distance_nm_parameterized` function using a vector of test cases to reduce duplication and improve maintainability.

## 2026-01-25 - [Refactor] Shared Factories in GUI Tests
Discovery: Duplicate test data setup ("Wall of Setup") in `tests/gui/routes_tests.rs`, `ui_logic_tests.rs`, and `table_items_tests.rs`.
Strategy: Exposed `tests/common/mod.rs` to the GUI test crate via `#[path = "../common/mod.rs"] mod common;` in `tests/gui/main.rs`, allowing reuse of existing `create_test_aircraft`, `create_test_airport`, and `create_test_runway` factories with struct update syntax.

## 2026-01-26 - [Refactor] Parameterized Tests in aircraft_tests.rs
Discovery: Repetitive copy-pasted test blocks for `format_aircraft` in `tests/aircraft_tests.rs`, leading to code duplication.
Strategy: Consolidated the tests into a single `test_format_aircraft` function using a vector of test cases (table-driven test) to reduce duplication and improve maintainability.

## 2026-01-27 - [Refactor] Consolidate CLI Test Setup
Discovery: `tests/cli_tests.rs` contained duplicate database setup logic (schema + data) and local helpers like `add_test_aircraft`.
Strategy: Refactored to use `common::setup_test_db` and shared seed data, reducing duplication and standardizing test data across the suite.
