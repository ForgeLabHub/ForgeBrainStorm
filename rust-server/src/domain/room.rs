//! Room model and in-memory registry.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::RwLock;
use rand::Rng;
use serde::Serialize;
use tokio::sync::broadcast;

use crate::domain::mode::{GameMode, TEAM_NAMES};
use crate::domain::player::Player;
use crate::error::{AppError, AppResult};

/// Room lifecycle states. Transitions are CAS-protected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RoomState {
    Waiting,
    Starting,
    Playing,
    Finished,
    Error,
}

/// Settings applied to a room (mutable by the host before start).
#[derive(Debug, Clone, Serialize)]
pub struct RoomSettings {
    pub topic: String,
    pub mode: GameMode,
    pub max_players: usize,
    #[serde(default = "default_question_count")]
    pub question_count: usize,
    #[serde(default = "default_num_options")]
    pub num_options: usize,
    #[serde(default = "default_timer", alias = "timer_seconds")]
    pub timer_secs: u64,
    #[serde(default)]
    pub difficulty: String,
}

fn default_question_count() -> usize {
    10
}
fn default_num_options() -> usize {
    4
}
fn default_timer() -> u64 {
    30
}

impl Default for RoomSettings {
    fn default() -> Self {
        Self {
            topic: "Общие знания".into(),
            mode: GameMode::Classic,
            max_players: 30,
            question_count: default_question_count(),
            num_options: default_num_options(),
            timer_secs: default_timer(),
            difficulty: "medium".into(),
        }
    }
}

impl RoomSettings {
    /// Clamp values to safe ranges — applies to sandbox rooms too.
    pub fn clamp(&mut self) {
        self.question_count = self.question_count.clamp(5, 30);
        self.num_options = self.num_options.clamp(2, 6);
        self.timer_secs = self
            .mode
            .timer_override()
            .unwrap_or_else(|| self.timer_secs.clamp(10, 120));
        self.topic.truncate(200);
        if !["easy", "medium", "hard"].contains(&self.difficulty.as_str()) {
            self.difficulty = "medium".into();
        }
    }
}

/// A live room.
#[derive(Debug)]
pub struct Room {
    pub code: String,
    pub host_user_id: i64,
    pub host_name: String,
    pub state: RoomState,
    pub settings: RoomSettings,
    pub players: HashMap<String, Player>, // keyed by sid
    pub questions: Vec<serde_json::Value>,
    pub current_q: usize,
    pub deadline: Option<Instant>,
    pub created_at: Instant,
    pub last_activity: Instant,
    /// Set when the first question goes out; used to record game duration.
    pub started_at: Option<Instant>,
    /// Broadcast channel for fan-out to all players in this room.
    /// Messages are pre-serialised `{type, data}` JSON objects.
    pub tx: broadcast::Sender<String>,
    /// Monotonic token identifying the current question. Timer tasks compare
    /// it before advancing so a stale timer can never skip a question.
    pub question_token: u64,
    /// Where the current question set came from: `ai:<provider>` or `fallback`.
    pub question_source: String,
}

impl Room {
    pub fn new(code: String, host_user_id: i64, host_name: String, settings: RoomSettings) -> Self {
        let (tx, _) = broadcast::channel(256);
        Self {
            code,
            host_user_id,
            host_name,
            state: RoomState::Waiting,
            settings,
            players: HashMap::new(),
            questions: Vec::new(),
            current_q: 0,
            deadline: None,
            created_at: Instant::now(),
            last_activity: Instant::now(),
            started_at: None,
            tx,
            question_token: 0,
            question_source: String::new(),
        }
    }

    pub fn touch(&mut self) {
        self.last_activity = Instant::now();
    }

    /// Share of the clock still left (1.0 right after the question appears,
    /// 0.0 at the buzzer). Modes with speed scoring turn this into points.
    pub fn speed_ratio(&self) -> f64 {
        let Some(deadline) = self.deadline else {
            return 1.0;
        };
        let total = self.settings.timer_secs.max(1) as f64;
        let left = deadline
            .saturating_duration_since(Instant::now())
            .as_secs_f64();
        (left / total).clamp(0.0, 1.0)
    }

    /// CAS transition: Waiting → Starting. Returns false if already taken.
    pub fn try_start(&mut self) -> bool {
        if self.state == RoomState::Waiting {
            self.state = RoomState::Starting;
            true
        } else {
            false
        }
    }

    /// Take the room into a new game. A waiting room starts as it is; a finished
    /// one is wiped first, so the host can run another round without the players
    /// rejoining. Anything in between is refused — a mid-game call would let the
    /// host erase the scores being played for.
    pub fn begin(&mut self) -> bool {
        match self.state {
            RoomState::Waiting => {}
            RoomState::Finished => self.reset_for_rematch(),
            _ => return false,
        }
        self.state = RoomState::Starting;
        true
    }

    /// Clear the previous game and put the room back in `Waiting`.
    pub fn reset_for_rematch(&mut self) {
        self.state = RoomState::Waiting;
        self.questions.clear();
        self.current_q = 0;
        self.started_at = None;
        self.deadline = None;
        // Bump the token so any in-flight timer becomes a no-op.
        self.question_token += 1;
        self.players.retain(|_, player| !player.is_disconnected());
        for player in self.players.values_mut() {
            player.reset_for_new_game();
        }
        self.touch();
    }

