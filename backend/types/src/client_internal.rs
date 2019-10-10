use serde::{Deserialize, Serialize};

/// Heartbeat, client -> daemon
#[derive(Deserialize, Serialize)]
pub struct Heartbeat {
    pub id: i32,
}

/// Heartbeat response, daemon -> client
#[derive(Deserialize, Serialize)]
pub struct HeartbeatResponse {
    pub success: bool,
    /// current name to use
    pub name: String,
    /// current channel to use
    pub cid: u16,
}

/// Connected, client -> daemon
#[derive(Deserialize, Serialize)]
pub struct Connected {
    pub id: i32,
    pub pid: u32,
}

/// Connected response, daemon -> client
#[derive(Deserialize, Serialize)]
pub struct ConnectedResponse {
    pub success: bool,
}
