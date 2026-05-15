/// A summary of the user's flight history statistics.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct FlightStatistics {
    /// The total number of flights recorded in the history.
    pub total_flights: usize,
    /// The total distance of all flights, in nautical miles.
    pub total_distance: i32,
    /// The name of the aircraft that has been flown the most times.
    pub most_flown_aircraft: Option<String>,
    /// The ICAO code of the airport that has been visited most frequently.
    pub most_visited_airport: Option<String>,
    /// The average distance of a single flight, in nautical miles.
    pub average_flight_distance: f64,
    /// A string representing the longest flight, e.g., "ICAO to ICAO".
    pub longest_flight: Option<String>,
    /// A string representing the shortest flight, e.g., "ICAO to ICAO".
    pub shortest_flight: Option<String>,
    /// The ICAO code of the most frequent departure airport.
    pub favorite_departure_airport: Option<String>,
    /// The ICAO code of the most frequent arrival airport.
    pub favorite_arrival_airport: Option<String>,
}
