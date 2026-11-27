//! WebSocket handler for real-time gameplay.
//!
//! Protocol (client → server): `{ "type": "...", "data": { ... } }`
//!   * `join_room`     `{ code }`
//!   * `start_game`    `{}`                 (host only)
//!   * `submit_answer` `{ answer_index }`
//!   * `use_power_up`  `{ power_up }`       (one of `double_points`,
//!     `freeze_time`, `remove_two_options`)
//!   * `play_again`    `{}`                 (host only)
//!
//! Server → client uses the same envelope. Per-socket writer and per-room
//! broadcast forwarder run as separate tasks so the read loop never blocks on
//! slow clients.

use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use futures::{SinkExt, StreamExt};
use parking_lot::RwLock;
use serde_json::{json, Value};
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

use crate::domain::player::{Player, PLAYER_LIVES};
use crate::domain::power_up::{self, PowerUp, FREEZE_SECS};
use crate::domain::room::{Room, RoomState};
use crate::middleware::auth::AuthContext;
use crate::services::ai::{AiCapability, GenerationRequest};
use crate::services::questions::fallback_questions;
use sqlx::PgPool;

use crate::services::{accounts, history, leaderboard};
use crate::AppState;

/// How long the answer is shown before the next question appears.
const RESULT_PAUSE: Duration = Duration::from_secs(5);

/// Upgrade HTTP to WebSocket. Auth is enforced by `ws_auth_middleware`, which
/// injects [`AuthContext`] before the upgrade happens.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    axum::Extension(auth): axum::Extension<AuthContext>,
) -> impl IntoResponse {
    info!(user_id = auth.user_id, "WebSocket upgrade requested");
    ws.on_upgrade(move |socket| handle_socket(socket, state, auth))
}

