use chrono::NaiveDate;
use flight_planner::date_utils::{format_date_for_display, get_current_date_utc};

#[test]
fn test_get_current_date_utc() {
    let date = get_current_date_utc();
    // Should be in YYYY-MM-DD format
    assert!(date.len() == 10);
    assert!(date.contains('-'));

    // Should be parseable as a date
    assert!(NaiveDate::parse_from_str(&date, "%Y-%m-%d").is_ok());
}

#[test]
fn test_format_date_for_display_valid_date() {
    let utc_date = Some("2024-12-25".to_string());
    let result = format_date_for_display(utc_date.as_ref());

    // Should return a valid date string (might be different due to timezone conversion)
    assert!(result.len() == 10);
    assert!(result.contains('-'));
    assert!(NaiveDate::parse_from_str(&result, "%Y-%m-%d").is_ok());
}

#[test]
fn test_format_date_for_display_none() {
    assert_eq!(format_date_for_display(None), "Never");
}

#[test]
fn test_format_date_for_display_empty() {
    let empty_date = Some("".to_string());
    assert_eq!(format_date_for_display(empty_date.as_ref()), "Never");
}

#[test]
fn test_format_date_for_display_invalid() {
    let invalid_date = Some("invalid-date".to_string());
    assert_eq!(
        format_date_for_display(invalid_date.as_ref()),
        "invalid-date"
    );
}

#[test]
fn test_format_date_for_display_utc_is_preserved() {
    let utc_date = Some("2024-01-01".to_string());
    // In timezones behind UTC, the current implementation will convert this to "2023-12-31"
    assert_eq!(
        format_date_for_display(utc_date.as_ref()),
        "2024-01-01"
    );
}
