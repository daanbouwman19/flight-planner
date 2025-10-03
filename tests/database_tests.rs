use flight_planner::database::get_db_url;
#[cfg(target_os = "windows")]
use flight_planner::database::get_install_shared_data_dir;
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
struct EnvVarGuard {
    key: &'static str,
    original: Option<String>,
}

#[cfg(target_os = "windows")]
impl EnvVarGuard {
    fn new(key: &'static str) -> Self {
        Self {
            key,
            original: env::var(key).ok(),
        }
    }

    fn clear(&self) {
        unsafe { env::remove_var(self.key) };
    }

    fn set(&self, value: &str) {
        unsafe { env::set_var(self.key, value) };
    }
}

#[cfg(target_os = "windows")]
impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        match &self.original {
            Some(value) => unsafe { env::set_var(self.key, value) },
            None => unsafe { env::remove_var(self.key) },
        }
    }
}

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
        let guard = EnvVarGuard::new("FLIGHT_PLANNER_SHARE_DIR");
        guard.clear();

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
#[cfg(target_os = "windows")]
fn test_get_install_shared_data_dir_windows_with_env_var() {
    with_env_lock(|| {
        let guard = EnvVarGuard::new("FLIGHT_PLANNER_SHARE_DIR");
        let test_dir = "C:\\test-share-dir";
        guard.set(test_dir);

        let expected_path = PathBuf::from(test_dir);
        assert_eq!(
            get_install_shared_data_dir().unwrap(),
            expected_path,
            "Should return the path from the environment variable"
        );
    });
}
