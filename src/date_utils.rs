use chrono::{Local, NaiveDate, TimeZone};

/// Utility functions for consistent date handling across the application.
/// All dates are stored in UTC but displayed in local time.
/// Gets the current date in UTC as a string in YYYY-MM-DD format for storage.
pub fn get_current_date_utc() -> String {
    chrono::Utc::now().format("%Y-%m-%d").to_string()
}

/// Converts a UTC date string (YYYY-MM-DD) to local time for display.
/// If the input is None or invalid, returns "Never" or "Invalid date".
pub fn format_date_for_display(utc_date: Option<&String>) -> String {
    match utc_date {
        Some(date_str) if !date_str.is_empty() => {
            // Parse the UTC date string
            match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                Ok(naive_date) => {
                    // Combine with midnight time to create a NaiveDateTime
                    let naive_datetime = naive_date.and_hms_opt(0, 0, 0).unwrap();
                    // Convert the UTC NaiveDateTime to the local timezone
                    let local_date = Local.from_utc_datetime(&naive_datetime).date_naive();
                    // Format the date for display
                    local_date.format("%Y-%m-%d").to_string()
                }
                Err(_) => {
                    // If parsing fails, return "Invalid date"
                    "Invalid date".to_string()
                }
            }
        }
        _ => "Never".to_string(),
    }
}
