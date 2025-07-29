use crate::errors::ValidationError;
use crate::models::Aircraft;
use crate::traits::AircraftOperations;
use crate::errors::Error;

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

pub fn ask_mark_flown<T: AircraftOperations, F: Fn() -> Result<char, std::io::Error>>(
    database_connections: &mut T,
    aircraft: &mut Aircraft,
    ask_char_fn: F,
) -> Result<(), Error> {
    if matches!(ask_char_fn(), Ok('y')) {
        aircraft.date_flown = Some(chrono::Local::now().format("%Y-%m-%d").to_string());
        aircraft.flown = 1;
        database_connections.update_aircraft(aircraft)?;
    }

    Ok(())
}
