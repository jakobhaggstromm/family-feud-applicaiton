use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct BuzzRequest {
    pub token: String,
}

#[derive(Clone, Serialize)]
pub struct Player {
    pub id: u32,
    pub name: String,
    pub token: String,      // unique token for reconnects
    pub is_active: bool,
}

#[derive(Deserialize)]
pub struct PlayerRequest {
    pub name: String,
    pub token: Option<String>, // optional token for reconnects
}

#[derive(Serialize)]
pub struct PlayerJoinResponse {
    pub token: Option<String>, // optional token for reconnects
}

#[derive(Deserialize)]
pub struct PlayerLeaveRequest {
    pub token: String,
}
#[derive(Serialize)]
pub struct WinnerResponse {
    pub id: u32,
    pub name: String,
}

#[derive(Serialize)]
pub struct StateResponse {
    pub active: bool,
    pub winner: Option<WinnerResponse>,
    pub players: Vec<Player>,
}
