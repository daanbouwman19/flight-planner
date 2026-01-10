# TestPilot Journal

## 2024-05-21 - Factory Extraction in Utility Tests
**Discovery:** Many tests in `tests/util_tests.rs` were repeating the full `Airport` struct initialization just to test distance calculations based on latitude and longitude.
**Strategy:** Extracted a `create_test_airport` factory helper and consolidated multiple copy-pasted tests into a single table-driven test case using a struct vector. This reduced boilerplate and makes adding new geometric cases trivial.
