use flight_planner::database::get_db_url;
#[cfg(target_os = "windows")]
use flight_planner::database::get_install_shared_data_dir;
use flight_planner::errors::Error;
use std::path::PathBuf;

fn default_path_fn() -> Result<PathBuf, Error> {
    Ok(PathBuf::from("default.db"))
}

#[cfg(target_os = "windows")]
use std::{env, sync::Mutex};

#[cfg(target_os = "windows")]
static ENV_LOCK: Mutex<()> = Mutex::new(());

#[cfg(target_os = "windows")]
fn with_share_dir_override<F, T>(value: Option<&str>, f: F) -> T
where
    F: FnOnce() -> T,
{
    struct RestoreGuard {
        original: Option<String>,
    }

    impl Drop for RestoreGuard {
        fn drop(&mut self) {
            match &self.original {
                Some(value) => unsafe { env::set_var("FLIGHT_PLANNER_SHARE_DIR", value) },
                None => unsafe { env::remove_var("FLIGHT_PLANNER_SHARE_DIR") },
            }
        }
    }

    let _lock = ENV_LOCK.lock().expect("env mutex poisoned");
    let guard = RestoreGuard {
        original: env::var("FLIGHT_PLANNER_SHARE_DIR").ok(),
    };

    match value {
        Some(val) => unsafe { env::set_var("FLIGHT_PLANNER_SHARE_DIR", val) },
        None => unsafe { env::remove_var("FLIGHT_PLANNER_SHARE_DIR") },
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
    with_share_dir_override(None, || {
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
    // Create a temporary "shared" directory and a dummy database file in it
    let tmp_dir = std::env::temp_dir().join("flight-planner-test");
    std::fs::create_dir_all(&tmp_dir).unwrap();
    let expected_db_path = tmp_dir.join("airports.db3");
    std::fs::File::create(&expected_db_path).unwrap();

    let shared_dir_str = tmp_dir.to_str().unwrap();

    // Set the env var to point to our temporary shared directory
    unsafe {
        std::env::set_var("FLIGHT_PLANNER_SHARE_DIR", shared_dir_str);
    }

    // We assume that `airports.db3` does not exist in the app data directory
    // for the test environment.
    let resolved_path = flight_planner::database::get_airport_db_path().unwrap();
    assert_eq!(
        resolved_path, expected_db_path,
        "Should find the database in the shared directory"
    );

    // Clean up the environment variable and temporary directory
    unsafe {
        std::env::remove_var("FLIGHT_PLANNER_SHARE_DIR");
    }
    std::fs::remove_dir_all(&tmp_dir).unwrap();
}

#[test]
#[cfg(target_os = "windows")]
fn test_get_install_shared_data_dir_windows_with_env_var() {
    let test_dir = "C:\\test-share-dir";
    with_share_dir_override(Some(test_dir), || {
        let expected_path = PathBuf::from(test_dir);
        assert_eq!(
            get_install_shared_data_dir().unwrap(),
            expected_path,
            "Should return the path from the environment variable"
        );
    });
}
