use chrono::{NaiveDate, Utc};

/// Gets the current date in UTC and formats it as a string.
///
/// All dates in the application are handled in UTC to ensure consistency. This
/// function provides the standard format (`YYYY-MM-DD`) for storing dates.
///
/// # Returns
///
/// A `String` containing the current UTC date.
pub fn get_current_date_utc() -> String {
    Utc::now().format("%Y-%m-%d").to_string()
}

/// Formats a UTC date string for display.
///
/// This function takes an optional date string and formats it for display.
/// If the input is `None` or an empty string, it returns "Never". If parsing
/// the date fails, it returns the original string as a fallback.
///
/// # Arguments
///
/// * `utc_date` - An `Option` containing a string slice with the date in
///   `YYYY-MM-DD` format.
///
/// # Returns
///
/// A `String` with the formatted date, "Never", or the original string.
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, Utc};

    #[test]
    fn test_get_current_date_utc() {
        let result = get_current_date_utc();
        let now = Utc::now();
        let year = now.year();
        let month = now.month();
        let day = now.day();
        assert!(result.starts_with(&format!("{}-{:02}-{:02}", year, month, day)));
    }

    #[test]
    fn test_format_date_for_display_some() {
        let date = Some(&"2022-01-01".to_string());
        assert_eq!(format_date_for_display(date), "2022-01-01");
    }

    #[test]
    fn test_format_date_for_display_none() {
        assert_eq!(format_date_for_display(None), "Never");
    }

    #[test]
    fn test_format_date_for_display_empty() {
        let date = Some(&"".to_string());
        assert_eq!(format_date_for_display(date), "Never");
    }

    #[test]
    fn test_format_date_for_display_invalid() {
        let date = Some(&"invalid-date".to_string());
        assert_eq!(format_date_for_display(date), "invalid-date");
    }
}
