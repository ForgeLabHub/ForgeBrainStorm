//! Player model and in-room state.

use std::time::Instant;

use serde::Serialize;

use crate::ws::protocol::PlayerAnswer;

/// A player within a room (WebSocket gameplay version).
#[derive(Debug, Clone)]
pub struct Player {
    pub user_id: i64,
    pub username: String,
    pub display_name: String,
    pub score: i64,
    pub lives: Option<u8>,  // None if not lives mode
    pub jokers_used: u32,
    pub current_streak: u32,
    pub answers: Vec<PlayerAnswer>,
    pub is_ready: bool,
    pub connected_at: Instant,
    pub last_activity: Instant,
}

impl Player {
    pub fn new(
        user_id: i64,
        username: String,
        display_name: String,
        lives: Option<u8>,
    ) -> Self {
        Self {
            user_id,
            username,
            display_name,
            score: 0,
            lives,
            jokers_used: 0,
            current_streak: 0,
            answers: Vec::new(),
            is_ready: false,
            connected_at: Instant::now(),
            last_activity: Instant::now(),
        }
    }
}

/// Legacy player model (for backward compatibility).
#[derive(Debug, Clone)]
pub struct LegacyPlayer {
    pub sid: String,
    pub user_id: Option<i64>,
    pub username: String,
    pub display_name: String,
    pub score: i64,
    pub streak: u32,
    pub total_correct: u32,
    pub answered: bool,
    pub answer_index: Option<usize>,
    pub answer_time: Option<Instant>,
    pub lives: u8,
    pub infinite_lives: bool,
    pub is_spectator: bool,
    pub is_cheat: bool,
    pub is_invisible: bool,
    pub instant_answer: bool,
    pub team: Option<u8>,
    pub joined_at: Instant,
    pub disconnected_at: Option<Instant>,
}

impl LegacyPlayer {
    pub fn new(sid: String, display_name: String, user_id: Option<i64>, username: Option<String>) -> Self {
        Self {
            sid,
            user_id,
            username: username.unwrap_or_default(),
            display_name,
            score: 0,
            streak: 0,
            total_correct: 0,
            answered: false,
            answer_index: None,
            answer_time: None,
            lives: 3,
            infinite_lives: false,
            is_spectator: false,
            is_cheat: false,
            is_invisible: false,
            instant_answer: false,
            team: None,
            joined_at: Instant::now(),
            disconnected_at: None,
        }
    }

    pub fn is_disconnected(&self) -> bool {
        self.disconnected_at.is_some()
    }
}

/// Serializable summary sent to clients.
#[derive(Debug, Serialize)]
pub struct PlayerSummary {
    pub name: String,
    pub score: i64,
    pub streak: u32,
    pub lives: u8,
    pub is_host: bool,
    pub is_ready: bool,
    pub team: Option<u8>,
    pub is_spectator: bool,
}
