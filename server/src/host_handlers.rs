use crate::handlers::SharedState;
use crate::models::{GamePhase, ServerMessage};
use crate::readers::read_questions_from_file;
use std::path::PathBuf;
// ───────────────────────────────────────────────
//  Team removal (host)
// ───────────────────────────────────────────────

pub fn handle_remove_team(state: &SharedState, team_id: u32) -> ServerMessage {
    let mut s = state.lock().unwrap();

    if s.phase != GamePhase::Lobby {
        return ServerMessage::ActionResult {
            success: false,
            error: Some("Can only remove teams in the lobby".to_string()),
        };
    }

    let before = s.teams.len();
    s.teams.retain(|t| t.id != team_id);

    if s.teams.len() < before {
        println!("Removed team {}", team_id);
        ServerMessage::ActionResult {
            success: true,
            error: None,
        }
    } else {
        ServerMessage::ActionResult {
            success: false,
            error: Some("Team not found".to_string()),
        }
    }
}

// ───────────────────────────────────────────────
//  Game flow
// ───────────────────────────────────────────────

pub fn handle_start_game(state: &SharedState) -> ServerMessage {
    let mut s = state.lock().unwrap();

    if s.teams.len() < 2 {
        return ServerMessage::ActionResult {
            success: false,
            error: Some("Need at least 2 teams to start".to_string()),
        };
    }

    let questions_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data")
        .join("questions")
        .join("questions.json");

    match read_questions_from_file(questions_path.to_string_lossy().as_ref()) {
        Ok(questions) => {
            s.questions = questions;
        }
        Err(err) => {
            println!(
                "Failed to load questions from {}: {}",
                questions_path.to_string_lossy(),
                err
            );
            return ServerMessage::ActionResult {
                success: false,
                error: Some(format!("Failed to load questions: {}", err)),
            };
        }
    }

    s.current_question_index = 0;
    s.teams.iter_mut().for_each(|team| team.score = 0);
    s.phase = GamePhase::Play;
    s.reset_round();
    println!("Game started!");

    ServerMessage::ActionResult {
        success: true,
        error: None,
    }
}

pub fn handle_load_questions(state: &SharedState, filename: &str) -> ServerMessage {
    let mut s = state.lock().unwrap();

    let data_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("data")
        .join("questions");
    let questions_path = data_dir.join(filename);

    if !questions_path.starts_with(&data_dir) {
        return ServerMessage::ActionResult {
            success: false,
            error: Some("Invalid file path".to_string()),
        };
    }

    match read_questions_from_file(questions_path.to_string_lossy().as_ref()) {
        Ok(questions) => {
            s.questions = questions;
            s.current_question_index = 0;
            s.reset_round();
            s.phase = GamePhase::Lobby;
            println!("Loaded questions from {}", filename);
            ServerMessage::ActionResult {
                success: true,
                error: None,
            }
        }
        Err(err) => {
            println!("Failed to load questions from {}: {}", filename, err);
            ServerMessage::ActionResult {
                success: false,
                error: Some(format!("Failed to load questions: {}", err)),
            }
        }
    }
}

pub fn handle_start_round(state: &SharedState) -> ServerMessage {
    let mut s = state.lock().unwrap();

    if s.phase != GamePhase::Lobby && s.phase != GamePhase::RoundOver {
        return ServerMessage::ActionResult {
            success: false,
            error: Some("Cannot start a round in current phase".to_string()),
        };
    }

    s.phase = GamePhase::Play;
    s.reset_round();
    println!(
        "Face-off started for question {}",
        s.current_question_index + 1
    );

    ServerMessage::ActionResult {
        success: true,
        error: None,
    }
}

pub fn handle_next_question(state: &SharedState) -> ServerMessage {
    let mut s = state.lock().unwrap();

    s.current_question_index += 1;
    s.reset_round();

    if s.current_question_index >= s.questions.len() {
        s.phase = GamePhase::GameOver;
        println!("No more questions — game over!");
    } else {
        s.phase = GamePhase::Play;
        println!(
            "Moving to question {} of {}",
            s.current_question_index + 1,
            s.questions.len()
        );
    }

    ServerMessage::ActionResult {
        success: true,
        error: None,
    }
}

pub fn handle_reset_game(state: &SharedState) -> ServerMessage {
    let mut s = state.lock().unwrap();

    s.teams.iter_mut().for_each(|t| t.score = 0);
    s.questions.clear();
    s.current_question_index = 0;
    s.reset_round();
    s.phase = GamePhase::Lobby;
    println!("Game reset");

    ServerMessage::ActionResult {
        success: true,
        error: None,
    }
}

// ───────────────────────────────────────────────
//  Host judging
// ───────────────────────────────────────────────

pub fn handle_award_point(state: &SharedState, team_id: u32, answer_index: usize) -> ServerMessage {
    let mut s = state.lock().unwrap();

    if s.phase != GamePhase::Answer {
        return ServerMessage::ActionResult {
            success: false,
            error: Some("Cannot award points in current phase".to_string()),
        };
    }

    if s.get_team(team_id).is_none() {
        return ServerMessage::ActionResult {
            success: false,
            error: Some("Team not found".to_string()),
        };
    }

    let points = s
        .current_question()
        .and_then(|q| q.answers.get(answer_index))
        .map(|a| a.points);

    if let Some(pts) = points {
        s.award_point(team_id, pts);
        if let Some(question) = s.current_question_mut() {
            if let Some(answer) = question.answers.get_mut(answer_index) {
                answer.revealed = true;
            }
        }
    }
    // If all answers revealed, award points and end round
    if s.all_answers_revealed() {
        s.phase = GamePhase::RoundOver;
    }

    println!("Awarded point to team {}", team_id);

    ServerMessage::ActionResult {
        success: true,
        error: None,
    }
}

pub fn handle_strike(state: &SharedState) -> ServerMessage {
    let mut s = state.lock().unwrap();

    if s.phase != GamePhase::Answer {
        return ServerMessage::ActionResult {
            success: false,
            error: Some("Not in answer phase".to_string()),
        };
    }

    s.strikes += 1;
    println!("Strike {}!", s.strikes);

    if s.strikes >= 3 {
        s.phase = GamePhase::Play;
        let controlling_team_id = s.controlling_team_id.unwrap_or(0);
        s.answered_teams.push(controlling_team_id);
        println!("3 strikes — steal opportunity!");
        if s.answered_teams.len() == s.teams.len() {
            s.reset_round();
        } else {
            s.controlling_team_id = None;
            s.strikes = 0;
        }
    }

    ServerMessage::ActionResult {
        success: true,
        error: None,
    }
}
