//! Game loop logic.
//!
//! Manages question timers, transitions between questions, and game over conditions.

use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::RwLock;
use tokio::time::sleep;
use tracing::{info, warn};

use crate::domain::room::{Room, RoomState};
use crate::services::ai::AiRouter;

use super::protocol::{PlayerAnswer, PlayerScore, QuestionData, ServerMessage};

/// Manages the game loop for a room.
pub struct GameLoop;

impl GameLoop {
    /// Run the game loop for a room.
    ///
    /// This handles:
    /// - Question timers
    /// - Broadcasting timer updates
    /// - Advancing to next question
    /// - Game over detection
    pub async fn run(room: Arc<RwLock<Room>>, _ai_router: Arc<AiRouter>) {
        info!("Game loop started");

        loop {
            // Check if game should continue
            let (should_continue, timer_secs, question_index) = {
                let room = room.read();

                if room.state != RoomState::Playing {
                    info!("Game no longer in Playing state, exiting loop");
                    (false, 0, 0)
                } else if room.current_question.is_none() {
                    info!("No current question, exiting loop");
                    (false, 0, 0)
                } else {
                    (
                        true,
                        room.settings.timer_secs,
                        room.current_question.unwrap_or(0),
                    )
                }
            };

            if !should_continue {
                break;
            }

            // Run timer
            Self::run_timer(room.clone(), timer_secs, question_index).await;

            // Check if game is still playing after timer
            {
                let room = room.read();
                if room.state != RoomState::Playing {
                    break;
                }
            }

            // After timer expires, broadcast question result (if not already done)
            Self::broadcast_question_result(room.clone(), question_index).await;

            // Wait before advancing
            sleep(Duration::from_secs(3)).await;

            // Advance to next question
            Self::advance_question(room.clone()).await;
        }

        info!("Game loop ended");
    }

    /// Run the question timer.
    async fn run_timer(room: Arc<RwLock<Room>>, duration_secs: u64, question_index: usize) {
        let start = Instant::now();

        // Update room start time
        {
            let mut room = room.write();
            room.question_start_time = Some(start);
        }

        // Broadcast timer updates every second
        for remaining in (0..duration_secs).rev() {
            sleep(Duration::from_secs(1)).await;

            let room = room.read();

            // Check if game still playing and on same question
            if room.state != RoomState::Playing || room.current_question != Some(question_index) {
                break;
            }

            let _ = room.tx.send(ServerMessage::TimerUpdate {
                seconds_remaining: remaining,
            });
        }
    }

    /// Broadcast question result after timeout.
    async fn broadcast_question_result(room: Arc<RwLock<Room>>, question_index: usize) {
        let room_guard = room.read();

        if room_guard.state != RoomState::Playing {
            return;
        }

        // Check if result was already broadcast (all players answered)
        let all_answered = room_guard.players.values().all(|p| {
            p.answers
                .iter()
                .any(|a| a.question_index == question_index)
        });

        if all_answered {
            // Result was already sent by submit_answer, skip
            return;
        }

        let question = &room_guard.questions[question_index];
        let correct_index = question["correct_index"]
            .as_u64()
            .or_else(|| question["correct"].as_u64())
            .unwrap_or(0) as usize;
        let explanation = question["explanation"].as_str().unwrap_or("").to_string();

        // Collect all answers for this question
        let player_answers: Vec<PlayerAnswer> = room_guard
            .players
            .values()
            .flat_map(|p| p.answers.iter())
            .filter(|a| a.question_index == question_index)
            .cloned()
            .collect();

        // Build leaderboard
        let mut leaderboard: Vec<PlayerScore> = room_guard
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

        leaderboard.sort_by(|a, b| b.score.cmp(&a.score));

        let _ = room_guard.tx.send(ServerMessage::QuestionResult {
            question_index,
            correct_index,
            explanation,
            player_answers,
            leaderboard,
        });
    }

    /// Advance to the next question.
    pub async fn advance_question(room: Arc<RwLock<Room>>) {
        let mut room_guard = room.write();

        if room_guard.state != RoomState::Playing {
            return;
        }

        let current_q = match room_guard.current_question {
            Some(q) => q,
            None => return,
        };

        let next_q = current_q + 1;

        // Check if game over (no more questions)
        if next_q >= room_guard.questions.len() {
            Self::end_game(&mut room_guard);
            return;
        }

        // Check if all players eliminated (lives mode)
        if room_guard.settings.mode == "lives" {
            let active_players = room_guard
                .players
                .values()
                .filter(|p| p.lives.unwrap_or(1) > 0)
                .count();

            if active_players == 0 {
                Self::end_game(&mut room_guard);
                return;
            }
        }

        // Advance to next question
        room_guard.current_question = Some(next_q);
        room_guard.question_start_time = None;

        // Broadcast new question
        let question = &room_guard.questions[next_q];
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

        let _ = room_guard.tx.send(ServerMessage::QuestionEmitted {
            question_index: next_q,
            question: question_data,
            time_limit: room_guard.settings.timer_secs,
        });

        info!(
            question = next_q,
            total = room_guard.questions.len(),
            "Advanced to next question"
        );
    }

    /// End the game.
    fn end_game(room: &mut Room) {
        room.state = RoomState::Finished;

        // Build final leaderboard
        let mut final_leaderboard: Vec<PlayerScore> = room
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

        final_leaderboard.sort_by(|a, b| b.score.cmp(&a.score));

        let winner = final_leaderboard.first().map(|p| p.username.clone());

        let _ = room.tx.send(ServerMessage::GameOver {
            final_leaderboard,
            winner,
        });

        info!("Game ended");
    }
}
