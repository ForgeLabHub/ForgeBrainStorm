//! WebSocket message types for client <-> server communication.
//!
//! All messages follow the format: `{ "type": "...", "data": {...} }`
//!
//! # Client Messages
//!
//! Messages sent from client to server:
//! - `JoinRoom` - join a room by code
//! - `StartGame` - start the game (host only)
//! - `SubmitAnswer` - submit an answer to the current question
//! - `UsePowerUp` - use a power-up
//! - `UseCheatPower` - use a cheat power (cheater role only)
//!
//! # Server Messages
//!
//! Messages sent from server to client:
//! - `RoomJoined` - confirmation of joining a room
//! - `PlayersUpdated` - player list changed
//! - `QuestionEmitted` - new question available
//! - `AnswerResult` - result of submitted answer
//! - `PowerUps` - player's power-up hand
//! - `CheatState` - cheat power budget
//! - `CheatReveal` - cheat power revealed correct answer
//! - `OptionsHidden` - options hidden by power-up
//! - `Error` - error message

use serde::{Deserialize, Serialize};

use crate::ids::{QuestionId, RoomCode, UserId};

/// Message envelope for all WebSocket messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEnvelope<T> {
    /// Message type identifier.
    pub r#type: String,
    /// Message payload.
    pub data: T,
}

/// Messages sent from client to server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ClientMessage {
    /// Join a room by code.
    #[serde(rename = "join_room")]
    JoinRoom {
        /// Room code to join.
        code: RoomCode,
    },
    /// Start the game (host only).
    #[serde(rename = "start_game")]
    StartGame,
    /// Submit an answer to the current question.
    #[serde(rename = "submit_answer")]
    SubmitAnswer {
        /// Question ID being answered.
        question_id: QuestionId,
        /// Selected option index (0-based).
        option_index: u8,
    },
    /// Use a power-up.
    #[serde(rename = "use_power_up")]
    UsePowerUp {
        /// Power-up type to use.
        power_up: String,
    },
    /// Use a cheat power (cheater role only).
    #[serde(rename = "use_cheat_power")]
    UseCheatPower {
        /// Cheat power type to use.
        power: String,
    },
}

/// Messages sent from server to client.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ServerMessage {
    /// Confirmation of joining a room.
    #[serde(rename = "room_joined")]
    RoomJoined {
        /// Room code.
        code: RoomCode,
        /// Whether the user is the host.
        is_host: bool,
        /// Room topic.
        topic: String,
        /// Game mode.
        mode: String,
        /// Game mode title (human-readable).
        mode_title: String,
    },
    /// Player list updated.
    #[serde(rename = "players_updated")]
    PlayersUpdated {
        /// List of players in the room.
        players: Vec<PlayerInfo>,
    },
    /// A player joined the room.
    #[serde(rename = "player_joined")]
    PlayerJoined {
        /// Player who joined.
        player: PlayerInfo,
    },
    /// A player left the room.
    #[serde(rename = "player_left")]
    PlayerLeft {
        /// Player who left.
        player: PlayerInfo,
    },
    /// New question available.
    #[serde(rename = "question_emitted")]
    QuestionEmitted {
        /// Question data (without correct answer).
        question: QuestionData,
        /// Time limit in seconds.
        time_limit: u64,
        /// Time remaining in seconds (for reconnect).
        time_remaining: u64,
        /// Question index (0-based).
        index: usize,
        /// Total number of questions.
        total: usize,
        /// Question source ("ai" or "offline").
        source: String,
        /// Game mode.
        mode: String,
    },
    /// Result of submitted answer.
    #[serde(rename = "answer_result")]
    AnswerResult {
        /// Question ID that was answered.
        question_id: QuestionId,
        /// Whether the answer was accepted.
        accepted: bool,
        /// Points earned (can be negative for penalties).
        points: i64,
        /// Current streak.
        streak: u32,
        /// Whether the answer was correct.
        correct: bool,
    },
    /// Player's power-up hand.
    #[serde(rename = "power_ups")]
    PowerUps {
        /// List of power-up types.
        items: Vec<String>,
    },
    /// Cheat power budget (cheater role only).
    #[serde(rename = "cheat_state")]
    CheatState {
        /// Remaining cheat powers.
        powers: serde_json::Value,
    },
    /// Cheat power revealed correct answer.
    #[serde(rename = "cheat_reveal")]
    CheatReveal {
        /// Index of the correct option.
        correct_index: u8,
    },
    /// Options hidden by power-up.
    #[serde(rename = "options_hidden")]
    OptionsHidden {
        /// Indices of hidden options.
        indexes: Vec<u8>,
    },
    /// Error message.
    #[serde(rename = "error")]
    Error {
        /// Error message.
        message: String,
        /// Error code (optional).
        #[serde(skip_serializing_if = "Option::is_none")]
        code: Option<String>,
    },
    /// Game over.
    #[serde(rename = "game_over")]
    GameOver {
        /// Final scores.
        scores: Vec<ScoreEntry>,
        /// Whether the game was persisted to database.
        persisted: bool,
        /// Game mode.
        mode: String,
        /// Team scores (if team mode).
        #[serde(skip_serializing_if = "Option::is_none")]
        teams: Option<Vec<TeamScore>>,
    },
}

