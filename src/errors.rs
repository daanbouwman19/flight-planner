use crate::modules::airport::AirportSearchError;
use std::fmt;

#[derive(Debug)]
pub enum ValidationError {
    InvalidData(String),
    InvalidId(i32),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ValidationError::InvalidId(id) => write!(f, "Invalid ID: {}", id),
        }
    }
}

impl std::error::Error for ValidationError {}

pub enum Error {
    Validation(ValidationError),
    AirportSearch(AirportSearchError),
    Diesel(diesel::result::Error),
    Other(std::io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Validation(e) => write!(f, "Validation error: {}", e),
            Error::AirportSearch(e) => write!(f, "Airport search error: {}", e),
            Error::Diesel(e) => write!(f, "Database error: {}", e),
            Error::Other(e) => write!(f, "Other error: {}", e),
        }
    }
}

impl From<ValidationError> for Error {
    fn from(error: ValidationError) -> Self {
        Error::Validation(error)
    }
}

impl From<AirportSearchError> for Error {
    fn from(error: AirportSearchError) -> Self {
        Error::AirportSearch(error)
    }
}

impl From<diesel::result::Error> for Error {
    fn from(error: diesel::result::Error) -> Self {
        Error::Diesel(error)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::Other(error)
    }
}
