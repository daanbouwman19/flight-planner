#[cfg(not(target_arch = "wasm32"))]
use diesel::prelude::*;

#[cfg_attr(
    not(target_arch = "wasm32"),
    derive(Queryable, Insertable, AsChangeset)
)]
#[cfg_attr(not(target_arch = "wasm32"), diesel(table_name = crate::schema::settings))]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Setting {
    pub key: String,
    pub value: String,
}
