use std::collections::HashSet;

#[test]
#[cfg(target_os = "windows")]
fn test_get_aircraft_csv_candidate_paths_no_duplicates_on_windows() {
    let candidates = flight_planner::get_aircraft_csv_candidate_paths();
    let unique_candidates: HashSet<_> = candidates.iter().collect();
    assert_eq!(
        candidates.len(),
        unique_candidates.len(),
        "get_aircraft_csv_candidate_paths should not produce duplicate paths on Windows"
    );
}
