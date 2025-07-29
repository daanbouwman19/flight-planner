use flight_planner::errors::*;
use std::io;

#[test]
fn test_validation_error_display() {
    let invalid_data_error = ValidationError::InvalidData("Invalid field".to_string());
    let invalid_id_error = ValidationError::InvalidId(-123);

    assert_eq!(
        format!("{invalid_data_error}"),
        "Invalid data: Invalid field"
    );
    assert_eq!(format!("{invalid_id_error}"), "Invalid ID: -123");
}

#[test]
fn test_error_display() {
    let validation_error = ValidationError::InvalidData("Test".to_string());
    let airport_search_error = AirportSearchError::NotFound;
    let diesel_error = diesel::result::Error::NotFound;
    let io_error = io::Error::other("Test IO error");

    assert_eq!(
        format!("{}", Error::Validation(validation_error)),
        "Validation error: Invalid data: Test"
    );
    assert_eq!(
        format!("{}", Error::AirportSearch(airport_search_error)),
        "Airport search error: Airport not found"
    );
    assert_eq!(
        format!("{}", Error::Diesel(diesel_error)),
        "Database error: Record not found"
    );
    assert_eq!(
        format!("{}", Error::Other(io_error)),
        "Other error: Test IO error"
    );
}

#[test]
fn test_error_from_validation_error() {
    let validation_error = ValidationError::InvalidData("Test".to_string());
    let error: Error = validation_error.into();
    match error {
        Error::Validation(e) => assert_eq!(format!("{e}"), "Invalid data: Test"),
        _ => panic!("Expected ValidationError"),
    }
}

#[test]
fn test_error_from_airport_search_error() {
    let airport_search_error = AirportSearchError::NoSuitableRunway;
    let error: Error = airport_search_error.into();
    match error {
        Error::AirportSearch(e) => assert_eq!(format!("{e}"), "No suitable runway found"),
        _ => panic!("Expected AirportSearchError"),
    }
}

#[test]
fn test_error_from_diesel_error() {
    let diesel_error = diesel::result::Error::NotFound;
    let error: Error = diesel_error.into();
    match error {
        Error::Diesel(e) => assert_eq!(format!("{e}"), "Record not found"),
        _ => panic!("Expected DieselError"),
    }
}

#[test]
fn test_error_from_io_error() {
    let io_error = io::Error::other("Test IO error");
    let error: Error = io_error.into();
    match error {
        Error::Other(e) => assert_eq!(format!("{e}"), "Test IO error"),
        _ => panic!("Expected IOError"),
    }
}
