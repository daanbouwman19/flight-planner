use diesel::prelude::*;

#[derive(Queryable, Insertable, AsChangeset, Debug, Clone)]
#[diesel(table_name = crate::schema::settings)]
pub struct Setting {
    pub key: String,
    pub value: String,
}
