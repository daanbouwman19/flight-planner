use flight_planner::database::get_db_url;
use flight_planner::errors::Error;
use std::path::PathBuf;

fn default_path_fn() -> Result<PathBuf, Error> {
    Ok(PathBuf::from("default.db"))
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
