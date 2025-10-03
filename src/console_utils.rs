use crate::errors::Error;
use crate::errors::ValidationError;
use crate::models::Aircraft;
use crate::traits::AircraftOperations;

/// Reads a 'y' or 'n' response from the user.
///
/// This function takes a closure that provides the input character. It returns
/// `true` for 'y', `false` for 'n', and an error for any other input.
///
/// # Arguments
///
/// * `read_input` - A closure that returns a `Result<char, std::io::Error>`.
///
/// # Returns
///
/// A `Result` containing `true` for 'y', `false` for 'n', or a `ValidationError`
/// on failure or invalid input.
pub fn read_yn<F: Fn() -> Result<char, std::io::Error>>(
    read_input: F,
) -> Result<bool, ValidationError> {
    let input = read_input().map_err(|e| ValidationError::InvalidData(e.to_string()))?;
    match input {
        'y' => Ok(true),
        'n' => Ok(false),
        _ => Err(ValidationError::InvalidData("Invalid input".to_string())),
    }
}

/// Reads and validates an ID from the user.
///
/// This function takes a closure that provides the input as a string. It attempts
/// to parse the string into an `i32` and validates that it is a positive number.
///
/// # Arguments
///
/// * `read_input` - A closure that returns a `Result<String, std::io::Error>`.
///
/// # Returns
///
/// A `Result` containing the valid ID as an `i32`, or a `ValidationError` on
/// failure, invalid input, or if the ID is not positive.
pub fn read_id<F: Fn() -> Result<String, std::io::Error>>(
    read_input: F,
) -> Result<i32, ValidationError> {
    let input = read_input().map_err(|e| ValidationError::InvalidData(e.to_string()))?;
    let id = match input.trim().parse::<i32>() {
        Ok(id) => id,
        Err(e) => {
            log::error!("Failed to parse id: {e}");
            return Err(ValidationError::InvalidData("Invalid id".to_string()));
        }
    };

    if id < 1 {
        return Err(ValidationError::InvalidId(id));
    }

    Ok(id)
}

/// Asks the user if they want to mark an aircraft as flown and updates it if confirmed.
///
/// If the user confirms by entering 'y', this function sets the aircraft's `flown`
/// status to `1` and updates its `date_flown` to the current UTC date.
///
/// # Arguments
///
/// * `database_connections` - A mutable reference to a type that implements `AircraftOperations`.
/// * `aircraft` - A mutable reference to the `Aircraft` to be updated.
/// * `ask_char_fn` - A closure that prompts the user and reads a single character response.
///
/// # Returns
///
/// A `Result` indicating success or an `Error` if the database update fails.
pub fn ask_mark_flown<T: AircraftOperations, F: Fn() -> Result<char, std::io::Error>>(
    database_connections: &mut T,
    aircraft: &mut Aircraft,
    ask_char_fn: F,
) -> Result<(), Error> {
    if matches!(ask_char_fn(), Ok('y')) {
        aircraft.date_flown = Some(crate::date_utils::get_current_date_utc());
        aircraft.flown = 1;
        database_connections.update_aircraft(aircraft)?;
    }

    Ok(())
}
