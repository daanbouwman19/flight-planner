use chrono::{NaiveDate, Utc};

/// Utility functions for consistent date handling across the application.
/// All dates are stored and displayed as UTC (no timezone conversion).
/// Gets the current date in UTC as a string in YYYY-MM-DD format for storage.
pub fn get_current_date_utc() -> String {
    Utc::now().format("%Y-%m-%d").to_string()
}

/// Formats a UTC date string (YYYY-MM-DD) for display.
/// Dates are displayed as they are stored (UTC), with no timezone conversion.
/// If the input is None or empty, returns "Never".
/// If parsing fails, returns the original string as a fallback.
pub fn format_date_for_display(utc_date: Option<&String>) -> String {
    match utc_date {
        Some(date_str) if !date_str.is_empty() => {
            // Parse the UTC date string
            match NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                Ok(naive_date) => {
                    // Format the date directly without timezone conversion
                    naive_date.format("%Y-%m-%d").to_string()
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
