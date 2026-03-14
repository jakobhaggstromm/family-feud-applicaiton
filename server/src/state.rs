use crate::models::{Player};

pub struct RoundState {
    pub active: bool,
    pub players: Vec<Player>,
    pub winner: Option<u32>,
    pub next_player_id: u32,
}

impl RoundState {
    pub fn new() -> Self {
        Self {
            active: false,
            players: Vec::new(),
            winner: None,
            next_player_id: 1,
        }
    }
}