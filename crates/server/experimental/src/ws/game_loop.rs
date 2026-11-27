//! Game loop logic.
//!
//! Manages question timers, transitions between questions, and game over conditions.
//! Persists game results to the database on game end.

use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::RwLock;
use sqlx::PgPool;
use tokio::time::sleep;
use tracing::{error, info, warn};

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
    /// - Persisting results to database
    pub async fn run(room: Arc<RwLock<Room>>, ai_router: Arc<AiRouter>, db: Option<PgPool>) {
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

            // Wait before advancing (5 seconds so players can see results)
            sleep(Duration::from_secs(5)).await;

            // Advance to next question
            Self::advance_question(room.clone(), db.clone()).await;
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
    pub async fn advance_question(room: Arc<RwLock<Room>>, db: Option<PgPool>) {
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
            let room_clone = room.clone();
            // Need to drop the write lock before calling async end_game
            drop(room_guard);
            Self::end_game(room_clone, db).await;
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
                let room_clone = room.clone();
                drop(room_guard);
                Self::end_game(room_clone, db).await;
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

    /// End the game and persist results.
    async fn end_game(room: Arc<RwLock<Room>>, db: Option<PgPool>) {
        let mut room_guard = room.write();
        room_guard.state = RoomState::Finished;

        // Build final leaderboard
        let mut final_leaderboard: Vec<PlayerScore> = room_guard
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

        let winner_username = final_leaderboard.first().map(|p| p.username.clone());
        let winner_id = final_leaderboard.first().map(|p| p.user_id);

        let _ = room_guard.tx.send(ServerMessage::GameOver {
            final_leaderboard: final_leaderboard.clone(),
            winner: winner_username,
        });

        info!("Game ended");

        // Persist results to database
        if let Some(pool) = db {
            let scores = final_leaderboard.clone();
            let room_code = room_guard.code.clone();
            let room_mode = room_guard.settings.mode.clone();
            let room_topic = room_guard.settings.topic.clone();
            let question_count = room_guard.questions.len();

            drop(room_guard);

            if let Err(e) = Self::persist_game_results(
                &pool,
                &room_code,
                &room_mode,
                &room_topic,
                question_count,
                &scores,
                winner_id,
            )
            .await
            {
                error!(error = %e, room_code, "Failed to persist game results");
            }
        }
    }

    /// Persist game results to database.
    ///
    /// Updates:
    /// - `users` table: xp, coins, games_played, wins
    /// - `leaderboard` table: total_score, games_played, wins
    /// - `room_history` table: game snapshot
    async fn persist_game_results(
        pool: &PgPool,
        room_code: &str,
        mode: &str,
        topic: &str,
        question_count: usize,
        scores: &[PlayerScore],
        winner_id: Option<i64>,
    ) -> Result<(), sqlx::Error> {
        let mut tx = pool.begin().await?;

        for (rank, score) in scores.iter().enumerate() {
            let user_id = score.user_id;
            let game_score = score.score;
            let is_winner = Some(user_id) == winner_id;

            // Calculate rewards
            let xp_earned = game_score as i64 + if is_winner { 50 } else { 10 };
            let coins_earned = (game_score / 10) as i32 + if is_winner { 20 } else { 5 };

            // Update users table
            sqlx::query(
                "UPDATE users SET
                    xp = xp + $1,
                    coins = coins + $2,
                    games_played = games_played + 1,
                    wins = wins + CASE WHEN $3 THEN 1 ELSE 0 END,
                    total_score = total_score + $4,
                    last_seen = NOW()
                 WHERE id = $5"
            )
            .bind(xp_earned)
            .bind(coins_earned)
            .bind(is_winner)
            .bind(game_score)
            .bind(user_id)
            .execute(&mut *tx)
            .await?;

            // Update leaderboard (insert or update)
            sqlx::query(
                "INSERT INTO leaderboard (user_id, username, total_score, games_played, wins, last_seen)
                 SELECT $1, u.username, $2, 1, CASE WHEN $3 THEN 1 ELSE 0 END, NOW()
                 FROM users u WHERE u.id = $1
                 ON CONFLICT (user_id) DO UPDATE SET
                    total_score = leaderboard.total_score + $2,
                    games_played = leaderboard.games_played + 1,
                    wins = leaderboard.wins + CASE WHEN $3 THEN 1 ELSE 0 END,
                    last_seen = NOW()"
            )
            .bind(user_id)
            .bind(game_score)
            .bind(is_winner)
            .execute(&mut *tx)
            .await?;

            info!(
                user_id,
                username = %score.username,
                rank = rank + 1,
                score = game_score,
                xp_earned,
                coins_earned,
                is_winner,
                "Persisted player results"
            );
        }

        // Save room history
        let players_json = serde_json::json!(
            scores.iter().map(|s| {
                serde_json::json!({
                    "user_id": s.user_id,
                    "username": s.username,
                    "score": s.score,
                    "correct_answers": s.correct_answers,
                    "total_answers": s.total_answers,
                })
            }).collect::<Vec<_>>()
        );

        sqlx::query(
            "INSERT INTO room_history (room_code, mode, topic, duration_secs, players_json)
             VALUES ($1, $2, $3, $4, $5)"
        )
        .bind(room_code)
        .bind(mode)
        .bind(topic)
        .bind((question_count as i32) * 35) // estimated: 30s timer + 5s result per question
        .bind(&players_json)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;

        info!(room_code, players = scores.len(), "Game results persisted");

        Ok(())
    }
}
