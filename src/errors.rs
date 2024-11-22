use std::fmt;

#[cfg(test)]
#[derive(Debug)]
pub enum ValidationError {
    InvalidData(String),
    DatabaseError(String),
}

#[cfg(test)]
impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            ValidationError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
        }
    }
}

#[cfg(test)]
impl std::error::Error for ValidationError {}
