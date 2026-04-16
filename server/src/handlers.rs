use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use std::sync::{Arc, Mutex};

use crate::models::{
    AnswerResponse, ClientMessage, GamePhase, QuestionResponse, ServerMessage, StateResponse,
};
use crate::state::GameState;

use crate::host_handlers::{
    handle_award_point, handle_load_questions, handle_next_question, handle_remove_team,
    handle_reset_game, handle_start_game, handle_start_round, handle_strike,
};
use crate::player_handlers::{handle_buzz, handle_join, handle_leave};

pub type SharedState = Arc<Mutex<GameState>>;

// ───────────────────────────────────────────────
//  WebSocket plumbing
// ───────────────────────────────────────────────

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: SharedState) {
    let (mut sender, mut receiver) = socket.split();
    let mut socket_admin_token: Option<String> = None;

    let mut broadcast_rx = {
        let s = state.lock().unwrap();
        s.broadcast_tx.subscribe()
    };

    loop {
        tokio::select! {
            recv_result = broadcast_rx.recv() => {
                match recv_result {
                    Ok(msg) => {
                        if send_json(&mut sender, &msg).await.is_err() {
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
            maybe_msg = receiver.next() => {
                match maybe_msg {
                    Some(Ok(Message::Text(text))) => {
                        let parsed: Result<ClientMessage, _> = serde_json::from_str(&text);
                        match parsed {
                            Ok(client_msg) => {
                                let outcome = process_message(client_msg, &state, socket_admin_token.as_deref());

                                if let Some(admin_token) = outcome.admin_token.clone() {
                                    socket_admin_token = Some(admin_token);
                                }

                                if send_json(&mut sender, &outcome.response).await.is_err() {
                                    break;
                                }

                                if outcome.broadcast_state {
                                    broadcast_state(&state);
                                }
                            }
                            Err(e) => {
                                println!("Failed to parse message: {}", e);
                                let err_msg = ServerMessage::Error {
                                    message: format!("Invalid message: {}", e),
                                };

                                if send_json(&mut sender, &err_msg).await.is_err() {
                                    break;
                                }
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(_)) => {}
                    Some(Err(_)) => break,
                }
            }
        }
    }

    if let Some(admin_token) = socket_admin_token {
        state.lock().unwrap().release_admin(&admin_token);
    }

    println!("WebSocket connection closed");
}

// ───────────────────────────────────────────────
//  Message router
// ───────────────────────────────────────────────

struct MessageOutcome {
    response: ServerMessage,
    broadcast_state: bool,
    admin_token: Option<String>,
}

fn process_message(
    msg: ClientMessage,
    state: &SharedState,
    socket_admin_token: Option<&str>,
) -> MessageOutcome {
    match msg {
        ClientMessage::ClaimAdmin { token, password } => {
            let result = {
                let mut s = state.lock().unwrap();

                if let Some(requested_token) = token.clone() {
                    s.claim_admin(Some(requested_token))
                } else if let Some(password) = password {
                    if s.verify_admin_password(&password) {
                        s.claim_admin(None)
                    } else {
                        Err("Invalid admin password".to_string())
                    }
                } else {
                    Err("Admin password required".to_string())
                }
            };

            match result {
                Ok(admin_token) => MessageOutcome {
                    response: ServerMessage::AdminAuthResult {
                        success: true,
                        token: Some(admin_token.clone()),
                        error: None,
                    },
                    broadcast_state: false,
                    admin_token: Some(admin_token),
                },
                Err(error) => MessageOutcome {
                    response: ServerMessage::AdminAuthResult {
                        success: false,
                        token: None,
                        error: Some(error),
                    },
                    broadcast_state: false,
                    admin_token: None,
                },
            }
        }

        // Player actions
        ClientMessage::Join { name, token } => MessageOutcome {
            response: handle_join(state, name, token),
            broadcast_state: true,
            admin_token: None,
        },
        ClientMessage::Leave { token } => MessageOutcome {
            response: handle_leave(state, token),
            broadcast_state: true,
            admin_token: None,
        },
        ClientMessage::Buzz { token } => MessageOutcome {
            response: handle_buzz(state, token),
            broadcast_state: true,
            admin_token: None,
        },

        // Host actions
        ClientMessage::RemoveTeam { team_id } => {
            authorized_host_action(state, socket_admin_token, move |shared_state| {
                handle_remove_team(shared_state, team_id)
            })
        }
        ClientMessage::StartGame => {
            let outcome = authorized_host_action(state, socket_admin_token, handle_start_game);
            if outcome.broadcast_state {
                let phase = state.lock().unwrap().phase.clone();
                if phase == GamePhase::Countdown {
                    spawn_countdown(Arc::clone(state));
                }
            }
            outcome
        }
        ClientMessage::LoadQuestions { filename } => {
            authorized_host_action(state, socket_admin_token, move |shared_state| {
                handle_load_questions(shared_state, &filename)
            })
        }
        ClientMessage::StartRound => {
            authorized_host_action(state, socket_admin_token, handle_start_round)
        }
        ClientMessage::AwardPoint {
            team_id,
            answer_index,
        } => authorized_host_action(state, socket_admin_token, move |shared_state| {
            handle_award_point(shared_state, team_id, answer_index)
        }),
        ClientMessage::NextQuestion => {
            let outcome = authorized_host_action(state, socket_admin_token, handle_next_question);
            if outcome.broadcast_state {
                let phase = state.lock().unwrap().phase.clone();
                if phase == GamePhase::Countdown {
                    spawn_countdown(Arc::clone(state));
                }
            }
            outcome
        }
        ClientMessage::ResetGame => {
            authorized_host_action(state, socket_admin_token, handle_reset_game)
        }
        ClientMessage::Strike => authorized_host_action(state, socket_admin_token, handle_strike),

        // Shared
        ClientMessage::GetState => MessageOutcome {
            response: handle_get_state(state),
            broadcast_state: false,
            admin_token: None,
        },
    }
}

fn authorized_host_action<F>(
    state: &SharedState,
    socket_admin_token: Option<&str>,
    action: F,
) -> MessageOutcome
where
    F: FnOnce(&SharedState) -> ServerMessage,
{
    let is_admin = state.lock().unwrap().is_admin(socket_admin_token);

    if !is_admin {
        return MessageOutcome {
            response: ServerMessage::ActionResult {
                success: false,
                error: Some("Admin access required or admin page is already in use".to_string()),
            },
            broadcast_state: false,
            admin_token: None,
        };
    }

    MessageOutcome {
        response: action(state),
        broadcast_state: true,
        admin_token: None,
    }
}

// ───────────────────────────────────────────────
//  State helpers
// ───────────────────────────────────────────────

fn handle_get_state(state: &SharedState) -> ServerMessage {
    let s = state.lock().unwrap();
    build_state_response(&s, false)
}

pub fn build_state_response(s: &GameState, _is_admin: bool) -> ServerMessage {
    let question_response = s.current_question().map(|q| QuestionResponse {
        text: q.text.clone(),
        answers: q
            .answers
            .iter()
            .map(|a| AnswerResponse {
                points: a.points,
                revealed: a.revealed,
                text: if a.revealed {
                    Some(a.text.clone())
                } else {
                    None
                },
                full_text: Some(a.text.clone()),
            })
            .collect(),
    });

    let countdown_seconds = if s.phase == GamePhase::Countdown {
        Some(s.countdown_seconds)
    } else {
        None
    };

    ServerMessage::State(StateResponse {
        phase: s.phase.clone(),
        teams: s.teams.clone(),
        current_question: question_response,
        controlling_team_id: s.controlling_team_id,
        strikes: s.strikes,
        round_points: s.round_points(),
        current_question_index: s.current_question_index,
        total_questions: s.questions.len(),
        countdown_seconds,
    })
}

fn broadcast_state(state: &SharedState) {
    let s = state.lock().unwrap();
    let state_msg = build_state_response(&s, false);
    let _ = s.broadcast_tx.send(state_msg);
}

fn spawn_countdown(state: SharedState) {
    tokio::spawn(async move {
        for seconds_left in [2u8, 1] {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            let mut s = state.lock().unwrap();
            if s.phase != GamePhase::Countdown {
                return;
            }
            s.countdown_seconds = seconds_left;
            let msg = build_state_response(&s, false);
            let _ = s.broadcast_tx.send(msg);
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        let mut s = state.lock().unwrap();
        if s.phase != GamePhase::Countdown {
            return;
        }
        s.phase = GamePhase::Play;
        s.countdown_seconds = 0;
        let msg = build_state_response(&s, false);
        let _ = s.broadcast_tx.send(msg);
    });
}

async fn send_json(
    sender: &mut futures::stream::SplitSink<WebSocket, Message>,
    msg: &ServerMessage,
) -> Result<(), ()> {
    let json = serde_json::to_string(msg).map_err(|_| ())?;
    sender
        .send(Message::Text(json.into()))
        .await
        .map_err(|_| ())
}
