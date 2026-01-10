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
    utc_date
        .filter(|s| !s.is_empty())
        .map(|date_str| {
            NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                .map(|naive_date| naive_date.format("%Y-%m-%d").to_string())
                .unwrap_or_else(|_| date_str.clone())
        })
        .unwrap_or_else(|| "Never".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_date_for_display_valid() {
        let input = Some("2023-10-27".to_string());
        assert_eq!(format_date_for_display(input.as_ref()), "2023-10-27");
    }

    #[test]
    fn test_format_date_for_display_none() {
        assert_eq!(format_date_for_display(None), "Never");
    }

    #[test]
    fn test_format_date_for_display_empty() {
        let input = Some("".to_string());
        assert_eq!(format_date_for_display(input.as_ref()), "Never");
    }

    #[test]
    fn test_format_date_for_display_invalid_format_returns_original() {
        let input = Some("invalid-date".to_string());
        assert_eq!(format_date_for_display(input.as_ref()), "invalid-date");
    }

    #[test]
    fn test_format_date_for_display_already_formatted() {
        // This checks the logic where we parse and re-format.
        // If the logic is preserved, this should work.
        let input = Some("2024-01-01".to_string());
        assert_eq!(format_date_for_display(input.as_ref()), "2024-01-01");
    }
}
