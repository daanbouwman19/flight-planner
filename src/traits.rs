use crate::errors::{AirportSearchError, Error};
use crate::models::{Aircraft, Airport, History, NewAircraft, Runway};

/// Defines a set of operations for managing aircraft data.
///
/// This trait abstracts the database interactions for aircraft-related
/// functionalities, allowing for different implementations (e.g., direct
/// connection vs. connection pool).
pub trait AircraftOperations {
    /// Gets the number of aircraft that have not been flown.
    fn get_not_flown_count(&mut self) -> Result<i64, Error>;
    /// Retrieves a random aircraft that has not been flown.
    fn random_not_flown_aircraft(&mut self) -> Result<Aircraft, Error>;
    /// Retrieves all aircraft from the database.
    fn get_all_aircraft(&mut self) -> Result<Vec<Aircraft>, Error>;
    /// Updates an existing aircraft record in the database.
    fn update_aircraft(&mut self, record: &Aircraft) -> Result<(), Error>;
    /// Retrieves a random aircraft from the database, regardless of its flown status.
    fn random_aircraft(&mut self) -> Result<Aircraft, Error>;
    /// Retrieves a specific aircraft by its unique ID.
    fn get_aircraft_by_id(&mut self, aircraft_id: i32) -> Result<Aircraft, Error>;
    /// Resets the flown status of all aircraft to not flown.
    fn mark_all_aircraft_not_flown(&mut self) -> Result<(), Error>;

    #[allow(dead_code)] // Used in integration tests
    /// Adds a new aircraft record to the database.
    fn add_aircraft(&mut self, record: &NewAircraft) -> Result<Aircraft, Error>;
}

/// Defines a set of operations for managing airport data, building upon `AircraftOperations`.
pub trait AirportOperations: AircraftOperations {
    /// Retrieves a random airport from the database.
    fn get_random_airport(&mut self) -> Result<Airport, AirportSearchError>;
    /// Finds a suitable destination airport for a given aircraft and departure airport.
    fn get_destination_airport(
        &mut self,
        aircraft: &Aircraft,
        departure: &Airport,
    ) -> Result<Airport, AirportSearchError>;
    /// Retrieves a random airport that has a runway suitable for the given aircraft.
    fn get_random_airport_for_aircraft(
        &mut self,
        aircraft: &Aircraft,
    ) -> Result<Airport, AirportSearchError>;
    /// Retrieves all runways for a specific airport.
    fn get_runways_for_airport(
        &mut self,
        airport: &Airport,
    ) -> Result<Vec<Runway>, AirportSearchError>;
    /// Finds a destination airport with a suitable runway within a given distance.
    fn get_destination_airport_with_suitable_runway(
        &mut self,
        departure: &Airport,
        max_distance_nm: i32,
        min_takeoff_distance_m: i32,
    ) -> Result<Airport, AirportSearchError>;
    /// Finds any airport within a specified distance from a departure airport.
    fn get_airport_within_distance(
        &mut self,
        departure: &Airport,
        max_distance_nm: i32,
    ) -> Result<Airport, AirportSearchError>;
    /// Retrieves all airports from the database.
    fn get_airports(&mut self) -> Result<Vec<Airport>, AirportSearchError>;

    #[allow(dead_code)] // Used in integration tests
    /// Retrieves a specific airport by its ICAO code.
    fn get_airport_by_icao(&mut self, icao: &str) -> Result<Airport, AirportSearchError>;
}

/// Defines a set of operations for managing flight history data.
pub trait HistoryOperations {
    /// Adds a new flight record to the history.
    fn add_to_history(
        &mut self,
        departure: &Airport,
        arrival: &Airport,
        aircraft_record: &Aircraft,
    ) -> Result<(), Error>;
    /// Retrieves all flight history records, ordered by most recent first.
    fn get_history(&mut self) -> Result<Vec<History>, Error>;
}

/// A marker trait that combines all database operation traits into a single bound.
///
/// This trait simplifies trait bounds for functions and structs that require
/// comprehensive database access.
pub trait DatabaseOperations: AircraftOperations + AirportOperations + HistoryOperations {}

/// A trait for items that can be searched.
pub trait Searchable {
    /// Returns a score indicating how well the item matches the query.
    /// A higher score indicates a better match.
    /// A score of 0 indicates no match.
    fn search_score(&self, query: &str) -> u8 {
        self.search_score_lower(&query.to_lowercase())
    }

    /// Returns a score using a pre-lowercased query for performance.
    /// This prevents allocating a new string for every item during a search.
    fn search_score_lower(&self, query_lower: &str) -> u8;
}
