
use diesel::prelude::*;
use diesel::result::Error;
use crate::models::*;

#[cfg(test)]
pub fn insert_runway(
    connection: &mut SqliteConnection,
    record: &Runway,
) -> Result<(), Error> {
    use crate::schema::Runways::dsl::*;
    
    diesel::insert_into(Runways)
        .values(record)
        .execute(connection)?;
    Ok(())
}

pub fn get_runways_for_airport(
    connection: &mut SqliteConnection,
    airport: &Airport,
) -> Result<Vec<Runway>, Error> {
    let runways = Runway::belonging_to(airport).load(connection)?;
    Ok(runways)
}