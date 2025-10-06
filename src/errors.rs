use std::fmt;

/// Represents errors that occur during data validation.
#[derive(Debug, PartialEq, Eq)]
pub enum ValidationError {
    /// Used when input data is in an invalid format.
    InvalidData(String),
    /// Used when an ID is invalid (e.g., not positive).
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

/// The main error type for the application.
///
/// This enum consolidates various kinds of errors that can occur throughout
/// the application, including validation, database, I/O, and other specific
/// errors. It implements `From` for several error types to allow for easy
/// conversion.
#[derive(Debug)]
pub enum Error {
    /// A validation error.
    Validation(ValidationError),
    /// An error that occurred during an airport search.
    AirportSearch(AirportSearchError),
    /// An error from the Diesel ORM.
    Diesel(diesel::result::Error),
    /// An error related to establishing a database connection.
    DieselConnection(diesel::ConnectionError),
    /// A general I/O error or another miscellaneous error.
    Other(std::io::Error),
    /// An error indicating that a file path is not valid UTF-8.
    InvalidPath(String),
    /// An error related to the logging configuration.
    LogConfig(String),
    /// An error that occurred during database migrations.
    Migration(String),
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
            Self::LogConfig(msg) => write!(f, "Logging configuration error: {msg}"),
            Self::Migration(msg) => write!(f, "Database migration error: {msg}"),
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

/// Represents errors that can occur during an airport search.
#[derive(Debug)]
pub enum AirportSearchError {
    /// Indicates that no airport matching the criteria was found.
    NotFound,
    /// Indicates that while airports were found, none had a suitable runway.
    NoSuitableRunway,
    /// Indicates that a potential airport was found but is outside the required distance.
    DistanceExceeded,
    /// Wraps other I/O or database errors that may occur during the search.
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
