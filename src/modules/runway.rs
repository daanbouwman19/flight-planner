use diesel::RunQueryDsl;

use crate::database::DatabasePool;
use crate::models::Runway;

impl DatabasePool {
    /// Retrieves all runways from the database.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `Vec<Runway>` on success, or a `diesel::result::Error`
    /// on failure.
    pub fn get_runways(&self) -> Result<Vec<Runway>, diesel::result::Error> {
        use crate::schema::Runways::dsl::Runways;
        let conn = &mut self.airport_pool.get().unwrap();

        let records: Vec<Runway> = Runways.get_results(conn)?;

        Ok(records)
    }
}

/// Formats a `Runway` struct into a human-readable string.
///
/// # Arguments
///
/// * `runway` - A reference to the `Runway` struct to format.
///
/// # Returns
///
/// A `String` containing the formatted runway details.
pub fn format_runway(runway: &Runway) -> String {
    format!(
        "Runway: {}, heading: {:.2}, length: {} ft, width: {} ft, surface: {}, elevation: {} ft",
        runway.Ident,
        runway.TrueHeading,
        runway.Length,
        runway.Width,
        runway.Surface,
        runway.Elevation
    )
}
