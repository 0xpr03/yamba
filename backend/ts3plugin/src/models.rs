/// Generic Request who require an instance ID
#[derive(Serialize)]
pub struct HeartbeatRequest {
    pub id: i32,
}

#[derive(Deserialize)]
pub struct HeartbeatResponse {
    pub success: bool,
}

#[derive(Serialize)]
pub struct ConnectedRequest {
    pub id: i32,
    pub pid: u32,
}

#[derive(Deserialize)]
pub struct ConnectedResponse {
    pub success: bool,
}
