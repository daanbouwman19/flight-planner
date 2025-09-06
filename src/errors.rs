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
    DieselConnection(diesel::ConnectionError),
    Other(std::io::Error),
    InvalidPath(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation(e) => write!(f, "Validation error: {e}"),
            Self::AirportSearch(e) => write!(f, "Airport search error: {e}"),
            Self::Diesel(e) => write!(f, "Database error: {e}"),
            Self::DieselConnection(e) => write!(f, "Database connection error: {e}"),
            Self::Other(e) => write!(f, "Other error: {e}"),
            Self::InvalidPath(p) => write!(f, "Invalid (non-UTF8) path: {p}"),
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

impl From<diesel::ConnectionError> for Error {
    fn from(error: diesel::ConnectionError) -> Self {
        Self::DieselConnection(error)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Other(error)
    }
}

impl From<r2d2::Error> for Error {
    fn from(error: r2d2::Error) -> Self {
        Self::Other(std::io::Error::other(error.to_string()))
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
            _ => Self::Other(std::io::Error::other(error.to_string())),
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
