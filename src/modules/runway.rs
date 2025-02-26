use diesel::RunQueryDsl;

use crate::database::DatabasePool;
use crate::models::*;

impl DatabasePool {
    pub fn get_runways(&self) -> Result<Vec<Runway>, diesel::result::Error> {
        use crate::schema::Runways::dsl::*;
        let conn = &mut self.airport_pool.get().unwrap();

        let records: Vec<Runway> = Runways.get_results(conn)?;

        Ok(records)
    }
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
