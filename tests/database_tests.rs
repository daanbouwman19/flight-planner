use flight_planner::database::get_db_url;
#[cfg(target_os = "windows")]
use flight_planner::database::get_install_shared_data_dir;
use flight_planner::errors::Error;
use std::path::PathBuf;

fn default_path_fn() -> Result<PathBuf, Error> {
    Ok(PathBuf::from("default.db"))
}

use std::{env, sync::Mutex};

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
fn test_get_db_url_with_some_url() {
    let url = "test.db";
    let result = get_db_url(Some(url), default_path_fn).unwrap();
    assert_eq!(result, "test.db");
}

#[test]
fn test_get_db_url_with_none_url() {
    let result = get_db_url(None, default_path_fn).unwrap();
    assert_eq!(result, "default.db");
}

#[test]
#[cfg(target_os = "windows")]
fn test_get_install_shared_data_dir_windows() {
    with_env_overrides(vec![("FLIGHT_PLANNER_SHARE_DIR", None)], || {
        let mut exe_path = env::current_exe().unwrap();
        exe_path.pop();
        assert_eq!(
            get_install_shared_data_dir().unwrap(),
            exe_path,
            "Should return the executable's directory by default"
        );
    });
}

#[test]
fn test_get_airport_db_path_shared_dir_fallback() {
    let tmp_dir = std::env::temp_dir().join("flight-planner-test-shared");
    std::fs::create_dir_all(&tmp_dir).unwrap();
    let expected_db_path = tmp_dir.join("airports.db3");
    std::fs::File::create(&expected_db_path).unwrap();

    let shared_dir_str = tmp_dir.to_str().unwrap();

    // Create a fake app data dir to ensure we don't pick up the real one
    let fake_app_data = tmp_dir.join("fake_app_data");
    std::fs::create_dir_all(&fake_app_data).unwrap();
    let fake_app_data_str = fake_app_data.to_str().unwrap();

    let mut overrides = vec![("FLIGHT_PLANNER_SHARE_DIR", Some(shared_dir_str))];

    // Override the app data dir
    overrides.push(("FLIGHT_PLANNER_DATA_DIR", Some(fake_app_data_str)));

    with_env_overrides(overrides, || {
        let resolved_path = flight_planner::database::get_airport_db_path().unwrap();
        assert_eq!(
            resolved_path, expected_db_path,
            "Should find the database in the shared directory"
        );
    });

    std::fs::remove_dir_all(&tmp_dir).unwrap();
}

#[test]
#[cfg(target_os = "windows")]
fn test_get_install_shared_data_dir_windows_with_env_var() {
    let test_dir = "C:\\test-share-dir";
    with_env_overrides(vec![("FLIGHT_PLANNER_SHARE_DIR", Some(test_dir))], || {
        let expected_path = PathBuf::from(test_dir);
        assert_eq!(
            get_install_shared_data_dir().unwrap(),
            expected_path,
            "Should return the path from the environment variable"
        );
    });
}

#[test]
fn test_get_aircraft_db_path() {
    let tmp_dir = std::env::temp_dir().join("flight-planner-test-aircraft");
    std::fs::create_dir_all(&tmp_dir).unwrap();
    let tmp_dir_str = tmp_dir.to_str().unwrap();

    with_env_overrides(vec![("FLIGHT_PLANNER_DATA_DIR", Some(tmp_dir_str))], || {
        let path = flight_planner::database::get_aircraft_db_path().unwrap();
        assert_eq!(path, tmp_dir.join("data.db"));
    });

    std::fs::remove_dir_all(&tmp_dir).unwrap();
}

#[test]
fn test_get_airport_db_path_in_app_data() {
    let tmp_dir = std::env::temp_dir().join("flight-planner-test-appdata");
    std::fs::create_dir_all(&tmp_dir).unwrap();
    let expected_db_path = tmp_dir.join("airports.db3");
    std::fs::File::create(&expected_db_path).unwrap();

    let tmp_dir_str = tmp_dir.to_str().unwrap();

    with_env_overrides(vec![("FLIGHT_PLANNER_DATA_DIR", Some(tmp_dir_str))], || {
        let path = flight_planner::database::get_airport_db_path().unwrap();
        assert_eq!(path, expected_db_path);
    });

    std::fs::remove_dir_all(&tmp_dir).unwrap();
}
