use axum::{extract::State, http::StatusCode, Json, response::IntoResponse};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::models::{BuzzRequest, StateResponse, WinnerResponse, PlayerLeaveRequest, PlayerJoinResponse};
use crate::state::RoundState;

pub type SharedState = Arc<Mutex<RoundState>>;
use crate::models::{Player, PlayerRequest};

pub async fn start_round(State(state): State<SharedState>) {
    let mut state = state.lock().unwrap();

    state.active = true;
    state.winner = None;
    println!("Starting round");
}

pub async fn buzz(
    State(state): State<SharedState>,
    Json(payload): Json<BuzzRequest>,
) -> StatusCode {
    let mut state = state.lock().unwrap();

    // Round not active
    if !state.active {
        return StatusCode::BAD_REQUEST;
    }

    // Check player exists
    if !state.players.iter().any(|p| p.token == payload.token) {
        return StatusCode::NOT_FOUND;
    }

    // Someone already buzzed
    if state.winner.is_some() {
        println!("Buzzers already buzzed");
        return StatusCode::CONFLICT;
    }

    // First buzz wins
    state.active = false;


    state.winner = Some(state.players.iter().find(|p| p.token == payload.token).unwrap().id);

    println!("Player {} buzzed first!", state.winner.unwrap());

    StatusCode::OK
}

pub async fn get_state(State(state): State<SharedState>) -> Json<StateResponse> {
    let state = state.lock().unwrap();

    let winner = match state.winner {
        Some(id) => state
            .players
            .iter()
            .find(|p| p.id == id)
            .map(|p| WinnerResponse {
                id: p.id,
                name: p.name.clone(),
            }),
        None => None,
    };
    println!("Round active: {}", state.active);

    if let Some(ref w) = winner {
        println!("Winner: {} (id {})", w.name, w.id);
    } else {
        println!("No winner yet");
    }

    Json(StateResponse {
        active: state.active,
        winner,
        players: state
            .players
            .iter()
            // .filter(|p| p.is_active)
            .cloned()
            .collect(),
    })
}

pub async fn join(
    State(state): State<SharedState>,
    Json(payload): Json<PlayerRequest>,
) -> impl IntoResponse {
    let mut state = state.lock().unwrap();

    // Reconnect with token
    if let Some(ref token) = payload.token {
        if let Some(player) = state.players.iter_mut().find(|p| &p.token == token) {
            player.is_active = true; // reactivate
            println!("Player {} ({}) reconnected", player.id, player.name);
            return (StatusCode::OK, Json(PlayerJoinResponse { token: Some(player.token.clone()) }));
        }
    }

    // Duplicate name check
    if state.players.iter().any(|p| p.name == payload.name && p.is_active) {
        return (StatusCode::CONFLICT, Json(PlayerJoinResponse { token: None }));
    }

    // Create new player
    let player = Player {
        id: state.next_player_id,
        name: payload.name.clone(),
        token: Uuid::new_v4().to_string(),
        is_active: true,
    };
    state.next_player_id += 1;
    state.players.push(player.clone());

    println!("Player {} ({}) joined with token {}", player.id, player.name, player.token);

    (StatusCode::CREATED, Json(PlayerJoinResponse { token: Some(player.token) }))
}

pub async fn leave(
    State(state): State<SharedState>,
    Json(payload): Json<PlayerLeaveRequest>,
) -> StatusCode {
    let mut state = state.lock().unwrap();

    // Find the player and mark them inactive
    if let Some(player) = state.players.iter_mut().find(|p| p.token == payload.token) {
        let player_id = player.id;     // clone the ID
        let player_name = player.name.clone();
        player.is_active = false;

        println!("Player {} ({}) has left the game", player_id, player_name);

        if state.winner == Some(player_id) {
            println!("The winner left, clearing winner and stopping round");
            state.winner = None;
            state.active = false;
        }

        StatusCode::OK
    } else {
        println!("Player not found");
        StatusCode::NOT_FOUND
    }
}
