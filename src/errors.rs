use std::fmt;

#[derive(Debug)]
pub enum ValidationError {
    InvalidData(String),
    DatabaseError(String),
    InvalidId(i32),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ValidationError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            ValidationError::InvalidId(id) => write!(f, "Invalid ID: {}", id),
        }
    }
}

impl std::error::Error for ValidationError {}