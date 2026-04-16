use uuid::Uuid;

use crate::handlers::SharedState;
use crate::models::{GamePhase, ServerMessage, Team};

// ───────────────────────────────────────────────
//  Team join / leave
// ───────────────────────────────────────────────

pub fn handle_join(state: &SharedState, name: String, token: Option<String>) -> ServerMessage {
    let mut s = state.lock().unwrap();

    // Reconnect with token
    if let Some(ref tok) = token {
        if let Some(team) = s.teams.iter_mut().find(|t| &t.token == tok) {
            team.is_active = true;
            println!("Team {} ({}) reconnected", team.id, team.name);
            return ServerMessage::JoinResult {
                success: true,
                token: Some(team.token.clone()),
                error: None,
            };
        }
    }

    // Duplicate name check
    if s.teams.iter().any(|t| t.name == name && t.is_active) {
        return ServerMessage::JoinResult {
            success: false,
            token: None,
            error: Some("Team name already taken".to_string()),
        };
    }

    let team = Team {
        id: s.next_team_id,
        name: name.clone(),
        score: 0,
        token: Uuid::new_v4().to_string(),
        is_active: true,
    };
    s.next_team_id += 1;
    println!(
        "Team {} ({}) joined with token {}",
        team.id, team.name, team.token
    );
    let token = team.token.clone();
    s.teams.push(team);

    ServerMessage::JoinResult {
        success: true,
        token: Some(token),
        error: None,
    }
}

pub fn handle_leave(state: &SharedState, token: String) -> ServerMessage {
    let mut s = state.lock().unwrap();

    if let Some(team) = s.teams.iter_mut().find(|t| t.token == token) {
        team.is_active = false;
        println!("Team {} ({}) left", team.id, team.name);
        ServerMessage::LeaveResult {
            success: true,
            error: None,
        }
    } else {
        ServerMessage::LeaveResult {
            success: false,
            error: Some("Team not found".to_string()),
        }
    }
}

// ───────────────────────────────────────────────
//  Buzzer
// ───────────────────────────────────────────────

pub fn handle_buzz(state: &SharedState, token: String) -> ServerMessage {
    let mut s = state.lock().unwrap();

    if s.phase != GamePhase::Play {
        return ServerMessage::BuzzResult {
            success: false,
            error: Some("Not in play phase".to_string()),
        };
    }

    let team = match s.get_team_by_token(&token) {
        Some(t) => t,
        None => {
            return ServerMessage::BuzzResult {
                success: false,
                error: Some("Invalid team token".to_string()),
            }
        }
    };

    if s.controlling_team_id.is_some() {
        return ServerMessage::BuzzResult {
            success: false,
            error: Some("Another team already buzzed".to_string()),
        };
    }

    if s.has_answered(team.id) {
        return ServerMessage::BuzzResult {
            success: false,
            error: Some("Team has already answered".to_string()),
        };
    }

    println!("Team {} ({}) buzzed first!", team.id, team.name);
    s.controlling_team_id = Some(team.id);
    s.phase = GamePhase::Answer;

    ServerMessage::BuzzResult {
        success: true,
        error: None,
    }
}
