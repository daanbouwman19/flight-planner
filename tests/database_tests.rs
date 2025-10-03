use flight_planner::database::{get_db_url, get_install_shared_data_dir};
use flight_planner::errors::Error;
use std::path::PathBuf;

fn default_path_fn() -> Result<PathBuf, Error> {
    Ok(PathBuf::from("default.db"))
}

#[cfg(target_os = "windows")]
use std::{
    env,
    sync::{Mutex, OnceLock},
};

#[cfg(target_os = "windows")]
static ENV_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

#[cfg(target_os = "windows")]
fn with_env_lock<F, T>(f: F) -> T
where
    F: FnOnce() -> T,
{
    let mutex = ENV_MUTEX.get_or_init(|| Mutex::new(()));
    let guard = mutex.lock().expect("env mutex poisoned");
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
    with_env_lock(|| {
        let original = env::var("FLIGHT_PLANNER_SHARE_DIR").ok();
        unsafe { env::remove_var("FLIGHT_PLANNER_SHARE_DIR") };

        let mut exe_path = env::current_exe().unwrap();
        exe_path.pop();
        assert_eq!(
            get_install_shared_data_dir().unwrap(),
            exe_path,
            "Should return the executable's directory by default"
        );

        match original {
            Some(value) => unsafe { env::set_var("FLIGHT_PLANNER_SHARE_DIR", value) },
            None => unsafe { env::remove_var("FLIGHT_PLANNER_SHARE_DIR") },
        }
    });
}

#[test]
#[cfg(target_os = "windows")]
fn test_get_install_shared_data_dir_windows_with_env_var() {
    with_env_lock(|| {
        let original = env::var("FLIGHT_PLANNER_SHARE_DIR").ok();
        let test_dir = "C:\\test-share-dir";
        unsafe { env::set_var("FLIGHT_PLANNER_SHARE_DIR", test_dir) };

        let expected_path = PathBuf::from(test_dir);
        assert_eq!(
            get_install_shared_data_dir().unwrap(),
            expected_path,
            "Should return the path from the environment variable"
        );

        match original {
            Some(value) => unsafe { env::set_var("FLIGHT_PLANNER_SHARE_DIR", value) },
            None => unsafe { env::remove_var("FLIGHT_PLANNER_SHARE_DIR") },
        }
    });
}
