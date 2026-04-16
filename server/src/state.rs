use tokio::sync::broadcast;
use uuid::Uuid;

use crate::models::{GamePhase, Question, ServerMessage, Team};

pub struct GameState {
    // ── Teams ──
    pub teams: Vec<Team>,
    pub next_team_id: u32,
    pub admin_token: Option<String>,
    pub admin_password: String,

    // ── Questions ──
    pub questions: Vec<Question>,
    pub current_question_index: usize,

    // ── Round state ──
    pub phase: GamePhase,
    /// Seconds remaining in countdown (only relevant during Countdown phase).
    pub countdown_seconds: u8,
    /// The team currently answering (or whose points are at stake).
    pub controlling_team_id: Option<u32>,
    /// Wrong guesses this round (max 3 → steal).
    pub strikes: u32,
    /// The teams that have answered this round.
    pub answered_teams: Vec<u32>,
    // ── Broadcast ──
    pub broadcast_tx: broadcast::Sender<ServerMessage>,
}

impl GameState {
    pub fn new(admin_password: String) -> Self {
        let (tx, _) = broadcast::channel(100);
        Self {
            teams: Vec::new(),
            next_team_id: 1,
            admin_token: None,
            admin_password,
            questions: Vec::new(),
            current_question_index: 0,
            phase: GamePhase::Lobby,
            countdown_seconds: 0,
            controlling_team_id: None,
            strikes: 0,
            answered_teams: Vec::new(),
            broadcast_tx: tx,
        }
    }

    // ── Helpers ──

    pub fn current_question(&self) -> Option<&Question> {
        self.questions.get(self.current_question_index)
    }

    pub fn current_question_mut(&mut self) -> Option<&mut Question> {
        self.questions.get_mut(self.current_question_index)
    }

    /// Sum of points from revealed answers in the current question.
    pub fn round_points(&self) -> u32 {
        self.current_question()
            .map(|q| {
                q.answers
                    .iter()
                    .filter(|a| a.revealed)
                    .map(|a| a.points)
                    .sum()
            })
            .unwrap_or(0)
    }

    /// True when every answer in the current question has been revealed.
    pub fn all_answers_revealed(&self) -> bool {
        self.current_question()
            .map(|q| q.answers.iter().all(|a| a.revealed))
            .unwrap_or(false)
    }

    // GETTERS / MUTATORS

    /// Check if a team with the given ID exists.
    pub fn get_team(&self, team_id: u32) -> Option<&Team> {
        self.teams.iter().find(|t| t.id == team_id)
    }

    /// Check if a team with the given token exists.
    pub fn get_team_by_token(&self, token: &str) -> Option<&Team> {
        self.teams.iter().find(|t| t.token == token)
    }

    pub fn get_team_mut(&mut self, team_id: u32) -> Option<&mut Team> {
        self.teams.iter_mut().find(|t| t.id == team_id)
    }

    pub fn claim_admin(&mut self, requested_token: Option<String>) -> Result<String, String> {
        if let Some(existing_token) = &self.admin_token {
            if let Some(token) = requested_token {
                if token == *existing_token {
                    return Ok(token);
                }
            }

            return Err("Admin page is already in use".to_string());
        }

        let token = requested_token.unwrap_or_else(|| Uuid::new_v4().to_string());
        self.admin_token = Some(token.clone());
        Ok(token)
    }

    pub fn verify_admin_password(&self, password: &str) -> bool {
        self.admin_password == password
    }

    pub fn release_admin(&mut self, token: &str) {
        if self.admin_token.as_deref() == Some(token) {
            self.admin_token = None;
        }
    }

    pub fn is_admin(&self, token: Option<&str>) -> bool {
        matches!((&self.admin_token, token), (Some(active_token), Some(provided_token)) if active_token == provided_token)
    }

    /// Award a point to the specified team. Returns `true` if successful, `false` if team ID not found.
    pub fn award_point(&mut self, team_id: u32, points: u32) -> bool {
        if let Some(team) = self.get_team_mut(team_id) {
            team.score += points;
            true
        } else {
            false
        }
    }

    /// Reset per-round fields for a new question.
    pub fn reset_round(&mut self) {
        self.controlling_team_id = None;
        self.strikes = 0;
        self.answered_teams.clear();
    }

    pub fn has_answered(&self, team_id: u32) -> bool {
        self.answered_teams.contains(&team_id)
    }
}
