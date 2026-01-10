use flight_planner::get_aircraft_csv_candidate_paths;
use flight_planner::get_app_data_dir;
use std::env;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn with_env_overrides<F, T>(overrides: Vec<(&str, Option<&str>)>, f: F) -> T
where
    F: FnOnce() -> T,
{
    struct RestoreGuard {
        original: Vec<(String, Option<String>)>,
    }

    impl Drop for RestoreGuard {
        fn drop(&mut self) {
            for (key, value) in &self.original {
                match value {
                    Some(val) => unsafe { env::set_var(key, val) },
                    None => unsafe { env::remove_var(key) },
                }
            }
        }
    }

    let _lock = ENV_LOCK.lock().expect("env mutex poisoned");

    let mut original = Vec::new();
    for (key, _) in &overrides {
        original.push((key.to_string(), env::var(key).ok()));
    }

    let guard = RestoreGuard { original };

    for (key, value) in overrides {
        match value {
            Some(val) => unsafe { env::set_var(key, val) },
            None => unsafe { env::remove_var(key) },
        }
    }

    let result = f();
    drop(guard);
    result
}

#[test]
#[cfg(target_os = "windows")]
fn test_get_aircraft_csv_candidate_paths_no_duplicates_on_windows() {
    let candidates = get_aircraft_csv_candidate_paths();
    let unique_candidates: std::collections::HashSet<_> = candidates.iter().collect();
    // This test assumes no duplicates by default logic
    assert_eq!(
        candidates.len(),
        unique_candidates.len(),
        "get_aircraft_csv_candidate_paths should not produce duplicate paths on Windows"
    );
}

#[test]
fn test_get_app_data_dir_usage_of_env_var() {
    let test_path_str = "tmp_test_dir";
    let test_path = std::path::PathBuf::from(test_path_str);

    with_env_overrides(
        vec![("FLIGHT_PLANNER_DATA_DIR", Some(test_path_str))],
        || {
            let dir = get_app_data_dir().expect("Failed to get app data dir");
            assert_eq!(dir, test_path);
        },
    );
}

#[test]
fn test_csv_candidate_paths_includes_app_data_dir() {
    let test_path_str = "tmp_csv_test_dir";
    let test_path = std::path::PathBuf::from(test_path_str);

    with_env_overrides(
        vec![("FLIGHT_PLANNER_DATA_DIR", Some(test_path_str))],
        || {
            let candidates = get_aircraft_csv_candidate_paths();
            assert!(candidates.contains(&test_path.join("aircrafts.csv")));
        },
    );
}
