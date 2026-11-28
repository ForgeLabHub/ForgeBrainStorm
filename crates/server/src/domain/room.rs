//! Room model and in-memory registry.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::RwLock;
use rand::Rng;
use serde::Serialize;
use tokio::sync::broadcast;

use crate::domain::chat::ChatLog;
use crate::domain::cheat::{BLIND_SECS, LEECH_POINTS};
use crate::domain::mode::{GameMode, TEAM_NAMES};
use crate::domain::player::Player;
use crate::domain::proposal::{Proposal, ProposalType};
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
    /// Provider the host picked for question generation; `None` means the
    /// router's own chain decides.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ai_provider: Option<String>,
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
            ai_provider: None,
        }
    }
}

impl RoomSettings {
    /// Clamp values to safe ranges — applies to sandbox rooms too.
    pub fn clamp(&mut self) {
        self.question_count = self
            .mode
            .question_override()
            .unwrap_or_else(|| self.question_count.clamp(5, 30));
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
    /// Where the current question set came from: `ai:<provider>`, `fallback`
    /// or `cheat` once a cheater rewrote the question.
    pub question_source: String,
    /// Chat history and per-player flood control.
    pub chat: ChatLog,
    /// Set by an admin: chat and emoji are refused until it is lifted.
    pub chat_locked: bool,
    /// Set when an admin closes the room. Sockets are told to leave and every
    /// further message is refused, so a closed room cannot keep playing on a
    /// stale `Arc`.
    pub closed: bool,
    /// Settings changes proposed by players while the room is in the lobby.
    pub proposals: Vec<Proposal>,
    proposal_counter: u64,
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
            chat: ChatLog::new(),
            chat_locked: false,
            closed: false,
            proposals: Vec::new(),
            proposal_counter: 0,
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
    /// Record a player's proposal and return its id. The proposer starts as
    /// the only voter, which is what the UI shows as vote count 1.
    pub fn create_proposal(
        &mut self,
        proposer_sid: String,
        proposer_name: String,
        proposal_type: ProposalType,
        value: String,
        description: String,
    ) -> String {
        self.proposal_counter += 1;
        let id = format!("prop_{}_{}", self.code, self.proposal_counter);
        self.proposals.push(Proposal::new(
            id.clone(),
            proposer_sid,
            proposer_name,
            proposal_type,
            value,
            description,
        ));
        // A lobby can run for hours; only the recent proposals stay usable.
        if self.proposals.len() > 50 {
            self.proposals.drain(0..self.proposals.len() - 50);
        }
        id
    }

    pub fn get_proposal(&self, id: &str) -> Option<&Proposal> {
        self.proposals.iter().find(|proposal| proposal.id == id)
    }

    pub fn get_proposal_mut(&mut self, id: &str) -> Option<&mut Proposal> {
        self.proposals.iter_mut().find(|proposal| proposal.id == id)
    }

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
        self.chat.clear();
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
        let coop = self.settings.mode.is_coop();
        for (index, (_, sid)) in seats.iter().enumerate() {
            if let Some(player) = self.players.get_mut(sid) {
                // Coop has a single side, so every player scores into the same pool.
                player.team = Some(if coop {
                    0
                } else {
                    (index % TEAM_NAMES.len()) as u8
                });
            }
        }
    }

