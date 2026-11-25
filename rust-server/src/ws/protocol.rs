//! WebSocket protocol definitions.
//!
//! All client → server and server → client messages are defined here.

use serde::{Deserialize, Serialize};

// Re-export PlayerAnswer from domain to avoid duplication.
pub use crate::domain::player::PlayerAnswer;

/// Client → Server messages.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ClientMessage {
    /// Create a new game room.
    CreateRoom {
        topic: Option<String>,
        mode: Option<String>,
        settings: Option<RoomSettingsInput>,
    },

    /// Join an existing room by code.
    JoinRoom {
        code: String,
        name: String,
    },

    /// Leave the current room.
    LeaveRoom,

    /// Start the game (host only).
    StartGame,

    /// Submit an answer to the current question.
    SubmitAnswer {
        question_index: usize,
        answer_index: usize,
    },

    /// Send a chat message.
    ChatMessage { text: String },

    /// Use a joker to remove wrong answers.
    UseJoker,

    /// Request a hint for the current question.
    GetHint,
}

/// Room settings input from client.
#[derive(Debug, Deserialize, Clone)]
pub struct RoomSettingsInput {
    pub question_count: Option<usize>,
    pub num_options: Option<usize>,
    pub timer_seconds: Option<u64>,
    pub difficulty: Option<String>,
}

/// Server → Client messages.
#[derive(Debug, Serialize, Clone)]
#[serde(tag = "type", content = "data")]
pub enum ServerMessage {
    /// Room created successfully.
    RoomCreated {
        code: String,
        host: String,
        topic: String,
        mode: String,
    },

    /// Joined a room successfully.
    RoomJoined {
        code: String,
        is_host: bool,
        topic: String,
        mode: String,
        players: Vec<PlayerInfo>,
    },

    /// Left the room.
    RoomLeft {},

    /// Game started.
    GameStarted {},

    /// New question emitted.
    QuestionEmitted {
        question_index: usize,
        question: QuestionData,
        time_limit: u64,
    },

    /// Question result after timeout or all answers.
    QuestionResult {
        question_index: usize,
        correct_index: usize,
        explanation: String,
        player_answers: Vec<PlayerAnswer>,
        leaderboard: Vec<PlayerScore>,
    },

    /// All questions finished, game over.
    GameOver {
        final_leaderboard: Vec<PlayerScore>,
        winner: Option<String>,
    },

    /// Chat message from a player.
    ChatMessage {
        user_id: i64,
        username: String,
        text: String,
        timestamp: i64,
    },

    /// Joker used successfully.
    JokerUsed {
        removed_indices: Vec<usize>,
    },

    /// Hint received.
    HintReceived { hint: String },

    /// Player joined the room.
    PlayerJoined { player: PlayerInfo },

    /// Player left the room.
    PlayerLeft {
        user_id: i64,
        username: String,
    },

    /// Error message.
    Error { message: String },

    /// Timer update (sent every second).
    TimerUpdate { seconds_remaining: u64 },
}

/// Player information.
#[derive(Debug, Serialize, Clone)]
pub struct PlayerInfo {
    pub user_id: i64,
    pub username: String,
    pub display_name: String,
    pub score: i64,
    pub lives: Option<u8>,
    pub is_ready: bool,
    pub is_host: bool,
}

/// Question data sent to clients.
#[derive(Debug, Serialize, Clone)]
pub struct QuestionData {
    pub question: String,
    pub options: Vec<String>,
    pub category: Option<String>,
}

/// Player score in leaderboard.
#[derive(Debug, Serialize, Clone)]
pub struct PlayerScore {
    pub user_id: i64,
    pub username: String,
    pub score: i64,
    pub correct_answers: usize,
    pub total_answers: usize,
}
