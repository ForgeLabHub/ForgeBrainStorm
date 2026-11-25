//! Room manager for WebSocket gameplay.
//!
//! Manages room lifecycle, player connections, game state, and broadcasts.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::RwLock;
use rand::seq::SliceRandom;
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

use crate::domain::room::{Room, RoomSettings, RoomState};
use crate::domain::player::Player;
use crate::error::AppError;
use crate::services::ai::{AiCapability, GenerationRequest};
use crate::AppState;

use super::protocol::{PlayerAnswer, PlayerInfo, PlayerScore, QuestionData, ServerMessage};
use super::game_loop::GameLoop;

/// Manages all active game rooms.
pub struct RoomManager {
    state: AppState,
    rooms: Arc<RwLock<HashMap<String, Arc<RwLock<Room>>>>>,
}

impl RoomManager {
    pub fn new(state: AppState) -> Self {
        Self {
            state,
            rooms: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new room. Returns the Arc<RwLock<Room>> so the caller can
    /// subscribe to the broadcast channel.
    pub async fn create_room(
        &self,
        user_id: i64,
        username: &str,
        topic: Option<String>,
        mode: Option<String>,
        settings_input: Option<super::protocol::RoomSettingsInput>,
    ) -> Result<Arc<RwLock<Room>>, AppError> {
        let mut settings = RoomSettings {
            topic: topic.unwrap_or_else(|| "Общие знания".into()),
            mode: mode.unwrap_or_else(|| "classic".into()),
            max_players: self.state.settings.max_players,
            ..Default::default()
        };

        // Apply client settings
        if let Some(input) = settings_input {
            if let Some(count) = input.question_count {
                settings.question_count = count.clamp(5, 30);
            }
            if let Some(opts) = input.num_options {
                settings.num_options = opts.clamp(2, 6);
            }
            if let Some(timer) = input.timer_seconds {
                settings.timer_secs = timer.clamp(10, 120);
            }
            if let Some(diff) = input.difficulty {
                settings.difficulty = diff;
            }
        }

        settings.clamp();

        let code = crate::domain::room::generate_room_code();
        let (tx, _) = broadcast::channel(256);

        let room = Room {
            code: code.clone(),
            host_user_id: user_id,
            host_name: username.to_string(),
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

        let room_arc = Arc::new(RwLock::new(room));
        self.rooms.write().insert(code.clone(), room_arc.clone());

        info!(code, user_id, username, "Room created");

        Ok(room_arc)
    }

    /// Join an existing room. Returns the Arc<RwLock<Room>> so the caller
    /// can subscribe to the broadcast channel.
    pub async fn join_room(
        &self,
        code: &str,
        user_id: i64,
        username: &str,
        display_name: &str,
    ) -> Result<Arc<RwLock<Room>>, AppError> {
        let rooms = self.rooms.read();
        let room_arc = rooms
            .get(code)
            .ok_or(AppError::NotFound("Room not found"))?
            .clone();
        drop(rooms);

        let mut room = room_arc.write();

        // Check if room is full
        if room.players.len() >= room.settings.max_players {
            return Err(AppError::Conflict("Room is full".into()));
        }

        // Check if game already started
        if room.state != RoomState::Waiting {
            return Err(AppError::Conflict("Game already started".into()));
        }

        // Check if already in room
        if room.players.contains_key(&user_id) {
            return Err(AppError::Conflict("Already in this room".into()));
        }

        let is_host = room.host_user_id == user_id;

        // Add player
        let player = Player::new(
            user_id,
            username.to_string(),
            display_name.to_string(),
            if room.settings.mode == "lives" { Some(3) } else { None },
        );

        room.players.insert(user_id, player);
        room.last_activity = Instant::now();

        // Broadcast player joined (AFTER inserting)
        let player_info = PlayerInfo {
            user_id,
            username: username.to_string(),
            display_name: display_name.to_string(),
            score: 0,
            lives: if room.settings.mode == "lives" { Some(3) } else { None },
            is_ready: false,
            is_host,
        };

        let _ = room.tx.send(ServerMessage::PlayerJoined { player: player_info });

        // Also send full player list update
        let all_players: Vec<PlayerInfo> = room
            .players
            .values()
            .map(|p| PlayerInfo {
                user_id: p.user_id,
                username: p.username.clone(),
                display_name: p.display_name.clone(),
                score: p.score,
                lives: p.lives,
                is_ready: p.is_ready,
                is_host: p.user_id == room.host_user_id,
            })
            .collect();
        let _ = room.tx.send(ServerMessage::PlayersUpdated { players: all_players });

        info!(code, user_id, username, "Player joined room");

        Ok(room_arc)
    }

    /// Leave a room.
    pub async fn leave_room(&self, code: &str, user_id: i64) {
        let rooms = self.rooms.read();
        if let Some(room_arc) = rooms.get(code) {
            let mut room = room_arc.write();

            // Get player info BEFORE removing
            let player_info = room.players.get(&user_id).map(|p| {
                (p.username.clone(), p.display_name.clone())
            });

            // Broadcast player left (BEFORE removing)
            if let Some((username, _)) = player_info {
                let _ = room.tx.send(ServerMessage::PlayerLeft {
                    user_id,
                    username,
                });
            }

            // Now remove the player
            room.players.remove(&user_id);

            // Send updated player list
            if !room.players.is_empty() {
                let all_players: Vec<PlayerInfo> = room
                    .players
                    .values()
                    .map(|p| PlayerInfo {
                        user_id: p.user_id,
                        username: p.username.clone(),
                        display_name: p.display_name.clone(),
                        score: p.score,
                        lives: p.lives,
                        is_ready: p.is_ready,
                        is_host: p.user_id == room.host_user_id,
                    })
                    .collect();
                let _ = room.tx.send(ServerMessage::PlayersUpdated { players: all_players });
            }

            // If room is empty, remove it
            if room.players.is_empty() {
                let code = room.code.clone();
                drop(room);
                drop(rooms);
                self.rooms.write().remove(&code);
                info!(code, "Room removed (empty)");
            } else {
                // Transfer host if needed
                if room.host_user_id == user_id {
                    if let Some(new_host_id) = room.players.keys().next().copied() {
                        room.host_user_id = new_host_id;
                        if let Some(new_host) = room.players.get(&new_host_id) {
                            room.host_name = new_host.username.clone();
                        }
                    }
                }
            }

            info!(code, user_id, "Player left room");
        }
    }

    /// Start the game (host only).
    pub async fn start_game(&self, code: &str, user_id: i64) -> Result<(), AppError> {
        let rooms = self.rooms.read();
        let room_arc = rooms
            .get(code)
            .ok_or(AppError::NotFound("Room not found"))?
            .clone();
        drop(rooms);

        let mut room = room_arc.write();

        // Check if user is host
        if room.host_user_id != user_id {
            return Err(AppError::Forbidden("Only host can start the game"));
        }

        // Check if game already started
        if room.state != RoomState::Waiting {
            return Err(AppError::Conflict("Game already started".into()));
        }

        // Check if enough players
        if room.players.is_empty() {
            return Err(AppError::Conflict("Need at least 1 player".into()));
        }

        // Generate questions using AI
        info!(code, "Generating questions...");
        let request = GenerationRequest {
            topic: room.settings.topic.clone(),
            count: room.settings.question_count,
            difficulty: room.settings.difficulty.clone(),
            num_options: room.settings.num_options,
            capabilities: vec![AiCapability::QuestionGeneration],
            exclude_hashes: Vec::new(),
        };

        match self.state.ai_router.generate_questions(&request).await {
            Ok(response) => {
                room.questions = response
                    .questions
                    .into_iter()
                    .map(|q| serde_json::to_value(q).unwrap_or_default())
                    .collect();
                room.state = RoomState::Playing;
                room.current_question = Some(0);
                room.question_start_time = Some(Instant::now());

                // Send first question to all players
                if let Some(question) = room.questions.first() {
                    let question_data = QuestionData {
                        question: question["question"].as_str().unwrap_or("").to_string(),
                        options: question["options"]
                            .as_array()
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                    .collect()
                            })
                            .unwrap_or_default(),
                        category: question["category"].as_str().map(|s| s.to_string()),
                    };

                    let _ = room.tx.send(ServerMessage::GameStarted {});
                    let _ = room.tx.send(ServerMessage::QuestionEmitted {
                        question_index: 0,
                        question: question_data,
                        time_limit: room.settings.timer_secs,
                    });
                } else {
                    let _ = room.tx.send(ServerMessage::GameStarted {});
                }

                info!(code, questions = room.questions.len(), "Game started");

                // Spawn game loop
                let room_clone = room_arc.clone();
                let ai_router = self.state.ai_router.clone();
                tokio::spawn(async move {
                    GameLoop::run(room_clone, ai_router).await;
                });
            }
            Err(e) => {
                error!(code, error = %e, "Failed to generate questions");
                return Err(AppError::AiUnavailable(format!(
                    "Failed to generate questions: {}",
                    e
                )));
            }
        }

        Ok(())
    }

    /// Submit an answer.
    pub async fn submit_answer(
        &self,
        code: &str,
        user_id: i64,
        question_index: usize,
        answer_index: usize,
    ) {
        let rooms = self.rooms.read();
        let room_arc = match rooms.get(code) {
            Some(r) => r.clone(),
            None => return,
        };
        drop(rooms);

        let mut room = room_arc.write();

        if room.state != RoomState::Playing {
            return;
        }

        let current_q = match room.current_question {
            Some(q) => q,
            None => return,
        };

        if current_q != question_index {
            return; // Wrong question index
        }

        let player = match room.players.get_mut(&user_id) {
            Some(p) => p,
            None => return,
        };

        // Check if already answered this question
        if player.answers.iter().any(|a| a.question_index == question_index) {
            return;
        }

        // Get correct answer from question JSON
        let question = &room.questions[current_q];
        let correct_index = question["correct_index"].as_u64()
            .or_else(|| question["correct"].as_u64())
            .unwrap_or(0) as usize;
        let is_correct = answer_index == correct_index;

        // Calculate points
        let base_points: i64 = 100;
        let time_bonus = if let Some(start) = room.question_start_time {
            let elapsed = start.elapsed().as_secs();
            let time_limit = room.settings.timer_secs;
            if elapsed < time_limit && time_limit > 0 {
                ((time_limit - elapsed) as i64 * 50) / time_limit as i64
            } else {
                0
            }
        } else {
            0
        };

        let points = if is_correct {
            player.current_streak += 1;
            let streak_bonus = (player.current_streak as i64 - 1) * 25;
            base_points + time_bonus + streak_bonus.max(0)
        } else {
            player.current_streak = 0;
            0
        };

        player.score += points;

        // Handle lives mode
        if room.settings.mode == "lives" && !is_correct {
            if let Some(ref mut lives) = player.lives {
                *lives = lives.saturating_sub(1);
            }
        }

        player.answers.push(PlayerAnswer {
            user_id,
            username: player.username.clone(),
            question_index,
            answer_index,
            is_correct,
            time_ms: room
                .question_start_time
                .map(|s| s.elapsed().as_millis() as u64)
                .unwrap_or(0),
            points_earned: points,
        });

        player.last_activity = Instant::now();

        // Check if all players answered
        let all_answered = room.players.values().all(|p| {
            p.answers.iter().any(|a| a.question_index == current_q)
        });

        if all_answered {
            // Broadcast question result
            let leaderboard = Self::build_leaderboard(&room);
            let question = &room.questions[current_q];
            let correct_index = question["correct_index"].as_u64()
                .or_else(|| question["correct"].as_u64())
                .unwrap_or(0) as usize;
            let explanation = question["explanation"].as_str().unwrap_or("").to_string();

            let player_answers: Vec<_> = room
                .players
                .values()
                .flat_map(|p| p.answers.iter())
                .filter(|a| a.question_index == current_q)
                .cloned()
                .collect();

            let _ = room.tx.send(ServerMessage::QuestionResult {
                question_index: current_q,
                correct_index,
                explanation,
                player_answers,
                leaderboard,
            });

            // Move to next question after delay
            let room_clone = room_arc.clone();
            drop(room);
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_secs(3)).await;
                GameLoop::advance_question(room_clone).await;
            });
        }
    }

    /// Send a chat message.
    pub async fn send_chat(&self, code: &str, user_id: i64, username: &str, text: &str) {
        let rooms = self.rooms.read();
        if let Some(room_arc) = rooms.get(code) {
            let room = room_arc.read();
            let _ = room.tx.send(ServerMessage::ChatMessage {
                user_id,
                username: username.to_string(),
                text: text.to_string(),
                timestamp: chrono::Utc::now().timestamp(),
            });
        }
    }

    /// Use a joker. Returns the indices of removed wrong answers.
    pub async fn use_joker(
        &self,
        code: &str,
        user_id: i64,
    ) -> Result<Vec<usize>, AppError> {
        let rooms = self.rooms.read();
        let room_arc = rooms
            .get(code)
            .ok_or(AppError::NotFound("Room not found"))?
            .clone();
        drop(rooms);

        let mut room = room_arc.write();

        if room.state != RoomState::Playing {
            return Err(AppError::InvalidState);
        }

        let player = room
            .players
            .get_mut(&user_id)
            .ok_or(AppError::NotFound("Player not found"))?;

        if player.jokers_used >= 1 {
            return Err(AppError::Conflict("No jokers remaining".into()));
        }

        let current_q = room.current_question.ok_or(AppError::InvalidState)?;

        let question = &room.questions[current_q];
        let correct_index = question["correct_index"].as_u64()
            .or_else(|| question["correct"].as_u64())
            .unwrap_or(0) as usize;
        let num_options = question["options"]
            .as_array()
            .map(|a| a.len())
            .unwrap_or(4);

        // Remove 2 wrong answers
        let mut wrong_indices: Vec<usize> = (0..num_options).filter(|&i| i != correct_index).collect();
        wrong_indices.shuffle(&mut rand::thread_rng());

        let removed: Vec<usize> = wrong_indices.into_iter().take(2).collect();

        player.jokers_used += 1;

        Ok(removed)
    }

    /// Get a hint.
    pub async fn get_hint(&self, code: &str, user_id: i64) -> Result<String, AppError> {
        let rooms = self.rooms.read();
        let room_arc = rooms
            .get(code)
            .ok_or(AppError::NotFound("Room not found"))?
            .clone();
        drop(rooms);

        let room = room_arc.write();

        if room.state != RoomState::Playing {
            return Err(AppError::InvalidState);
        }

        let current_q = room.current_question.ok_or(AppError::InvalidState)?;

        let question = &room.questions[current_q];
        let question_text = question["question"].as_str().unwrap_or("").to_string();

        // Generate hint using AI
        match self.state.ai_router.generate_hint(&question_text).await {
            Ok(hint) => Ok(hint),
            Err(e) => Err(AppError::AiUnavailable(format!("Failed to generate hint: {}", e))),
        }
    }

    /// Build leaderboard from room state.
    fn build_leaderboard(room: &Room) -> Vec<PlayerScore> {
        let mut scores: Vec<PlayerScore> = room
            .players
            .values()
            .map(|p| {
                let correct = p.answers.iter().filter(|a| a.is_correct).count();
                let total = p.answers.len();
                PlayerScore {
                    user_id: p.user_id,
                    username: p.username.clone(),
                    score: p.score,
                    correct_answers: correct,
                    total_answers: total,
                }
            })
            .collect();

        scores.sort_by(|a, b| b.score.cmp(&a.score));
        scores
    }

    /// Get room Arc by code.
    pub fn get_room(&self, code: &str) -> Option<Arc<RwLock<Room>>> {
        self.rooms.read().get(code).cloned()
    }

    /// List all rooms (basic info).
    pub fn list_rooms(&self) -> Vec<crate::domain::room::RoomSnapshot> {
        self.rooms
            .read()
            .values()
            .map(|r| {
                let room = r.read();
                crate::domain::room::RoomSnapshot {
                    code: room.code.clone(),
                    host_name: room.host_name.clone(),
                    state: room.state,
                    mode: room.settings.mode.clone(),
                    topic: room.settings.topic.clone(),
                    player_count: room.players.len(),
                }
            })
            .collect()
    }
}
