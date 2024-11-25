use crate::models::*;

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
    use crate::DatabaseConnections;
    use diesel::prelude::*;

    impl DatabaseConnections {
        pub fn insert_runway(&mut self, record: &Runway) -> Result<(), ValidationError> {
            use crate::schema::Runways::dsl::*;

            if record.Ident.is_empty() || record.Length < 0 {
                return Err(ValidationError::InvalidData(
                    "Ident and length cannot be empty".to_string(),
                ));
            }

            diesel::insert_into(Runways)
                .values(record)
                .execute(&mut self.airport_connection)
                .map_err(|e| ValidationError::DatabaseError(e.to_string()))?;

            Ok(())
        }
    }
}
