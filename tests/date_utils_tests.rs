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
fn test_format_date_for_display_parameterized() {
    let cases = vec![
        (Some("2024-12-25"), "2024-12-25", "Valid date"),
        (None, "Never", "None input"),
        (Some(""), "Never", "Empty string"),
        (
            Some("invalid-date"),
            "invalid-date",
            "Invalid format fallback",
        ),
        (
            Some("2024-01-01"),
            "2024-01-01",
            "UTC preserved (no timezone shift)",
        ),
    ];

    for (input, expected, description) in cases {
        let input_string = input.map(|s| s.to_string());
        assert_eq!(
            format_date_for_display(input_string.as_ref()),
            expected,
            "Failed case: {}",
            description
        );
    }
}