    /// Seat players into teams by join order: alternating, so the sides stay
    /// even to within one player. Other modes clear the field, which keeps a
    /// rematch from carrying team badges into a free-for-all.
    pub fn assign_teams(&mut self) {
        if !self.settings.mode.is_team() {
            for player in self.players.values_mut() {
                player.team = None;
            }
            return;
        }
        let mut seats: Vec<(Instant, String)> = self
            .players
            .values()
            .map(|player| (player.joined_at, player.sid.clone()))
            .collect();
        seats.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
        for (index, (_, sid)) in seats.iter().enumerate() {
            if let Some(player) = self.players.get_mut(sid) {
                player.team = Some((index % TEAM_NAMES.len()) as u8);
            }
        }
    }

    pub fn all_answered(&self) -> bool {
        self.players
            .values()
            .filter(|p| !p.is_spectator && !p.is_disconnected() && !p.instant_answer)
            .all(|p| p.answered)
    }

    pub fn reset_answers(&mut self) {
        for p in self.players.values_mut() {
            p.answered = false;
            p.answer_index = None;
            p.answer_time = None;
        }
    }

    pub fn advance_question(&mut self) -> bool {
        self.current_q += 1;
        self.current_q < self.questions.len()
    }

    /// Install a freshly generated question set and start playing.
    pub fn set_questions(&mut self, questions: Vec<serde_json::Value>, source: &str) {
        self.questions = questions;
        self.question_source = source.to_string();
        self.current_q = 0;
        self.question_token = 1;
        self.state = RoomState::Playing;
        self.started_at = Some(Instant::now());
        self.deadline = Some(Instant::now() + Duration::from_secs(self.settings.timer_secs));
        self.reset_answers();
        self.touch();
    }
}

/// In-memory room registry.
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
            return Err(AppError::Conflict("Достигнут лимит комнат".into()));
        }
        let code = gen_code();
        if rooms.contains_key(&code) {
            // Extremely unlikely (26^6), but retry once.
            let code = gen_code();
            if rooms.contains_key(&code) {
                return Err(AppError::Internal(anyhow::anyhow!("code collision")));
            }
        }
        let room = Room::new(code.clone(), host_user_id, host_name.to_string(), settings);
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

    /// Rooms worth showing in the lobby: an empty room whose players all
    /// dropped is dead weight, so only rooms with a live socket are listed.
    pub fn list_public(&self) -> Vec<RoomSnapshot> {
        self.rooms
            .read()
            .values()
            .filter_map(|r| {
                let snapshot = RoomSnapshot::from_room(&r.read());
                (snapshot.player_count > 0).then_some(snapshot)
            })
            .collect()
    }

    pub fn remove(&self, code: &str) {
        self.rooms.write().remove(code);
    }

    /// Remove finished and idle rooms. Returns count removed.
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

/// Serializable room snapshot for HTTP responses.
#[derive(Debug, Serialize)]
pub struct RoomSnapshot {
    pub code: String,
    #[serde(rename = "host")]
    pub host_name: String,
    pub state: RoomState,
    pub mode: String,
    pub mode_title: String,
    pub topic: String,
    pub player_count: usize,
}

impl RoomSnapshot {
    fn from_room(room: &Room) -> Self {
        Self {
            code: room.code.clone(),
            host_name: room.host_name.clone(),
            state: room.state,
            mode: room.settings.mode.as_str().to_string(),
            mode_title: room.settings.mode.title().to_string(),
            topic: room.settings.topic.clone(),
            player_count: room
                .players
                .values()
                .filter(|p| !p.is_disconnected())
                .count(),
        }
    }
}

/// Generate a random 6-character uppercase room code (letters only).
///
/// Uses `secrets::gen_code` semantics — we deliberately don't use
/// `random::choices` because a 6-char alphabetic code (26^6 ≈ 3e8) must
/// be collision-resistant. The registry handles the atomic uniqueness
/// check via `rooms.contains_key(&code)` with a single retry.
fn gen_code() -> String {
    let mut rng = rand::thread_rng();
    (0..6)
        .map(|_| {
            // Generate a random letter A-Z directly, no filter needed.
            let idx = rng.gen_range(0..26);
            (b'A' + idx) as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::player::Player;

    fn room_with_player(score: i64) -> Room {
        let settings = RoomSettings {
            topic: "Космос".into(),
            ..RoomSettings::default()
        };
        let mut room = Room::new("TEST01".into(), 1, "host".into(), settings);
        let mut player = Player::new("sid".into(), "host".into(), Some(1), Some("host".into()));
        player.score = score;
        room.players.insert("sid".into(), player);
        room.questions = vec![serde_json::json!({"question": "q"})];
        room
    }

    #[test]
    fn a_finished_room_can_play_again_from_zero() {
        let mut room = room_with_player(900);
        room.state = RoomState::Finished;

        assert!(room.begin());
        assert_eq!(room.state, RoomState::Starting);
        assert_eq!(room.players["sid"].score, 0);
        assert!(room.questions.is_empty());
        assert_eq!(room.current_q, 0);
        assert!(room.started_at.is_none());
    }

    #[test]
    fn a_running_game_cannot_be_restarted_under_the_players() {
        let mut room = room_with_player(300);
        room.state = RoomState::Playing;

        assert!(!room.begin());
        assert_eq!(room.state, RoomState::Playing);
        assert_eq!(room.players["sid"].score, 300);
    }

    #[test]
    fn a_second_start_on_a_starting_room_is_refused() {
        let mut room = room_with_player(0);
        room.state = RoomState::Starting;

        assert!(!room.begin());
    }
}
