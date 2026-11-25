//! WebSocket handler for real-time gameplay.
//!
//! Protocol (client → server): `{ "type": "...", "data": { ... } }`
//!   * `join_room`     `{ code }`
//!   * `start_game`    `{}`                 (host only)
//!   * `submit_answer` `{ answer_index }`
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

use crate::domain::player::Player;
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

                let (topic, is_host, players, subscription) = {
                    let mut room = room.write();
                    room.players.insert(
                        sid.clone(),
                        Player::new(
                            sid.clone(),
                            auth.username.clone(),
                            Some(auth.user_id),
                            Some(auth.username.clone()),
                        ),
                    );
                    room.touch();
                    (
                        room.settings.topic.clone(),
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
                        "data": { "code": code, "is_host": is_host, "topic": topic }
                    }),
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

                let (is_host, topic, count, difficulty, num_options, waiting) = {
                    let room = room.read();
                    (
                        room.host_user_id == auth.user_id,
                        room.settings.topic.clone(),
                        room.settings.question_count,
                        room.settings.difficulty.clone(),
                        room.settings.num_options,
                        room.state == RoomState::Waiting,
                    )
                };

                if !is_host {
                    send_json(
                        &out_tx,
                        json!({"type": "error", "data": {"message": "Только хост может начать игру"}}),
                    );
                    continue;
                }
                if !waiting || !room.write().try_start() {
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

                let (first_question, players, token) = {
                    let mut room = room.write();
                    room.set_questions(questions, &source);
                    (question_event(&room), players_event(&room), room.question_token)
                };

                let tx = room.read().tx.clone();
                if let Some(event) = first_question {
                    let _ = tx.send(event);
                }
                let _ = tx.send(players);
                spawn_timer(room, token, pool.clone());
            }

            "submit_answer" => {
                let Some(room) = room_arc.clone() else {
                    continue;
                };
                let index = message["data"]["answer_index"].as_u64().unwrap_or(0) as usize;

                let mut advance = false;
                let mut token = 0;
                let mut players = None;
                {
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

                    if let Some(player) = room.players.get_mut(&sid) {
                        if !player.answered {
                            player.answered = true;
                            player.answer_index = Some(index);
                            player.answer_time = Some(Instant::now());
                            if index == correct {
                                player.score += 100;
                                player.streak += 1;
                                player.total_correct += 1;
                            } else {
                                player.streak = 0;
                                if lives_mode {
                                    player.lives = player.lives.saturating_sub(1);
                                }
                            }
                        }
                    }

                    advance = room.all_answered();
                    token = room.question_token;
                    room.touch();
                    players = Some(players_event(&room));
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
                let mut players = None;
                {
                    let mut room = room.write();
                    if room.host_user_id != auth.user_id {
                        continue;
                    }
                    room.state = RoomState::Waiting;
                    room.questions.clear();
                    room.current_q = 0;
                    room.started_at = None;
                    // Bump the token so any in-flight timer becomes a no-op.
                    room.question_token += 1;
                    room.deadline = None;
                    for player in room.players.values_mut() {
                        player.score = 0;
                        player.streak = 0;
                        player.total_correct = 0;
                        player.answered = false;
                        player.answer_index = None;
                        player.answer_time = None;
                        player.lives = 3;
                    }
                    room.touch();
                    players = Some(players_event(&room));
                }
                if let Some(event) = players {
                    let _ = room.read().tx.send(event);
                }
            }

            other => debug!(user_id = auth.user_id, kind = other, "ignoring unknown ws message"),
        }
    }

    if let Some(handle) = forwarder {
        handle.abort();
    }
    if let Some(room) = room_arc {
        let event = {
            let mut room = room.write();
            room.players.remove(&sid);
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

fn players_event(room: &Room) -> String {
    let players: Vec<Value> = room
        .players
        .values()
        .map(|player| {
            json!({
                "name": player.display_name,
                "score": player.score,
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

/// Reveal the current answer, then move to the next question or finish.
/// Idempotent per `token`: a stale timer (or a duplicate "all answered" call)
/// sees a mismatched token and returns without advancing.
async fn reveal_and_advance(room: Arc<RwLock<Room>>, token: u64, pool: Option<PgPool>) {
    let mut events: Vec<String> = Vec::new();
    let mut schedule_next: Option<u64> = None;
    let mut finished: Option<(Vec<(i64, String, i64)>, history::GameInfo)> = None;

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

        if room.current_q >= room.questions.len() {
            room.state = RoomState::Finished;
            room.deadline = None;
            let scores: Vec<Value> = room
                .players
                .values()
                .map(|player| json!({"name": player.display_name, "score": player.score}))
                .collect();
            events.push(json!({"type": "game_over", "data": {"scores": scores}}).to_string());
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
            results.sort_by(|a, b| b.2.cmp(&a.2));
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

/// Write finished-game stats to Postgres: per-user totals, leaderboard row, coins
/// and the game history rows that back the profile page.
async fn persist_results(
    pool: &PgPool,
    results: Vec<(i64, String, i64)>,
    info: history::GameInfo,
) {
    let best = results.iter().map(|(_, _, score)| *score).max().unwrap_or(0);
    let mut outcomes = Vec::with_capacity(results.len());
    for (index, (user_id, username, score)) in results.into_iter().enumerate() {
        let won = best > 0 && score == best;
        if let Err(error) = accounts::update_stats(pool, user_id, score, won, score / 10).await {
            warn!(user_id, %error, "failed to update user stats");
            continue;
        }
        if let Err(error) =
            leaderboard::update_leaderboard(pool, user_id, &username, score, won, info.duration_secs)
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
