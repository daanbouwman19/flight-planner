use crate::models::*;
use diesel::prelude::*;
use diesel::result::Error;

pub fn get_runways_for_airport(
    connection: &mut SqliteConnection,
    airport: &Airport,
) -> Result<Vec<Runway>, Error> {
    let runways = Runway::belonging_to(airport).load(connection)?;

    Ok(runways)
}

pub fn format_runway(runway: &Runway) -> String {
    format!(
        "Runway: {}, heading: {:.2}, length: {} ft, width: {} ft, surface: {}, elevation: {}ft",
        runway.Ident,
        runway.TrueHeading,
        runway.Length,
        runway.Width,
        runway.Surface,
        runway.Elevation
    )
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::errors::ValidationError;

    pub fn insert_runway(
        connection: &mut SqliteConnection,
        record: &Runway,
    ) -> Result<(), ValidationError> {
        use crate::schema::Runways::dsl::*;

        if record.Ident.is_empty() || record.Length < 0 {
            return Err(ValidationError::InvalidData(
                "Ident and length cannot be empty".to_string(),
            ));
        }

        diesel::insert_into(Runways)
            .values(record)
            .execute(connection)
            .map_err(|e| ValidationError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}
