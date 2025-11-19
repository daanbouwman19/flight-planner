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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Aircraft, NewAircraft};
    use crate::traits::AircraftOperations;
    use diesel::result::Error as DieselError;
    use std::io;

    // Mock for AircraftOperations
    struct MockAircraftOperations;

    impl AircraftOperations for MockAircraftOperations {
        fn get_not_flown_count(&mut self) -> Result<i64, DieselError> {
            Ok(0)
        }

        fn random_not_flown_aircraft(&mut self) -> Result<Aircraft, DieselError> {
            Ok(new_aircraft())
        }

        fn get_all_aircraft(&mut self) -> Result<Vec<Aircraft>, DieselError> {
            Ok(vec![])
        }

        fn update_aircraft(&mut self, _record: &Aircraft) -> Result<(), DieselError> {
            Ok(())
        }

        fn random_aircraft(&mut self) -> Result<Aircraft, DieselError> {
            Ok(new_aircraft())
        }

        fn get_aircraft_by_id(&mut self, _aircraft_id: i32) -> Result<Aircraft, DieselError> {
            Ok(new_aircraft())
        }

        fn mark_all_aircraft_not_flown(&mut self) -> Result<(), DieselError> {
            Ok(())
        }

        fn add_aircraft(&mut self, _record: &NewAircraft) -> Result<Aircraft, DieselError> {
            Ok(new_aircraft())
        }
    }

    fn new_aircraft() -> Aircraft {
        Aircraft {
            id: 1,
            manufacturer: "Boeing".to_string(),
            variant: "737-800".to_string(),
            icao_code: "B738".to_string(),
            flown: 0,
            aircraft_range: 3000,
            category: "A".to_string(),
            cruise_speed: 450,
            date_flown: None,
            takeoff_distance: Some(2000),
        }
    }

    #[test]
    fn test_read_yn_y() {
        let result = read_yn(|| Ok('y'));
        assert_eq!(result, Ok(true));
    }

    #[test]
    fn test_read_yn_n() {
        let result = read_yn(|| Ok('n'));
        assert_eq!(result, Ok(false));
    }

    #[test]
    fn test_read_yn_invalid() {
        let result = read_yn(|| Ok('a'));
        assert!(result.is_err());
    }

    #[test]
    fn test_read_yn_error() {
        let result = read_yn(|| Err(io::Error::new(io::ErrorKind::Other, "test error")));
        assert!(result.is_err());
    }

    #[test]
    fn test_read_id_valid() {
        let result = read_id(|| Ok("123".to_string()));
        assert_eq!(result, Ok(123));
    }

    #[test]
    fn test_read_id_invalid() {
        let result = read_id(|| Ok("abc".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_read_id_zero() {
        let result = read_id(|| Ok("0".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_read_id_negative() {
        let result = read_id(|| Ok("-1".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn test_read_id_error() {
        let result = read_id(|| Err(io::Error::new(io::ErrorKind::Other, "test error")));
        assert!(result.is_err());
    }

    #[test]
    fn test_ask_mark_flown_y() {
        let mut mock_db = MockAircraftOperations;
        let mut aircraft = new_aircraft();
        let result = ask_mark_flown(&mut mock_db, &mut aircraft, || Ok('y'));
        assert!(result.is_ok());
        assert_eq!(aircraft.flown, 1);
        assert!(aircraft.date_flown.is_some());
    }

    #[test]
    fn test_ask_mark_flown_n() {
        let mut mock_db = MockAircraftOperations;
        let mut aircraft = new_aircraft();
        let result = ask_mark_flown(&mut mock_db, &mut aircraft, || Ok('n'));
        assert!(result.is_ok());
        assert_eq!(aircraft.flown, 0);
        assert!(aircraft.date_flown.is_none());
    }
}