async fn handle_socket(socket: WebSocket, state: AppState, auth: AuthContext) {
    let pool = Some(state.db.pool().clone());
    let (mut sender, mut receiver) = socket.split();
    let (out_tx, mut out_rx) = mpsc::unbounded_channel::<String>();

    let writer: JoinHandle<()> = tokio::spawn(async move {
        while let Some(text) = out_rx.recv().await {
            if sender.send(Message::Text(text)).await.is_err() {
                break;
            }
        }
    });

    let sid = uuid::Uuid::new_v4().to_string();
    let mut room_arc: Option<Arc<RwLock<Room>>> = None;
    let mut forwarder: Option<JoinHandle<()>> = None;

    info!(user_id = auth.user_id, "WebSocket connected");

    while let Some(incoming) = receiver.next().await {
        let text = match incoming {
            Ok(Message::Text(text)) => text,
            Ok(Message::Close(_)) => break,
            Ok(_) => continue,
            Err(_) => break,
        };

        let Ok(message) = serde_json::from_str::<Value>(&text) else {
            continue;
        };

        match message["type"].as_str().unwrap_or_default() {
            "join_room" => {
                let code = message["data"]["code"]
                    .as_str()
                    .unwrap_or_default()
                    .to_uppercase();

                let Some(room) = state.rooms.get_arc(&code) else {
                    send_json(
                        &out_tx,
                        json!({"type": "error", "data": {"message": "Комната не найдена"}}),
                    );
                    continue;
                };

                let (topic, mode, is_host, players, subscription) = {
                    let mut room = room.write();
                    // A reload opens a new socket for the same account: carry the score over
                    // instead of leaving a ghost player behind.
                    let carried = room
                        .players
                        .values()
                        .find(|player| player.user_id == Some(auth.user_id))
                        .map(|player| {
                            (
                                player.score,
                                player.streak,
                                player.total_correct,
                                player.lives,
                                player.power_ups.clone(),
                                player.double_points,
                                player.is_spectator,
                                player.infinite_lives,
                            )
                        })
                        .unwrap_or((0, 0, 0, 3, Vec::new(), false, false, false));
                    room.players
                        .retain(|_, player| player.user_id != Some(auth.user_id));
                    let mut player = Player::new(
                        sid.clone(),
                        auth.username.clone(),
                        Some(auth.user_id),
                        Some(auth.username.clone()),
                    );
                    player.score = carried.0;
                    player.streak = carried.1;
                    player.total_correct = carried.2;
                    player.lives = carried.3;
                    player.power_ups = carried.4;
                    player.double_points = carried.5;
                    player.is_spectator = carried.6;
                    player.infinite_lives = carried.7;
                    player.outbox = Some(out_tx.clone());
                    room.players.insert(sid.clone(), player);
                    room.touch();
                    (
                        room.settings.topic.clone(),
                        room.settings.mode.clone(),
                        room.host_user_id == auth.user_id,
                        players_event(&room),
                        room.tx.subscribe(),
                    )
                };

                room_arc = Some(room.clone());
                if let Some(handle) = forwarder.take() {
                    handle.abort();
                }
                forwarder = Some(spawn_forwarder(subscription, out_tx.clone()));

                send_json(
                    &out_tx,
                    json!({
                        "type": "room_joined",
                        "data": { "code": code, "is_host": is_host, "topic": topic, "mode": mode }
                    }),
                );
                let (replay, hand) = {
                    let room = room.read();
                    let replay = match room.state {
                        RoomState::Playing => question_event(&room),
                        RoomState::Finished => Some(game_over_event(&room)),
                        _ => None,
                    };
                    let hand = room
                        .players
                        .get(&sid)
                        .map(|player| player.power_ups.clone())
                        .unwrap_or_default();
                    (replay, hand)
                };
                if let Some(event) = replay {
                    let _ = out_tx.send(event);
                }
                send_json(
                    &out_tx,
                    json!({"type": "power_ups", "data": {"items": hand.iter().map(|kind| kind.as_str()).collect::<Vec<_>>()}}),
                );
                let _ = room.read().tx.send(players);
            }

            "start_game" => {
                let Some(room) = room_arc.clone() else {
                    send_json(
                        &out_tx,
                        json!({"type": "error", "data": {"message": "Сначала войдите в комнату"}}),
                    );
                    continue;
                };

                let (is_host, topic, count, difficulty, num_options) = {
                    let room = room.read();
                    (
                        room.host_user_id == auth.user_id,
                        room.settings.topic.clone(),
                        room.settings.question_count,
                        room.settings.difficulty.clone(),
                        room.settings.num_options,
                    )
                };

                if !is_host {
                    send_json(
                        &out_tx,
                        json!({"type": "error", "data": {"message": "Только хост может начать игру"}}),
                    );
                    continue;
                }
                if !room.write().begin() {
                    send_json(
                        &out_tx,
                        json!({"type": "error", "data": {"message": "Игра уже началась"}}),
                    );
                    continue;
                }

                let request = GenerationRequest {
                    topic: topic.clone(),
                    count,
                    difficulty,
                    num_options,
                    capabilities: vec![AiCapability::QuestionGeneration],
                    exclude_hashes: Vec::new(),
                    bypass_cache: true,
                };

                // Generation can take several seconds; tell the room so the UI
                // shows progress instead of a frozen "waiting" screen.
                let _ = room.read().tx.send(
                    json!({
                        "type": "game_preparing",
                        "data": { "topic": topic, "count": count }
                    })
                    .to_string(),
                );

                let (questions, source) = match state.ai_router.generate_questions(&request).await {
                    Ok(response) => {
                        let provider = response.provider_used.clone();
                        (
                            response
                                .questions
                                .into_iter()
                                .filter_map(|question| serde_json::to_value(question).ok())
                                .collect::<Vec<_>>(),
                            format!("ai:{provider}"),
                        )
                    }
                    Err(error) => {
                        warn!(
                            error = %error,
                            topic = %topic,
                            "AI generation failed, using offline fallback bank"
                        );
                        (
                            fallback_questions(&topic, count)
                                .into_iter()
                                .filter_map(|question| serde_json::to_value(question).ok())
                                .collect::<Vec<_>>(),
                            "fallback".to_string(),
                        )
                    }
                };

                let questions = fit_count(questions, count);

                let (first_question, players, token, hands) = {
                    let mut room = room.write();
                    room.set_questions(questions, &source);
                    let mut hands = Vec::new();
                    let lives_mode = room.settings.mode == "lives";
                    for player in room.players.values_mut() {
                        player.power_ups = PowerUp::STARTING_HAND.to_vec();
                        player.double_points = false;
                        player.is_spectator = false;
                        player.lives = PLAYER_LIVES;
                        player.infinite_lives = !lives_mode;
                        if let Some(outbox) = player.outbox.as_ref() {
                            hands.push((outbox.clone(), power_ups_event(&player.power_ups)));
                        }
                    }
                    (
                        question_event(&room),
                        players_event(&room),
                        room.question_token,
                        hands,
                    )
                };

                let tx = room.read().tx.clone();
                for (outbox, event) in hands {
                    let _ = outbox.send(event);
                }
                if let Some(event) = first_question {
                    let _ = tx.send(event);
                }
                let _ = tx.send(players);
                spawn_timer(room, token, pool.clone());
            }

            "use_power_up" => {
                let Some(room) = room_arc.clone() else {
                    send_json(
                        &out_tx,
                        json!({"type": "error", "data": {"message": "Сначала войдите в комнату"}}),
                    );
                    continue;
                };
                let Some(kind) = message["data"]["power_up"]
                    .as_str()
                    .and_then(PowerUp::parse)
                else {
                    send_json(
                        &out_tx,
                        json!({"type": "error", "data": {"message": "Неизвестное усиление"}}),
                    );
                    continue;
                };

                let mut room = room.write();
                if room.state != RoomState::Playing {
                    send_json(
                        &out_tx,
                        json!({"type": "error", "data": {"message": "Усиления работают только в игре"}}),
                    );
                    continue;
                }

                // Needs the question, not the hand: compute before borrowing the player.
                let mut hidden: Vec<usize> = Vec::new();
                if kind == PowerUp::RemoveTwoOptions {
                    if let Some(question) = room.questions.get(room.current_q) {
                        hidden = power_up::wrong_options(question);
                    }
                }

                let (name, hand) = {
                    let Some(player) = room.players.get_mut(&sid) else {
                        continue;
                    };
                    if !player.spend(kind) {
                        send_json(
                            &out_tx,
                            json!({"type": "error", "data": {"message": "Этого усиления нет в запасе"}}),
                        );
                        continue;
                    }
                    if kind == PowerUp::DoublePoints {
                        player.double_points = true;
                    }
                    (
                        player.display_name.clone(),
                        power_ups_event(&player.power_ups),
                    )
                };

                if kind == PowerUp::FreezeTime {
                    if let Some(deadline) = room.deadline {
                        room.deadline = Some(deadline + Duration::from_secs(FREEZE_SECS));
                    }
                }
                room.touch();

                let seconds = if kind == PowerUp::FreezeTime {
                    FREEZE_SECS
                } else {
                    0
                };
                let _ = room.tx.send(
                    json!({
                        "type": "power_up_used",
                        "data": { "player": name, "power_up": kind.as_str(), "seconds": seconds }
                    })
                    .to_string(),
                );
                if let Some(outbox) = room.players.get(&sid).and_then(|p| p.outbox.clone()) {
                    let _ = outbox.send(hand);
                    if kind == PowerUp::RemoveTwoOptions {
                        let _ = outbox.send(
                            json!({"type": "options_hidden", "data": {"indexes": hidden}})
                                .to_string(),
                        );
                    }
                }
            }

            "submit_answer" => {
                let Some(room) = room_arc.clone() else {
                    continue;
                };
                let index = message["data"]["answer_index"].as_u64().unwrap_or(0) as usize;

                let (advance, token, players, notice, eliminated) = {
                    let mut room = room.write();
                    if room.state != RoomState::Playing {
                        continue;
                    }
                    let Some(correct) = room
                        .questions
                        .get(room.current_q)
                        .and_then(|question| question["correct_index"].as_u64())
                        .map(|value| value as usize)
                    else {
                        continue;
                    };
                    let lives_mode = room.settings.mode == "lives";

                    let mut knocked_out = false;
                    let mut eliminated = false;
                    if let Some(player) = room.players.get_mut(&sid) {
                        eliminated = player.is_spectator;
                        if !eliminated && !player.answered {
                            player.answered = true;
                            player.answer_index = Some(index);
                            player.answer_time = Some(Instant::now());
                            player.award(index == correct, lives_mode);
                            if lives_mode && player.lives == 0 {
                                player.is_spectator = true;
                                player.instant_answer = false;
                                knocked_out = true;
                            }
                        }
                    }

                    let advance = room.all_answered();
                    let token = room.question_token;
                    room.touch();
                    let notice = if knocked_out {
                        room.players.get(&sid).and_then(|p| p.outbox.clone())
                    } else {
                        None
                    };
                    (
                        advance,
                        token,
                        Some(players_event(&room)),
                        notice,
                        eliminated,
                    )
                };

                if eliminated {
                    send_json(
                        &out_tx,
                        json!({"type": "error", "data": {"message": "Вы выбыли из игры"}}),
                    );
                    continue;
                }
                if let Some(outbox) = notice {
                    let _ = outbox.send(
                        json!({"type": "eliminated", "data": {"message": "Жизни кончились"}})
                            .to_string(),
                    );
                }
                if let Some(event) = players {
                    let _ = room.read().tx.send(event);
                }
                if advance {
                    reveal_and_advance(room, token, pool.clone()).await;
                }
            }

            "play_again" => {
                let Some(room) = room_arc.clone() else {
                    continue;
                };
                let players = {
                    let mut room = room.write();
                    if room.host_user_id != auth.user_id {
                        continue;
                    }
                    room.reset_for_rematch();
                    for player in room.players.values() {
                        if let Some(outbox) = player.outbox.as_ref() {
                            let _ = outbox.send(power_ups_event(&player.power_ups));
                        }
                    }
                    Some(players_event(&room))
                };
                if let Some(event) = players {
                    let _ = room.read().tx.send(event);
                }
            }

            other => debug!(
                user_id = auth.user_id,
                kind = other,
                "ignoring unknown ws message"
            ),
        }
    }

    if let Some(handle) = forwarder {
        handle.abort();
    }
    if let Some(room) = room_arc {
        let event = {
            let mut room = room.write();
            if room.state == RoomState::Waiting {
                room.players.remove(&sid);
            } else if let Some(player) = room.players.get_mut(&sid) {
                player.disconnected_at = Some(Instant::now());
            }
            room.touch();
            players_event(&room)
        };
        let _ = room.read().tx.send(event);
    }
    writer.abort();
    info!(user_id = auth.user_id, "WebSocket disconnected");
}

