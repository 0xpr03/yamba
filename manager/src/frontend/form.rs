use serde::Deserialize;

use yamba_types::ID;

#[derive(Debug, Deserialize)]
pub struct TSCreate {
    pub host: String,
    #[serde(skip_deserializing)]
    pub port: Option<u16>,
    #[serde(skip_deserializing)]
    pub cid: Option<i32>,
    #[serde(skip_deserializing)]
    pub password: Option<String>,
    pub id: ID,
    pub name: String,
}
