//! Room model and in-memory registry.
//!
//! Two room types:
//! - `Room`: WebSocket gameplay version (keyed by user_id)
//! - `LegacyRoom`: HTTP API version (keyed by session ID)

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::RwLock;
use rand::Rng;
use serde::Serialize;
use tokio::sync::broadcast;

use crate::domain::player::{LegacyPlayer, Player};
use crate::error::{AppError, AppResult};
use crate::ws::protocol::ServerMessage;

/// Room lifecycle states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RoomState {
    Waiting,
    Starting,
    Playing,
    Finished,
    Error,
}

/// Settings applied to a room.
#[derive(Debug, Clone, Serialize)]
pub struct RoomSettings {
    pub topic: String,
    pub mode: String,
    pub max_players: usize,
    #[serde(default = "default_question_count")]
    pub question_count: usize,
    #[serde(default = "default_num_options")]
    pub num_options: usize,
    #[serde(default = "default_timer")]
    pub timer_secs: u64,
    #[serde(default)]
    pub difficulty: String,
}

fn default_question_count() -> usize { 10 }
fn default_num_options() -> usize { 4 }
fn default_timer() -> u64 { 30 }

impl Default for RoomSettings {
    fn default() -> Self {
        Self {
            topic: "Общие знания".into(),
            mode: "classic".into(),
            max_players: 30,
            question_count: default_question_count(),
            num_options: default_num_options(),
            timer_secs: default_timer(),
            difficulty: "medium".into(),
        }
    }
}

impl RoomSettings {
    /// Clamp values to safe ranges.
    pub fn clamp(&mut self) {
        self.question_count = self.question_count.clamp(5, 30);
        self.num_options = self.num_options.clamp(2, 6);
        self.timer_secs = self.timer_secs.clamp(10, 120);
        self.topic.truncate(200);
        if !["easy", "medium", "hard"].contains(&self.difficulty.as_str()) {
            self.difficulty = "medium".into();
        }
        if !["classic", "ffa", "team", "lives", "coop", "svoyaigra", "sandbox"]
            .contains(&self.mode.as_str())
        {
            self.mode = "classic".into();
        }
    }
}

/// WebSocket gameplay room (keyed by user_id).
#[derive(Debug)]
pub struct Room {
    pub code: String,
    pub host_user_id: i64,
    pub host_name: String,
    pub state: RoomState,
    pub settings: RoomSettings,
    pub players: HashMap<i64, Player>,
    pub questions: Vec<serde_json::Value>,
    pub current_question: Option<usize>,
    pub question_start_time: Option<Instant>,
    pub created_at: Instant,
    pub last_activity: Instant,
    pub tx: broadcast::Sender<ServerMessage>,
}

/// Legacy room for HTTP API (keyed by session ID).
#[derive(Debug)]
pub struct LegacyRoom {
    pub code: String,
    pub host_sid: String,
    pub state: RoomState,
    pub settings: RoomSettings,
    pub players: HashMap<String, LegacyPlayer>,
    pub questions: Vec<serde_json::Value>,
    pub current_q: usize,
    pub deadline: Option<Instant>,
    pub created_at: Instant,
    pub last_activity: Instant,
    pub tx: broadcast::Sender<LegacyWsEvent>,
}

/// Legacy WebSocket events.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum LegacyWsEvent {
    PlayersUpdated,
    QuestionEmitted { index: usize },
    QuestionResult { correct_index: usize },
    ChatMessage { name: String, text: String, ts: i64 },
    GameOver { scores: HashMap<String, i64> },
    Custom(String, serde_json::Value),
}

/// In-memory room registry (WebSocket version).
#[derive(Clone)]
pub struct RoomRegistry {
    rooms: Arc<RwLock<HashMap<String, Arc<RwLock<Room>>>>>,
    max_rooms: usize,
}

impl RoomRegistry {
    pub fn new(max_rooms: usize) -> Self {
        Self {
            rooms: Arc::new(RwLock::new(HashMap::new())),
            max_rooms,
        }
    }

    pub fn create(
        &self,
        host_user_id: i64,
        host_name: &str,
        mut settings: RoomSettings,
    ) -> AppResult<RoomSnapshot> {
        settings.clamp();
        let mut rooms = self.rooms.write();

        if rooms.len() >= self.max_rooms {
            return Err(AppError::Conflict("Room limit reached".into()));
        }

        let code = generate_room_code();
        let (tx, _) = broadcast::channel(256);

        let room = Room {
            code: code.clone(),
            host_user_id,
            host_name: host_name.to_string(),
            state: RoomState::Waiting,
            settings,
            players: HashMap::new(),
            questions: Vec::new(),
            current_question: None,
            question_start_time: None,
            created_at: Instant::now(),
            last_activity: Instant::now(),
            tx,
        };

        let snapshot = RoomSnapshot::from_room(&room);
        rooms.insert(code, Arc::new(RwLock::new(room)));

        Ok(snapshot)
    }

    pub fn get(&self, code: &str) -> Option<RoomSnapshot> {
        let rooms = self.rooms.read();
        rooms.get(code).map(|r| RoomSnapshot::from_room(&r.read()))
    }

    pub fn get_arc(&self, code: &str) -> Option<Arc<RwLock<Room>>> {
        self.rooms.read().get(code).cloned()
    }

    pub fn list_public(&self) -> Vec<RoomSnapshot> {
        self.rooms
            .read()
            .values()
            .map(|r| RoomSnapshot::from_room(&r.read()))
            .collect()
    }

    pub fn remove(&self, code: &str) {
        self.rooms.write().remove(code);
    }

    pub fn cleanup_stale(&self, idle_timeout: Duration) -> usize {
        let mut rooms = self.rooms.write();
        let now = Instant::now();
        let before = rooms.len();

        rooms.retain(|_, room| {
            let r = room.read();
            let idle = now.duration_since(r.last_activity) > idle_timeout;
            let finished = r.state == RoomState::Finished;
            !(idle && finished)
        });

        before - rooms.len()
    }
}

/// Serializable room snapshot.
#[derive(Debug, Serialize)]
pub struct RoomSnapshot {
    pub code: String,
    #[serde(rename = "host")]
    pub host_name: String,
    pub state: RoomState,
    pub mode: String,
    pub topic: String,
    pub player_count: usize,
}

impl RoomSnapshot {
    fn from_room(room: &Room) -> Self {
        Self {
            code: room.code.clone(),
            host_name: room.host_name.clone(),
            state: room.state,
            mode: room.settings.mode.clone(),
            topic: room.settings.topic.clone(),
            player_count: room.players.len(),
        }
    }
}

/// Generate a random 6-character room code.
pub fn generate_room_code() -> String {
    let mut rng = rand::thread_rng();
    (0..6)
        .map(|_| {
            let idx = rng.gen_range(0..26);
            (b'A' + idx) as char
        })
        .collect()
}

/// Legacy room registry (for backward compatibility).
#[derive(Clone)]
pub struct LegacyRoomRegistry {
    rooms: Arc<RwLock<HashMap<String, Arc<RwLock<LegacyRoom>>>>>,
    max_rooms: usize,
}

impl LegacyRoomRegistry {
    pub fn new(max_rooms: usize) -> Self {
        Self {
            rooms: Arc::new(RwLock::new(HashMap::new())),
            max_rooms,
        }
    }
}