fn send_json(out: &mpsc::UnboundedSender<String>, value: Value) {
    let _ = out.send(value.to_string());
}

fn spawn_forwarder(
    mut subscription: broadcast::Receiver<String>,
    out: mpsc::UnboundedSender<String>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            match subscription.recv().await {
                Ok(text) => {
                    if out.send(text).is_err() {
                        break;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    debug!(skipped, "websocket client lagged behind room broadcast");
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    })
}

fn power_ups_event(hand: &[PowerUp]) -> String {
    json!({
        "type": "power_ups",
        "data": { "items": hand.iter().map(|kind| kind.as_str()).collect::<Vec<_>>() }
    })
    .to_string()
}

fn game_over_event(room: &Room) -> String {
    let scores: Vec<Value> = room
        .players
        .values()
        .map(|player| {
            json!({
                "name": player.display_name,
                "score": player.score,
                "lives": player.lives,
                "eliminated": player.is_spectator,
            })
        })
        .collect();
    json!({"type": "game_over", "data": {"scores": scores}}).to_string()
}

fn players_event(room: &Room) -> String {
    let players: Vec<Value> = room
        .players
        .values()
        .map(|player| {
            json!({
                "name": player.display_name,
                "score": player.score,
                "lives": player.lives,
                "infinite_lives": player.infinite_lives,
                "eliminated": player.is_spectator,
                "disconnected": player.is_disconnected(),
                "is_host": player.user_id == Some(room.host_user_id),
                "is_ready": true,
            })
        })
        .collect();
    json!({"type": "players_updated", "data": {"players": players}}).to_string()
}

fn question_event(room: &Room) -> Option<String> {
    let question = room.questions.get(room.current_q)?;
    let time_limit = room.settings.timer_secs;
    Some(
        json!({
            "type": "question_emitted",
            "data": {
                "question": {
                    "question": question["question"],
                    "options": question["options"],
                    "time_limit": time_limit,
                },
                "time_limit": time_limit,
                "index": room.current_q,
                "total": room.questions.len(),
                "source": room.question_source,
            }
        })
        .to_string(),
    )
}

/// Per-player results plus the metadata row written to `games`.
type FinishedGame = (Vec<(i64, String, i64)>, history::GameInfo);

/// Reveal the current answer, then move to the next question or finish.
/// Idempotent per `token`: a stale timer (or a duplicate "all answered" call)
/// sees a mismatched token and returns without advancing.
async fn reveal_and_advance(room: Arc<RwLock<Room>>, token: u64, pool: Option<PgPool>) {
    let mut events: Vec<String> = Vec::new();
    let mut schedule_next: Option<u64> = None;
    let finished: Option<FinishedGame>;

    {
        let mut room = room.write();
        if room.question_token != token || room.state != RoomState::Playing {
            return;
        }

        if let Some(question) = room.questions.get(room.current_q) {
            let correct = question["correct_index"].as_u64().unwrap_or(0) as usize;
            let explanation = question["explanation"].as_str().unwrap_or("").to_string();
            events.push(
                json!({
                    "type": "question_result",
                    "data": { "correctIndex": correct, "explanation": explanation }
                })
                .to_string(),
            );
        }

        room.current_q += 1;
        room.question_token += 1;

        // In survival mode the last player standing ends the game early.
        let out_of_lives = room.settings.mode == "lives"
            && room
                .players
                .values()
                .all(|player| player.is_spectator || player.is_disconnected());

        if room.current_q >= room.questions.len() || out_of_lives {
            room.state = RoomState::Finished;
            room.deadline = None;
            events.push(game_over_event(&room));
            events.push(players_event(&room));

            let mut results = room
                .players
                .values()
                .filter_map(|player| {
                    player
                        .user_id
                        .map(|id| (id, player.username.clone(), player.score))
                })
                .collect::<Vec<_>>();
            results.sort_by_key(|entry| std::cmp::Reverse(entry.2));
            let duration = room
                .started_at
                .map(|started| started.elapsed().as_secs() as i32)
                .unwrap_or(0);
            let info = history::GameInfo {
                room_code: room.code.clone(),
                topic: room.settings.topic.clone(),
                mode: room.settings.mode.clone(),
                players: room.players.len() as i32,
                duration_secs: duration,
            };
            finished = Some((results, info));
        } else {
            finished = None;
            // Hold the result screen briefly before the next question so
            // players can read the correct answer.
            room.reset_answers();
            room.deadline = None;
            schedule_next = Some(room.question_token);
        }
        room.touch();
    }

    let tx = room.read().tx.clone();
    for event in events {
        let _ = tx.send(event);
    }
    if let (Some(pool), Some((results, info))) = (pool.clone(), finished) {
        tokio::spawn(async move { persist_results(&pool, results, info).await });
    }
    if let Some(token) = schedule_next {
        let room = room.clone();
        let pool = pool.clone();
        tokio::spawn(async move {
            tokio::time::sleep(RESULT_PAUSE).await;
            let event = {
                let room = room.read();
                if room.question_token != token || room.state != RoomState::Playing {
                    return;
                }
                question_event(&room)
            };
            if let Some(event) = event {
                let _ = room.read().tx.send(event);
                let secs = { room.read().settings.timer_secs };
                room.write().deadline = Some(Instant::now() + Duration::from_secs(secs));
            }
            spawn_timer(room, token, pool);
        });
    }
}

fn spawn_timer(room: Arc<RwLock<Room>>, token: u64, pool: Option<PgPool>) {
    tokio::spawn(async move {
        let secs = { room.read().settings.timer_secs };
        tokio::time::sleep(Duration::from_secs(secs)).await;
        reveal_and_advance(room, token, pool).await;
    });
}

/// A provider answers with as many questions as it likes; the room plays the
/// number the host asked for. Trimming here keeps the round count and the
/// "question 3 of 5" label honest even when a provider ignores `count`.
fn fit_count(mut questions: Vec<Value>, count: usize) -> Vec<Value> {
    if questions.len() > count {
        questions.truncate(count);
    }
    questions
}

/// Write finished-game stats to Postgres: per-user totals, leaderboard row, coins
/// and the game history rows that back the profile page.
async fn persist_results(pool: &PgPool, results: Vec<(i64, String, i64)>, info: history::GameInfo) {
    let best = results
        .iter()
        .map(|(_, _, score)| *score)
        .max()
        .unwrap_or(0);
    let mut outcomes = Vec::with_capacity(results.len());
    for (index, (user_id, username, score)) in results.into_iter().enumerate() {
        let won = best > 0 && score == best;
        if let Err(error) = accounts::update_stats(pool, user_id, score, won, score / 10).await {
            warn!(user_id, %error, "failed to update user stats");
            continue;
        }
        if let Err(error) = leaderboard::update_leaderboard(
            pool,
            user_id,
            &username,
            score,
            won,
            info.duration_secs,
        )
        .await
        {
            warn!(user_id, %error, "failed to update leaderboard");
        }
        if score > 0 {
            let _ = accounts::add_coins(pool, user_id, (score / 20) as i32).await;
        }
        outcomes.push(history::GameOutcome {
            user_id,
            username,
            score,
            place: index as i32 + 1,
        });
    }

    if let Err(error) = history::save_game(pool, &info, &outcomes).await {
        warn!(room = %info.room_code, %error, "failed to save game history");
    }
}

#[cfg(test)]
mod tests {
    use super::fit_count;
    use serde_json::{json, Value};

    fn questions(count: usize) -> Vec<Value> {
        (0..count)
            .map(|i| json!({"question": format!("q{i}"), "options": ["a", "b"]}))
            .collect()
    }

    #[test]
    fn provider_overdelivery_is_trimmed_to_the_requested_count() {
        assert_eq!(fit_count(questions(10), 5).len(), 5);
    }

    #[test]
    fn a_shorter_answer_is_left_alone() {
        assert_eq!(fit_count(questions(3), 5).len(), 3);
    }

    #[test]
    fn trimming_keeps_the_first_questions() {
        let trimmed = fit_count(questions(4), 2);
        let first = trimmed[0]["question"].as_str().unwrap();
        assert_eq!(first, "q0");
    }
}