    pub fn all_answered(&self) -> bool {
        self.players
            .values()
            .filter(|p| !p.is_spectator && !p.is_disconnected())
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

    /// Resolve a cheat target: a socket id or a display name, case-insensitive.
    /// Spectators and the cheater themselves are not valid victims.
    pub fn resolve_victim(&self, needle: &str, cheater_sid: &str) -> Option<String> {
        let needle = needle.trim();
        if needle.is_empty() {
            return None;
        }
        let lower = needle.to_lowercase();
        self.players
            .values()
            .filter(|player| player.sid != cheater_sid && !player.is_spectator)
            .find(|player| player.sid == needle || player.display_name.to_lowercase() == lower)
            .map(|player| player.sid.clone())
    }

    /// Take over `victim`: their points move to `cheater` and they finish the
    /// game as a spectator. Returns the victim's name and the points moved.
    pub fn hijack(&mut self, cheater_sid: &str, victim_sid: &str) -> Option<(String, i64)> {
        if cheater_sid == victim_sid {
            return None;
        }
        let (name, points) = {
            let victim = self.players.get_mut(victim_sid)?;
            if victim.is_spectator {
                return None;
            }
            let points = victim.score.max(0);
            victim.score = 0;
            victim.is_spectator = true;
            victim.hijacked = true;
            victim.answered = true;
            victim.answer_index = None;
            (victim.display_name.clone(), points)
        };
        let cheater = self.players.get_mut(cheater_sid)?;
        cheater.score += points;
        Some((name, points))
    }

    /// Take [`crate::domain::cheat::LEECH_POINTS`] from every other active
    /// player. Scores never go below zero, so a player sitting at zero cannot
    /// fund the cheater. Returns the number of victims and the points moved;
    /// zero points means the power found nothing to do and the caller should
    /// refund it.
    pub fn leech(&mut self, cheater_sid: &str) -> (usize, i64) {
        let victims: Vec<String> = self
            .players
            .values()
            .filter(|player| player.sid != cheater_sid && !player.is_spectator && player.score > 0)
            .map(|player| player.sid.clone())
            .collect();

        let mut taken = 0;
        for victim in &victims {
            if let Some(player) = self.players.get_mut(victim) {
                let amount = player.score.min(LEECH_POINTS);
                player.score -= amount;
                taken += amount;
            }
        }
        if let Some(cheater) = self.players.get_mut(cheater_sid) {
            cheater.score += taken;
        }
        (victims.len(), taken)
    }

    /// Trade scores with the current leader. Returns the leader's name, the
    /// cheater's old score and the leader's, so the room can be told what
    /// happened. `None` when the cheater is already leading — swapping with
    /// yourself would spend a use for nothing.
    pub fn swap_with_leader(&mut self, cheater_sid: &str) -> Option<(String, i64, i64)> {
        let (leader_sid, leader_score) = self
            .players
            .iter()
            .filter(|(sid, player)| sid.as_str() != cheater_sid && !player.is_spectator)
            .map(|(sid, player)| (sid.clone(), player.score))
            .max_by_key(|(_, score)| *score)?;

        let cheater_score = self.players.get(cheater_sid)?.score;
        if leader_score <= cheater_score {
            return None;
        }

        let leader_name = {
            let leader = self.players.get_mut(&leader_sid)?;
            leader.score = cheater_score;
            leader.display_name.clone()
        };
        let cheater = self.players.get_mut(cheater_sid)?;
        cheater.score = leader_score;

        Some((leader_name, cheater_score, leader_score))
    }

    /// Silence `victim` for the rest of the game. `None` when they are already
    /// muted, so the caller can refund the use.
    pub fn mute(&mut self, victim_sid: &str) -> Option<String> {
        let victim = self.players.get_mut(victim_sid)?;
        if victim.muted {
            return None;
        }
        victim.muted = true;
        Some(victim.display_name.clone())
    }

    /// Curse `victim`: their next answer scores nothing. `None` when a curse is
    /// already pending, because a second one would be wasted.
    pub fn jinx(&mut self, victim_sid: &str) -> Option<String> {
        let victim = self.players.get_mut(victim_sid)?;
        if victim.jinxed {
            return None;
        }
        victim.jinxed = true;
        Some(victim.display_name.clone())
    }

    /// Blind `victim` for [`crate::domain::cheat::BLIND_SECS`]. Returns their
    /// name, or `None` when they are already blind (the caller refunds).
    pub fn blind(&mut self, victim_sid: &str) -> Option<String> {
        let victim = self.players.get_mut(victim_sid)?;
        if victim.is_blind() {
            return None;
        }
        victim.blind_until = Some(Instant::now() + Duration::from_secs(BLIND_SECS));
        Some(victim.display_name.clone())
    }

    /// Replace the current question with one the cheater wrote. Answers already
    /// given are dropped so the whole room answers the new text.
    pub fn rewrite_question(
        &mut self,
        text: String,
        options: Vec<String>,
        correct_index: usize,
    ) -> bool {
        if options.len() < 2 || correct_index >= options.len() {
            return false;
        }
        let Some(slot) = self.questions.get_mut(self.current_q) else {
            return false;
        };
        *slot = serde_json::json!({
            "question": text,
            "options": options,
            "correct_index": correct_index,
            "explanation": "Вопрос подменил читер. Верный ответ знает только он.",
        });
        self.question_source = "cheat".to_string();
        self.reset_answers();
        self.touch();
        true
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
        drop(rooms);
        self.publish_active_rooms();
        Ok(snapshot)
    }

    /// Number of live rooms; feeds the `room_active_total` gauge.
    pub fn len(&self) -> usize {
        self.rooms.read().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
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
        self.publish_active_rooms();
    }

    fn publish_active_rooms(&self) {
        crate::metrics::Metrics::global().set_active_rooms(self.len());
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
        let removed = before - rooms.len();
        drop(rooms);
        self.publish_active_rooms();
        removed
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

    fn room_with_two_players() -> Room {
        let mut room = room_with_player(500);
        let mut victim = Player::new("v1".into(), "Боря".into(), Some(2), Some("borya".into()));
        victim.score = 300;
        room.players.insert("v1".into(), victim);
        room.state = RoomState::Playing;
        room
    }

    #[test]
    fn a_victim_is_found_by_name_case_insensitively() {
        let room = room_with_two_players();
        assert_eq!(room.resolve_victim("боря", "sid"), Some("v1".into()));
        assert_eq!(room.resolve_victim("v1", "sid"), Some("v1".into()));
        // A cheater cannot target themselves, an unknown name, or nothing.
        assert_eq!(room.resolve_victim("host", "sid"), None);
        assert_eq!(room.resolve_victim("никого", "sid"), None);
        assert_eq!(room.resolve_victim("   ", "sid"), None);
    }

    #[test]
    fn hijacking_moves_the_points_and_benches_the_victim() {
        let mut room = room_with_two_players();
        let taken = room.hijack("sid", "v1").expect("hijacked");

        assert_eq!(taken, ("Боря".into(), 300));
        assert_eq!(room.players["sid"].score, 800);
        assert_eq!(room.players["v1"].score, 0);
        assert!(room.players["v1"].is_spectator);
        assert!(room.players["v1"].hijacked);
        // The victim is out of the game, so they no longer count as answering.
        assert_eq!(room.resolve_victim("v1", "sid"), None);
    }

    #[test]
    fn leeching_takes_from_everyone_but_never_below_zero() {
        let mut room = room_with_two_players();
        room.players.get_mut("v1").unwrap().score = 10;

        let (victims, taken) = room.leech("sid");
        assert_eq!(victims, 1);
        assert_eq!(taken, 10);
        assert_eq!(room.players["v1"].score, 0);
        assert_eq!(room.players["sid"].score, 510);

        // Nothing left to take: the caller refunds instead of charging again.
        assert_eq!(room.leech("sid"), (0, 0));
    }

    #[test]
    fn leeching_skips_spectators() {
        let mut room = room_with_two_players();
        room.players.get_mut("v1").unwrap().is_spectator = true;
        assert_eq!(room.leech("sid"), (0, 0));
    }

    #[test]
    fn swapping_scores_only_pays_off_when_behind() {
        let mut room = room_with_two_players();
        // The host has 500, the victim 300, so a swap would lose points.
        assert!(room.swap_with_leader("sid").is_none());

        room.players.get_mut("sid").unwrap().score = 100;
        let (name, old, new) = room.swap_with_leader("sid").expect("swapped");
        assert_eq!(name, "Боря");
        assert_eq!((old, new), (100, 300));
        assert_eq!(room.players["sid"].score, 300);
        assert_eq!(room.players["v1"].score, 100);
    }

    #[test]
    fn swapping_ignores_benched_players() {
        let mut room = room_with_two_players();
        room.players.get_mut("sid").unwrap().score = 100;
        room.players.get_mut("v1").unwrap().is_spectator = true;
        assert!(room.swap_with_leader("sid").is_none());
    }

    #[test]
    fn muting_and_cursing_report_repeats() {
        let mut room = room_with_two_players();
        assert_eq!(room.mute("v1"), Some("Боря".into()));
        assert_eq!(room.mute("v1"), None);
        assert_eq!(room.jinx("v1"), Some("Боря".into()));
        assert_eq!(room.jinx("v1"), None);
        assert_eq!(room.mute("nobody"), None);
    }

    #[test]
    fn blinding_sets_a_deadline_and_does_not_stack() {
        let mut room = room_with_two_players();
        assert_eq!(room.blind("v1"), Some("Боря".into()));
        assert!(room.players["v1"].is_blind());
        assert_eq!(room.blind("v1"), None, "a second blind would be wasted");
        assert_eq!(room.blind("nobody"), None);
    }

    #[test]
    fn an_already_benched_player_cannot_be_hijacked_twice() {
        let mut room = room_with_two_players();
        room.players.get_mut("v1").unwrap().is_spectator = true;
        assert!(room.hijack("sid", "v1").is_none());
    }

    #[test]
    fn rewriting_replaces_the_question_and_drops_the_answers() {
        let mut room = room_with_two_players();
        room.players.get_mut("v1").unwrap().answered = true;

        let ok =
            room.rewrite_question("Сколько будет 2+2?".into(), vec!["3".into(), "4".into()], 1);

        assert!(ok);
        let question = &room.questions[room.current_q];
        assert_eq!(question["question"], "Сколько будет 2+2?");
        assert_eq!(question["correct_index"], 1);
        assert_eq!(room.question_source, "cheat");
        assert!(!room.players["v1"].answered);
    }

    #[test]
    fn a_rewrite_with_a_bad_correct_index_is_refused() {
        let mut room = room_with_two_players();
        assert!(!room.rewrite_question("Вопрос".into(), vec!["a".into()], 0));
        assert!(!room.rewrite_question("Вопрос".into(), vec!["a".into(), "b".into()], 5));
        assert_eq!(room.questions[0]["question"], "q");
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
