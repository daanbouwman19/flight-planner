use chrono::{Local, NaiveDate, TimeZone};
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
    let utc_date_string = "2024-12-25".to_string();
    let result = format_date_for_display(Some(&utc_date_string));

    // Should return a valid date string that is converted to the local timezone.
    let naive_date = NaiveDate::parse_from_str(&utc_date_string, "%Y-%m-%d").unwrap();
    let naive_datetime = naive_date.and_hms_opt(0, 0, 0).unwrap();
    let local_date = Local.from_utc_datetime(&naive_datetime).date_naive();

    assert_eq!(result, local_date.format("%Y-%m-%d").to_string());
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
        "Invalid date"
    );
}
