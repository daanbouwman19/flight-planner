use chrono::{DateTime, Local, NaiveDate, TimeZone, Utc};

/// Utility functions for consistent date handling across the application.
/// All dates are stored in UTC but displayed in local time.
/// Gets the current date in UTC as a string in YYYY-MM-DD format for storage.
pub fn get_current_date_utc() -> String {
    Utc::now().format("%Y-%m-%d").to_string()
}

/// Converts a UTC date string (YYYY-MM-DD) to local time for display.
/// If the input is None or invalid, returns "Never".
pub fn format_date_for_display(utc_date: Option<&String>) -> String {
    match utc_date {
        Some(date_str) if !date_str.is_empty() => {
            // Parse the UTC date string
            match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                Ok(naive_date) => {
                    // Convert to UTC DateTime at start of day
                    let utc_datetime =
                        Utc.from_utc_datetime(&naive_date.and_hms_opt(0, 0, 0).unwrap());

                    // Convert to local time and format for display
                    let local_datetime: DateTime<Local> = utc_datetime.with_timezone(&Local);
                    local_datetime.format("%Y-%m-%d").to_string()
                }
                Err(_) => {
                    // If parsing fails, return the original string as fallback
                    date_str.clone()
                }
            }
        }
        _ => "Never".to_string(),
    }
}
