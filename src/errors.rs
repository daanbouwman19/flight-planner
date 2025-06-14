use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum ValidationError {
    InvalidData(String),
    InvalidId(i32),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidData(msg) => write!(f, "Invalid data: {msg}"),
            Self::InvalidId(id) => write!(f, "Invalid ID: {id}"),
        }
    }
}

impl std::error::Error for ValidationError {}

#[derive(Debug)]
pub enum Error {
    Validation(ValidationError),
    AirportSearch(AirportSearchError),
    Diesel(diesel::result::Error),
    Other(std::io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation(e) => write!(f, "Validation error: {e}"),
            Self::AirportSearch(e) => write!(f, "Airport search error: {e}"),
            Self::Diesel(e) => write!(f, "Database error: {e}"),
            Self::Other(e) => write!(f, "Other error: {e}"),
        }
    }
}

impl From<ValidationError> for Error {
    fn from(error: ValidationError) -> Self {
        Self::Validation(error)
    }
}

impl From<AirportSearchError> for Error {
    fn from(error: AirportSearchError) -> Self {
        Self::AirportSearch(error)
    }
}

impl From<diesel::result::Error> for Error {
    fn from(error: diesel::result::Error) -> Self {
        Self::Diesel(error)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Other(error)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Validation(ref err) => Some(err),
            Self::AirportSearch(ref err) => Some(err),
            Self::Diesel(ref err) => Some(err),
            Self::Other(ref err) => Some(err),
            // Add other variants here if they wrap errors and implement std::error::Error
        }
    }
}

#[derive(Debug)]
pub enum AirportSearchError {
    NotFound,
    NoSuitableRunway,
    DistanceExceeded,
    Other(std::io::Error),
}

impl From<diesel::result::Error> for AirportSearchError {
    fn from(error: diesel::result::Error) -> Self {
        match error {
            diesel::result::Error::NotFound => Self::NotFound,
            _ => Self::Other(std::io::Error::other(
                error.to_string(),
            )),
        }
    }
}

impl From<std::io::Error> for AirportSearchError {
    fn from(error: std::io::Error) -> Self {
        Self::Other(error)
    }
}

impl std::error::Error for AirportSearchError {}

impl std::fmt::Display for AirportSearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "Airport not found"),
            Self::NoSuitableRunway => write!(f, "No suitable runway found"),
            Self::DistanceExceeded => write!(f, "Distance exceeded"),
            Self::Other(error) => write!(f, "Other error: {error}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let io_error = io::Error::new(io::ErrorKind::Other, "Test IO error");

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
        let io_error = io::Error::new(io::ErrorKind::Other, "Test IO error");
        let error: Error = io_error.into();
        match error {
            Error::Other(e) => assert_eq!(format!("{e}"), "Test IO error"),
            _ => panic!("Expected IOError"),
        }
    }
}
