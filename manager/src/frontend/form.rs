use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TSCreate {
    pub ip: String,
    pub port: Option<u16>,
    pub cid: Option<i32>,
    pub password: Option<String>,
}
