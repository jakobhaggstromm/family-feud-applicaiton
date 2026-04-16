use serde::{Deserialize, Serialize};

// ───────────────────────────────────────────────
//  Incoming WebSocket messages (client → server)
// ───────────────────────────────────────────────

#[derive(Deserialize, Debug)]
#[serde(tag = "action")]
pub enum ClientMessage {
    // ── Team joins by name, reconnects with token ──
    #[serde(rename = "join")]
    Join { name: String, token: Option<String> },
    #[serde(rename = "leave")]
    Leave { token: String },
    #[serde(rename = "claim_admin")]
    ClaimAdmin {
        token: Option<String>,
        password: Option<String>,
    },

    // ── Host: remove a team ──
    #[serde(rename = "remove_team")]
    RemoveTeam { team_id: u32 },

    // ── Game flow (host) ──
    #[serde(rename = "start_game")]
    StartGame,
    #[serde(rename = "load_questions")]
    LoadQuestions { filename: String },
    #[serde(rename = "start_round")]
    StartRound,
    #[serde(rename = "award_point")]
    AwardPoint { team_id: u32, answer_index: usize },
    #[serde(rename = "next_question")]
    NextQuestion,
    #[serde(rename = "reset_game")]
    ResetGame,

    // ── Buzzer (team device sends token) ──
    #[serde(rename = "buzz")]
    Buzz { token: String },

    // ── Host judging ──
    #[serde(rename = "strike")]
    Strike,

    // ── State ──
    #[serde(rename = "get_state")]
    GetState,
}

#[derive(Deserialize, Debug, Clone)]
pub struct QuestionInput {
    #[serde(alias = "question")]
    pub text: String,
    pub answers: Vec<AnswerInput>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AnswerInput {
    #[serde(alias = "answer")]
    pub text: String,
    pub points: u32,
}

// ───────────────────────────────────────────────
//  Outgoing WebSocket messages (server → client)
// ───────────────────────────────────────────────

#[derive(Serialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "admin_auth_result")]
    AdminAuthResult {
        success: bool,
        token: Option<String>,
        error: Option<String>,
    },
    #[serde(rename = "join_result")]
    JoinResult {
        success: bool,
        token: Option<String>,
        error: Option<String>,
    },
    #[serde(rename = "leave_result")]
    LeaveResult {
        success: bool,
        error: Option<String>,
    },
    #[serde(rename = "buzz_result")]
    BuzzResult {
        success: bool,
        error: Option<String>,
    },
    /// Generic success/error for host actions.
    #[serde(rename = "action_result")]
    ActionResult {
        success: bool,
        error: Option<String>,
    },
    #[serde(rename = "state")]
    State(StateResponse),
    #[serde(rename = "error")]
    Error { message: String },
}

// ───────────────────────────────────────────────
//  Domain types
// ───────────────────────────────────────────────

#[derive(Clone, Serialize, Debug)]
pub struct Team {
    pub id: u32,
    pub name: String,
    pub score: u32,
    /// Secret token — the team device uses this to buzz.
    #[serde(skip_serializing)]
    pub token: String,
    pub is_active: bool,
}

#[derive(Clone, Debug)]
pub struct Answer {
    pub text: String,
    pub points: u32,
    pub revealed: bool,
}

#[derive(Clone, Debug)]
pub struct Question {
    pub text: String,
    pub answers: Vec<Answer>,
}

/// The phases a round moves through.
#[derive(Clone, Serialize, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum GamePhase {
    /// Setting up teams / loading questions.
    Lobby,
    /// Teams are buzzing to win control.
    Play,
    /// The controlling team is guessing answers.
    Answer,
    /// Round finished — points awarded.
    RoundOver,
    /// All questions played.
    GameOver,
}

// ───────────────────────────────────────────────
//  State response sent to every client
// ───────────────────────────────────────────────

#[derive(Serialize, Clone, Debug)]
pub struct StateResponse {
    pub phase: GamePhase,
    pub teams: Vec<Team>,
    pub current_question: Option<QuestionResponse>,
    pub controlling_team_id: Option<u32>,
    pub strikes: u32,
    pub round_points: u32,
    pub current_question_index: usize,
    pub total_questions: usize,
}

/// Question as seen by clients — unrevealed answer text is hidden.
#[derive(Serialize, Clone, Debug)]
pub struct QuestionResponse {
    pub text: String,
    pub answers: Vec<AnswerResponse>,
}

#[derive(Serialize, Clone, Debug)]
pub struct AnswerResponse {
    pub points: u32,
    pub revealed: bool,
    /// `None` when the answer has not been revealed yet (player view).
    pub text: Option<String>,
    /// Full text visible only to admin.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_text: Option<String>,
}