/// Player information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInfo {
    /// Player's session ID.
    pub sid: String,
    /// Player's display name.
    pub name: String,
    /// Player's user ID (if authenticated).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<UserId>,
    /// Current score.
    pub score: i64,
    /// Current streak.
    pub streak: u32,
    /// Remaining lives (if lives mode).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lives: Option<u8>,
    /// Team index (if team mode).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team: Option<u8>,
    /// Whether the player is the host.
    pub is_host: bool,
    /// Whether the player is a spectator.
    #[serde(default)]
    pub is_spectator: bool,
}

/// Question data (without correct answer).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionData {
    /// Question text.
    pub question: String,
    /// Available options.
    pub options: Vec<String>,
}

/// Score entry for game over.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreEntry {
    /// Player's user ID.
    pub user_id: UserId,
    /// Player's display name.
    pub username: String,
    /// Final score.
    pub score: i64,
    /// Place (1-based).
    pub place: usize,
}

/// Team score for team mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamScore {
    /// Team index.
    pub team: u8,
    /// Team name.
    pub name: String,
    /// Team total score.
    pub score: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_message_serialization() {
        let msg = ClientMessage::JoinRoom {
            code: RoomCode::new("ABC123"),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("join_room"));
        assert!(json.contains("ABC123"));

        let decoded: ClientMessage = serde_json::from_str(&json).unwrap();
        match decoded {
            ClientMessage::JoinRoom { code } => {
                assert_eq!(code.as_str(), "ABC123");
            }
            _ => panic!("Expected JoinRoom message"),
        }
    }

    #[test]
    fn test_server_message_serialization() {
        let msg = ServerMessage::Error {
            message: "Room not found".to_string(),
            code: Some("room_not_found".to_string()),
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("error"));
        assert!(json.contains("Room not found"));

        let decoded: ServerMessage = serde_json::from_str(&json).unwrap();
        match decoded {
            ServerMessage::Error { message, code } => {
                assert_eq!(message, "Room not found");
                assert_eq!(code, Some("room_not_found".to_string()));
            }
            _ => panic!("Expected Error message"),
        }
    }

    #[test]
    fn test_player_info_serialization() {
        let player = PlayerInfo {
            sid: "session123".to_string(),
            name: "Alice".to_string(),
            user_id: Some(UserId::new(42)),
            score: 100,
            streak: 3,
            lives: Some(3),
            team: None,
            is_host: true,
            is_spectator: false,
        };
        let json = serde_json::to_string(&player).unwrap();
        assert!(json.contains("Alice"));
        assert!(json.contains("42"));

        let decoded: PlayerInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.name, "Alice");
        assert_eq!(decoded.score, 100);
    }

    #[test]
    fn test_question_data_serialization() {
        let question = QuestionData {
            question: "What is 2+2?".to_string(),
            options: vec!["3".to_string(), "4".to_string(), "5".to_string()],
        };
        let json = serde_json::to_string(&question).unwrap();
        assert!(json.contains("What is 2+2?"));
        assert!(json.contains("4"));

        let decoded: QuestionData = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.question, "What is 2+2?");
        assert_eq!(decoded.options.len(), 3);
    }
}
