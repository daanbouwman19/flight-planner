use std::fmt;

#[derive(Debug)]
pub enum ValidationError {
    InvalidData(String),
    DatabaseError(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ValidationError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
        }
    }
}

impl std::error::Error for ValidationError {}
