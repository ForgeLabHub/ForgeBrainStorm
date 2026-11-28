//! WebSocket handler for real-time gameplay.
//!
//! Protocol (client → server): `{ "type": "...", "data": { ... } }`
//!   * `join_room`     `{ code }`
//!   * `start_game`    `{}`                 (host only)
//!   * `submit_answer` `{ answer_index }`
//!   * `use_power_up`  `{ power_up }`       (one of `double_points`,
//!     `freeze_time`, `remove_two_options`)
//!   * `play_again`    `{}`                 (host only)
//!   * `chat_send`     `{ text }`
//!   * `emoji`         `{ emoji, target? }`  (palette is server-side)
//!   * `cheat_power`   `{ power, target?, payload? }` — `target` is a player
//!     id or display name for `hijack`/`mute`/`jinx`/`blind`, `payload` is
//!     `{ question, options, correct_index }` for `rewrite`
//!
//! Cheat powers: `peek`, `freeze`, `steal`, `double`, `hijack`, `rewrite`,
//! `mute`, `jinx`, `leech`, `blind`, `swap`, `hint`. A power that changes
//! nothing (already cursed, nobody to steal from) is refunded and answered
//! with `cheat_no_effect`.
//!
//! Server → client uses the same envelope. Per-socket writer and per-room
//! broadcast forwarder run as separate tasks so the read loop never blocks on
//! slow clients.

use std::collections::HashSet;
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

use crate::domain::chat::{self, ChatError, MAX_CHAT_CHARS};
use crate::domain::cheat::{CheatPower, BLIND_SECS, CHEAT_FREEZE_SECS, STEAL_POINTS};
use crate::domain::mode::{GameMode, TEAM_NAMES};
use crate::domain::player::Player;
use crate::domain::power_up::{self, PowerUp, FREEZE_SECS};
use crate::domain::proposal::{Proposal, ProposalInfo, ProposalType};
use crate::domain::room::{Room, RoomState};
use crate::metrics::Metrics;
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

    let metrics = Metrics::global();
    metrics.ws_connections_active.inc();
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

        // An admin can close a room while its sockets are still attached. The
        // `Arc` survives the registry entry, so the room itself has to refuse
        // further messages instead of pretending the game goes on.
        if room_arc.as_ref().is_some_and(|room| room.read().closed) {
            send_json(
                &out_tx,
                json!({"type": "error", "data": {
                    "message": "Комната закрыта администратором",
                    "code": "room_closed"
                }}),
            );
            break;
        }

        match message["type"].as_str().unwrap_or_default() {
            "join_room" => {
                let code = message["data"]["code"]
                    .as_str()
                    .unwrap_or_default()
                    .to_uppercase();

                let Some(room) = state.rooms.get_arc(&code) else {
                    send_json(
                        &out_tx,
                        json!({"type": "error", "data": {"message": "Комната не найдена", "code": "room_not_found"}}),
                    );
                    continue;
                };

                if room.read().closed {
                    send_json(
                        &out_tx,
                        json!({"type": "error", "data": {"message": "Комната закрыта администратором", "code": "room_closed"}}),
                    );
                    continue;
                }

                let (topic, mode, is_host, players, subscription) = {
                    let mut room = room.write();
                    // A reload opens a new socket for the same account: carry the game
                    // state over instead of leaving a ghost player behind.
                    let carried = room
                        .players
                        .values()
                        .find(|player| player.user_id == Some(auth.user_id))
                        .map(Player::reconnect_state);
                    room.players
                        .retain(|_, player| player.user_id != Some(auth.user_id));
                    let mut player = Player::new(
                        sid.clone(),
                        auth.username.clone(),
                        Some(auth.user_id),
                        Some(auth.username.clone()),
                    );
                    if let Some(state) = carried {
                        player.apply_reconnect(state);
                    }
                    player.is_cheat = auth.role.is_cheater();
                    player.outbox = Some(out_tx.clone());
                    room.players.insert(sid.clone(), player);
                    room.touch();
                    (
                        room.settings.topic.clone(),
                        room.settings.mode.as_str().to_string(),
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
                        "data": {
                            "code": code,
                            "is_host": is_host,
                            "topic": topic,
                            "mode": mode,
                            "mode_title": GameMode::parse(&mode)
                                .map(|parsed| parsed.title())
                                .unwrap_or("Классика"),
                        }
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
                let history = room.read().chat.history();
                send_json(
                    &out_tx,
                    json!({"type": "chat_history", "data": {"messages": history}}),
                );
                let cheat = room
                    .read()
                    .players
                    .get(&sid)
                    .filter(|player| player.is_cheat)
                    .map(|player| player.cheat.to_json());
                if let Some(budget) = cheat {
                    send_json(
                        &out_tx,
                        json!({"type": "cheat_state", "data": {"powers": budget}}),
                    );
                }
                // A reload keeps the curses: the client needs them to explain
                // why a correct answer scored nothing or why the buttons are
                // dead, and `players_updated` only carries the public ones.
                let status = room.read().players.get(&sid).map(|player| {
                    json!({
                        "muted": player.muted,
                        "jinxed": player.jinxed,
                        "hijacked": player.hijacked,
                        "eliminated": player.is_spectator,
                        "blind_seconds": player.blind_until.map(|until| {
                            until.saturating_duration_since(Instant::now()).as_secs()
                        }),
                    })
                });
                if let Some(data) = status {
                    send_json(&out_tx, json!({"type": "status", "data": data}));
                }
                let proposals = {
                    let room = room.read();
                    proposals_event(&room)
                };
                send_json(
                    &out_tx,
                    json!({"type": "proposals_list", "data": proposals}),
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

                let generation_started = Instant::now();
                let requested_provider = room.read().settings.ai_provider.clone();
                let (questions, source) = match state
                    .ai_router
                    .generate_questions_with_provider(&request, requested_provider.as_deref())
                    .await
                {
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

                metrics.record_ai_generation(
                    source.trim_start_matches("ai:"),
                    if source.starts_with("ai:") {
                        "ok"
                    } else {
                        "fallback"
                    },
                    generation_started.elapsed().as_secs_f64(),
                );

                let questions = fit_count(questions, count, &topic);

                let (first_question, players, token, hands) = {
                    let mut room = room.write();
                    room.set_questions(questions, &source);
                    metrics.record_game_start(room.settings.mode.as_str());
                    room.assign_teams();
                    let mode = room.settings.mode;
                    let mut hands = Vec::new();
                    for player in room.players.values_mut() {
                        player.power_ups = PowerUp::STARTING_HAND.to_vec();
                        player.double_points = false;
                        player.is_spectator = false;
                        player.lives = mode.lives().unwrap_or(1);
                        player.infinite_lives = mode.lives().is_none();
                        player.cheat.reset();
                        if let Some(outbox) = player.outbox.as_ref() {
                            hands.push((outbox.clone(), power_ups_event(&player.power_ups)));
                            if player.is_cheat {
                                hands.push((
                                    outbox.clone(),
                                    json!({"type": "cheat_state", "data": {"powers": player.cheat.to_json()}})
                                        .to_string(),
                                ));
                            }
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

            "chat_send" => {
                let Some(room) = room_arc.clone() else {
                    send_json(
                        &out_tx,
                        json!({"type": "error", "data": {"message": "Сначала войдите в комнату"}}),
                    );
                    continue;
                };

                let raw = message["data"]["text"].as_str().unwrap_or_default();
                let Some((from, muted, locked)) = chat_sender(&room, &sid) else {
                    continue;
                };
                if locked {
                    send_json(
                        &out_tx,
                        json!({"type": "error", "data": {
                            "message": "Чат комнаты заморожен администратором",
                            "code": "chat_locked"
                        }}),
                    );
                    continue;
                }
                if muted {
                    send_json(
                        &out_tx,
                        json!({"type": "error", "data": {"message": "Читер запретил вам писать в чат", "code": "chat_muted"}}),
                    );
                    continue;
                }

                let result = {
                    let mut room = room.write();
                    room.chat.push(&sid, &from, raw)
                };
                match result {
                    Ok(message) => {
                        room.write().touch();
                        let event = json!({"type": "chat_message", "data": message});
                        let _ = room.read().tx.send(event.to_string());
                    }
                    Err(error) => send_chat_error(&out_tx, error),
                }
            }

            "emoji" => {
                let Some(room) = room_arc.clone() else {
                    send_json(
                        &out_tx,
                        json!({"type": "error", "data": {"message": "Сначала войдите в комнату"}}),
                    );
                    continue;
                };

                let emoji = message["data"]["emoji"].as_str().unwrap_or_default();
                let target = message["data"]["target"].as_str().map(str::to_owned);
                let Some((from, muted, locked)) = chat_sender(&room, &sid) else {
                    continue;
                };
                if muted {
                    send_json(
                        &out_tx,
                        json!({"type": "error", "data": {"message": "Читер запретил вам писать в чат", "code": "chat_muted"}}),
                    );
                    continue;
                }
                if locked {
                    send_json(
                        &out_tx,
                        json!({"type": "error", "data": {
                            "message": "Чат комнаты заморожен администратором",
                            "code": "chat_locked"
                        }}),
                    );
                    continue;
                }

                let result = {
                    let mut room = room.write();
                    room.chat.push_emoji(&sid, &from, emoji)
                };
                match result {
                    Ok(message) => {
                        room.write().touch();
                        let event = json!({
                            "type": "emoji",
                            "data": { "from": from, "emoji": message.text, "target": target }
                        });
                        let _ = room.read().tx.send(event.to_string());
                    }
                    Err(error) => send_chat_error(&out_tx, error),
                }
            }

            "cheat_power" => {
                let Some(room) = room_arc.clone() else {
                    send_json(
                        &out_tx,
                        json!({"type": "error", "data": {"message": "Сначала войдите в комнату"}}),
                    );
                    continue;
                };
                let Some(power) = message["data"]["power"]
                    .as_str()
                    .and_then(CheatPower::parse)
                else {
                    send_json(
                        &out_tx,
                        json!({"type": "error", "data": {"message": "Неизвестная способность"}}),
                    );
                    continue;
                };
                let target_raw = message["data"]["target"].as_str().map(str::to_owned);

                let mut public_events: Vec<String> = Vec::new();
                let mut private_events: Vec<String> = Vec::new();
                let mut victim_events: Vec<(String, String)> = Vec::new();
                let mut players_changed = false;
                let budget;

                {
                    let mut room = room.write();
                    if room.state != RoomState::Playing {
                        send_json(
                            &out_tx,
                            json!({"type": "error", "data": {"message": "Способности работают только в игре"}}),
                        );
                        continue;
                    }

                    let allowed = room
                        .players
                        .get(&sid)
                        .map(|player| player.is_cheat && player.cheat.remaining(power) > 0)
                        .unwrap_or(false);
                    if !allowed {
                        send_json(
                            &out_tx,
                            json!({"type": "error", "data": {"message": "Способность недоступна"}}),
                        );
                        continue;
                    }

                    // Validate the victim before spending: a mistyped name must
                    // not burn a use.
                    let target_sid = if power.needs_target() {
                        let resolved = target_raw
                            .as_deref()
                            .and_then(|needle| room.resolve_victim(needle, &sid));
                        match resolved {
                            Some(victim) => Some(victim),
                            None => {
                                send_json(
                                    &out_tx,
                                    json!({"type": "error", "data": {
                                        "message": "Такого игрока нет в игре",
                                        "code": "cheat_no_target"
                                    }}),
                                );
                                continue;
                            }
                        }
                    } else {
                        None
                    };

                    // A power that needs a live question is rejected between
                    // questions instead of silently doing nothing.
                    if power.needs_question() && room.questions.get(room.current_q).is_none() {
                        send_json(
                            &out_tx,
                            json!({"type": "error", "data": {
                                "message": "Сейчас нет активного вопроса",
                                "code": "cheat_no_question"
                            }}),
                        );
                        continue;
                    }

                    let rewrite = if power == CheatPower::Rewrite {
                        match parse_rewrite(&message["data"]["payload"]) {
                            Some(input) => Some(input),
                            None => {
                                send_json(
                                    &out_tx,
                                    json!({"type": "error", "data": {
                                        "message": "Вопрос: 3-200 символов, 2-6 вариантов, верный отмечен",
                                        "code": "cheat_bad_question"
                                    }}),
                                );
                                continue;
                            }
                        }
                    } else {
                        None
                    };

                    let name = room
                        .players
                        .get(&sid)
                        .map(|player| player.display_name.clone())
                        .unwrap_or_default();
                    if let Some(player) = room.players.get_mut(&sid) {
                        player.cheat.spend(power);
                    }

                    // Every arm reports whether it actually changed something.
                    // A no-op (nothing to steal, already cursed, clock already
                    // stopped) refunds the use: the budget pays for effects,
                    // not for clicks.
                    let applied = match power {
                        CheatPower::Peek => {
                            let correct = room
                                .questions
                                .get(room.current_q)
                                .and_then(|question| question["correct_index"].as_u64())
                                .map(|value| value as usize);
                            match correct {
                                Some(index) => {
                                    private_events.push(
                                        json!({"type": "cheat_reveal", "data": {"correct_index": index}})
                                            .to_string(),
                                    );
                                    true
                                }
                                None => false,
                            }
                        }
                        CheatPower::Double => {
                            let armed = room
                                .players
                                .get(&sid)
                                .map(|player| player.double_points)
                                .unwrap_or(true);
                            if armed {
                                false
                            } else {
                                if let Some(player) = room.players.get_mut(&sid) {
                                    player.double_points = true;
                                }
                                private_events.push(
                                    json!({"type": "cheat_armed", "data": {"power": "double"}})
                                        .to_string(),
                                );
                                true
                            }
                        }
                        CheatPower::Freeze => match room.deadline {
                            Some(deadline) => {
                                room.deadline =
                                    Some(deadline + Duration::from_secs(CHEAT_FREEZE_SECS));
                                public_events.push(
                                    json!({
                                        "type": "cheat_used",
                                        "data": {"player": name, "power": "freeze", "seconds": CHEAT_FREEZE_SECS}
                                    })
                                    .to_string(),
                                );
                                true
                            }
                            None => false,
                        },
                        CheatPower::Steal => {
                            let leader = room
                                .players
                                .iter()
                                .filter(|(other, player)| {
                                    **other != sid && !player.is_spectator && player.score > 0
                                })
                                .map(|(other, player)| (other.clone(), player.score))
                                .max_by_key(|(_, score)| *score);
                            match leader {
                                Some((leader_sid, leader_score)) => {
                                    let amount = leader_score.clamp(0, STEAL_POINTS);
                                    if let Some(player) = room.players.get_mut(&leader_sid) {
                                        player.score -= amount;
                                    }
                                    if let Some(player) = room.players.get_mut(&sid) {
                                        player.score += amount;
                                    }
                                    players_changed = true;
                                    public_events.push(
                                        json!({
                                            "type": "cheat_used",
                                            "data": {"player": name, "power": "steal", "points": amount}
                                        })
                                        .to_string(),
                                    );
                                    true
                                }
                                None => false,
                            }
                        }
                        CheatPower::Hijack => {
                            let victim = target_sid.clone().unwrap_or_default();
                            match room.hijack(&sid, &victim) {
                                Some((victim_name, points)) => {
                                    players_changed = true;
                                    public_events.push(
                                        json!({
                                            "type": "cheat_used",
                                            "data": {
                                                "player": name,
                                                "power": "hijack",
                                                "target": victim_name,
                                                "points": points
                                            }
                                        })
                                        .to_string(),
                                    );
                                    victim_events.push((
                                        victim,
                                        json!({
                                            "type": "hijacked",
                                            "data": {"by": name, "points": points}
                                        })
                                        .to_string(),
                                    ));
                                    true
                                }
                                None => false,
                            }
                        }
                        CheatPower::Rewrite => match rewrite {
                            Some(input) => {
                                if room.rewrite_question(input.text, input.options, input.correct) {
                                    public_events.push(
                                        json!({
                                            "type": "cheat_used",
                                            "data": {"player": name, "power": "rewrite"}
                                        })
                                        .to_string(),
                                    );
                                    if let Some(event) = question_event(&room) {
                                        public_events.push(event);
                                    }
                                    players_changed = true;
                                    true
                                } else {
                                    false
                                }
                            }
                            None => false,
                        },
                        CheatPower::Mute => {
                            let victim = target_sid.clone().unwrap_or_default();
                            match room.mute(&victim) {
                                Some(victim_name) => {
                                    players_changed = true;
                                    public_events.push(
                                        json!({
                                            "type": "cheat_used",
                                            "data": {"player": name, "power": "mute", "target": victim_name}
                                        })
                                        .to_string(),
                                    );
                                    victim_events.push((
                                        victim,
                                        json!({"type": "muted", "data": {"by": name}}).to_string(),
                                    ));
                                    true
                                }
                                None => false,
                            }
                        }
                        CheatPower::Jinx => {
                            let victim = target_sid.clone().unwrap_or_default();
                            match room.jinx(&victim) {
                                Some(victim_name) => {
                                    players_changed = true;
                                    public_events.push(
                                        json!({
                                            "type": "cheat_used",
                                            "data": {"player": name, "power": "jinx", "target": victim_name}
                                        })
                                        .to_string(),
                                    );
                                    victim_events.push((
                                        victim,
                                        json!({"type": "jinxed", "data": {"by": name}}).to_string(),
                                    ));
                                    true
                                }
                                None => false,
                            }
                        }
                        CheatPower::Blind => {
                            let victim = target_sid.clone().unwrap_or_default();
                            match room.blind(&victim) {
                                Some(victim_name) => {
                                    players_changed = true;
                                    public_events.push(
                                        json!({
                                            "type": "cheat_used",
                                            "data": {
                                                "player": name,
                                                "power": "blind",
                                                "target": victim_name,
                                                "seconds": BLIND_SECS
                                            }
                                        })
                                        .to_string(),
                                    );
                                    victim_events.push((
                                        victim,
                                        json!({
                                            "type": "blinded",
                                            "data": {"by": name, "seconds": BLIND_SECS}
                                        })
                                        .to_string(),
                                    ));
                                    true
                                }
                                None => false,
                            }
                        }
                        CheatPower::Leech => {
                            let (victims, taken) = room.leech(&sid);
                            if taken > 0 {
                                players_changed = true;
                                public_events.push(
                                    json!({
                                        "type": "cheat_used",
                                        "data": {
                                            "player": name,
                                            "power": "leech",
                                            "victims": victims,
                                            "points": taken
                                        }
                                    })
                                    .to_string(),
                                );
                                true
                            } else {
                                false
                            }
                        }
                        CheatPower::Swap => match room.swap_with_leader(&sid) {
                            Some((leader_name, old_score, new_score)) => {
                                players_changed = true;
                                public_events.push(
                                    json!({
                                        "type": "cheat_used",
                                        "data": {
                                            "player": name,
                                            "power": "swap",
                                            "target": leader_name,
                                            "from": old_score,
                                            "to": new_score
                                        }
                                    })
                                    .to_string(),
                                );
                                true
                            }
                            None => false,
                        },
                        CheatPower::Hint => {
                            let hidden = room
                                .questions
                                .get(room.current_q)
                                .map(power_up::wrong_options)
                                .unwrap_or_default();
                            if hidden.is_empty() {
                                false
                            } else {
                                private_events.push(
                                    json!({"type": "options_hidden", "data": {"indexes": hidden}})
                                        .to_string(),
                                );
                                public_events.push(
                                    json!({
                                        "type": "cheat_used",
                                        "data": {"player": name, "power": "hint"}
                                    })
                                    .to_string(),
                                );
                                true
                            }
                        }
                    };

                    if !applied {
                        if let Some(player) = room.players.get_mut(&sid) {
                            player.cheat.refund(power);
                        }
                        public_events.clear();
                        private_events.clear();
                        victim_events.clear();
                        players_changed = false;
                        send_json(
                            &out_tx,
                            json!({"type": "error", "data": {
                                "message": cheat_no_effect_message(power),
                                "code": "cheat_no_effect"
                            }}),
                        );
                    }

                    room.touch();
                    budget = room
                        .players
                        .get(&sid)
                        .map(|player| player.cheat.to_json())
                        .unwrap_or(serde_json::Value::Null);
                }

                for event in public_events {
                    let _ = room.read().tx.send(event);
                }
                for event in private_events {
                    let _ = out_tx.send(event);
                }
                for (victim_sid, event) in victim_events {
                    if let Some(outbox) = room
                        .read()
                        .players
                        .get(&victim_sid)
                        .and_then(|player| player.outbox.clone())
                    {
                        let _ = outbox.send(event);
                    }
                }
                send_json(
                    &out_tx,
                    json!({"type": "cheat_state", "data": {"powers": budget}}),
                );
                if players_changed {
                    let event = players_event(&room.read());
                    let _ = room.read().tx.send(event);
                }
            }

            "submit_answer" => {
                let Some(room) = room_arc.clone() else {
                    continue;
                };
                let index = message["data"]["answer_index"].as_u64().unwrap_or(0) as usize;

                let (advance, token, players, notice, eliminated, blind_left) = {
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
                    let mode = room.settings.mode;
                    let speed = room.speed_ratio();

                    let mut knocked_out = false;
                    let mut eliminated = false;
                    let mut blind_left: Option<u64> = None;
                    if let Some(player) = room.players.get_mut(&sid) {
                        eliminated = player.is_spectator;
                        if !eliminated && player.is_blind() {
                            blind_left = player.blind_until.map(|until| {
                                until
                                    .saturating_duration_since(Instant::now())
                                    .as_secs()
                                    .max(1)
                            });
                        } else if !eliminated && !player.answered {
                            player.answered = true;
                            player.answer_index = Some(index);
                            player.answer_time = Some(Instant::now());
                            player.award(index == correct, mode, speed);
                            metrics.record_answer(index == correct);
                            if mode.lives().is_some() && player.lives == 0 {
                                player.is_spectator = true;
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
                        blind_left,
                    )
                };

                if let Some(seconds) = blind_left {
                    send_json(
                        &out_tx,
                        json!({"type": "error", "data": {
                            "message": format!("Вас ослепили: ответ через {seconds} с"),
                            "code": "blinded"
                        }}),
                    );
                    continue;
                }

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

            "propose" => {
                let Some(room) = room_arc.clone() else {
                    send_json(
                        &out_tx,
                        json!({"type": "error", "data": {"message": "Сначала войдите в комнату"}}),
                    );
                    continue;
                };

                let proposal_type = match message["data"]["proposal_type"].as_str() {
                    Some("topic") => ProposalType::Topic,
                    Some("mode") => ProposalType::Mode,
                    Some("difficulty") => ProposalType::Difficulty,
                    Some("question_count") => ProposalType::QuestionCount,
                    Some("num_options") => ProposalType::NumOptions,
                    Some("timer_secs") => ProposalType::TimerSecs,
                    Some("ai_provider") => ProposalType::AiProvider,
                    _ => {
                        send_json(
                            &out_tx,
                            json!({"type": "error", "data": {
                                "message": "Неизвестный тип предложения",
                                "code": "proposal_bad_type"
                            }}),
                        );
                        continue;
                    }
                };
                let Some(value) = message["data"]["value"].as_str() else {
                    send_json(
                        &out_tx,
                        json!({"type": "error", "data": {
                            "message": "Укажите значение",
                            "code": "proposal_no_value"
                        }}),
                    );
                    continue;
                };
                // Validate here, not on accept: the host should see only
                // proposals that can actually be applied.
                if let Some(reason) = proposal_rejection(&state, proposal_type.clone(), value) {
                    send_json(
                        &out_tx,
                        json!({"type": "error", "data": {"message": reason, "code": "proposal_invalid"}}),
                    );
                    continue;
                }

                let description = message["data"]["description"]
                    .as_str()
                    .unwrap_or_default()
                    .chars()
                    .take(120)
                    .collect::<String>();

                let event = {
                    let mut room = room.write();
                    let proposer_name = room
                        .players
                        .get(&sid)
                        .map(|player| player.display_name.clone());
                    match (room.state, proposer_name) {
                        (RoomState::Waiting, Some(name)) => {
                            room.create_proposal(
                                sid.clone(),
                                name,
                                proposal_type,
                                value.to_string(),
                                description,
                            );
                            Some(proposals_event(&room))
                        }
                        _ => None,
                    }
                };
                match event {
                    Some(payload) => {
                        room.write().touch();
                        let _ = room
                            .read()
                            .tx
                            .send(json!({"type": "proposals_list", "data": payload}).to_string());
                    }
                    None => send_json(
                        &out_tx,
                        json!({"type": "error", "data": {
                            "message": "Предложения можно делать только в лобби",
                            "code": "proposal_not_in_lobby"
                        }}),
                    ),
                }
            }

            "vote_proposal" | "unvote_proposal" => {
                let Some(room) = room_arc.clone() else {
                    send_json(
                        &out_tx,
                        json!({"type": "error", "data": {"message": "Сначала войдите в комнату"}}),
                    );
                    continue;
                };
                let Some(proposal_id) = message["data"]["proposal_id"].as_str() else {
                    send_json(
                        &out_tx,
                        json!({"type": "error", "data": {
                            "message": "Укажите ID предложения",
                            "code": "proposal_no_id"
                        }}),
                    );
                    continue;
                };
                let voting = message["type"] == "vote_proposal";

                let outcome = {
                    let mut room = room.write();
                    if room.state != RoomState::Waiting {
                        Err("Голосовать можно только в лобби")
                    } else {
                        match room.get_proposal_mut(proposal_id) {
                            Some(proposal) => {
                                let changed = if voting {
                                    proposal.vote(&sid)
                                } else {
                                    // The proposer's own vote stays: a proposal
                                    // without it would look like nobody wants it.
                                    proposal.proposer_sid != sid && proposal.unvote(&sid)
                                };
                                Ok(changed.then(|| proposals_event(&room)))
                            }
                            None => Err("Предложение не найдено"),
                        }
                    }
                };

                match outcome {
                    Ok(Some(payload)) => {
                        let _ = room
                            .read()
                            .tx
                            .send(json!({"type": "proposals_list", "data": payload}).to_string());
                    }
                    Ok(None) => {}
                    Err(message_text) => send_json(
                        &out_tx,
                        json!({"type": "error", "data": {
                            "message": message_text,
                            "code": "proposal_vote_failed"
                        }}),
                    ),
                }
            }

            "accept_proposal" => {
                let Some(room) = room_arc.clone() else {
                    send_json(
                        &out_tx,
                        json!({"type": "error", "data": {"message": "Сначала войдите в комнату"}}),
                    );
                    continue;
                };
                let Some(proposal_id) = message["data"]["proposal_id"].as_str() else {
                    send_json(
                        &out_tx,
                        json!({"type": "error", "data": {
                            "message": "Укажите ID предложения",
                            "code": "proposal_no_id"
                        }}),
                    );
                    continue;
                };

                let outcome = {
                    let mut room = room.write();
                    if room.host_user_id != auth.user_id {
                        Err(("Только хост может принимать предложения", "not_host"))
                    } else if room.state != RoomState::Waiting {
                        Err((
                            "Принимать предложения можно только в лобби",
                            "proposal_not_in_lobby",
                        ))
                    } else {
                        match room.get_proposal(proposal_id).cloned() {
                            Some(proposal) => {
                                let change = apply_proposal(&mut room, &proposal);
                                if let Some(stored) = room.get_proposal_mut(proposal_id) {
                                    stored.active = false;
                                }
                                Ok((change, proposals_event(&room)))
                            }
                            None => Err(("Предложение не найдено", "proposal_not_found")),
                        }
                    }
                };

                match outcome {
                    Ok((change, payload)) => {
                        let event = json!({
                            "type": "proposal_accepted",
                            "data": {
                                "proposal_id": proposal_id,
                                "change": change,
                            }
                        });
                        let _ = room.read().tx.send(event.to_string());
                        let _ = room
                            .read()
                            .tx
                            .send(json!({"type": "proposals_list", "data": payload}).to_string());
                    }
                    Err((message_text, code)) => send_json(
                        &out_tx,
                        json!({"type": "error", "data": {"message": message_text, "code": code}}),
                    ),
                }
            }

            "get_proposals" => {
                let Some(room) = room_arc.clone() else {
                    continue;
                };
                let payload = {
                    let room = room.read();
                    proposals_event(&room)
                };
                send_json(&out_tx, json!({"type": "proposals_list", "data": payload}));
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
    metrics.ws_connections_active.dec();
    info!(user_id = auth.user_id, "WebSocket disconnected");
}

fn send_json(out: &mpsc::UnboundedSender<String>, value: Value) {
    let _ = out.send(value.to_string());
}

/// Why a cheat power changed nothing. The use is refunded either way; the
/// message explains what the player should have done differently.
fn cheat_no_effect_message(power: CheatPower) -> &'static str {
    match power {
        CheatPower::Double => "Удвоение уже заряжено",
        CheatPower::Steal => "Брать нечего: у соперников нет очков",
        CheatPower::Swap => "Вы уже лидируете — меняться не с кем",
        CheatPower::Leech => "Ни у кого нет очков",
        CheatPower::Mute => "Игрок уже не может писать",
        CheatPower::Jinx => "Игрок уже проклят",
        CheatPower::Blind => "Игрок уже ослеплён",
        CheatPower::Hijack => "Взять этого игрока не удалось",
        CheatPower::Rewrite => "Вопрос не заменён",
        CheatPower::Freeze | CheatPower::Peek | CheatPower::Hint => "Сейчас это не сработает",
    }
}

fn send_chat_error(out: &mpsc::UnboundedSender<String>, error: ChatError) {
    send_json(
        out,
        json!({"type": "error", "data": {"message": error.message(), "code": error.code()}}),
    );
}

/// Name and mute flag of the socket's player; `None` when the socket has not
/// joined a room (or was removed from it).
fn chat_sender(room: &Arc<RwLock<Room>>, sid: &str) -> Option<(String, bool, bool)> {
    let room = room.read();
    room.players
        .get(sid)
        .map(|player| (player.display_name.clone(), player.muted, room.chat_locked))
}

/// A question a cheater typed in, already validated.
struct RewriteInput {
    text: String,
    options: Vec<String>,
    correct: usize,
}

/// Validate the `payload` of a `rewrite` cheat. Text goes through the chat
/// sanitiser, so the same length and control-character rules apply.
fn parse_rewrite(payload: &Value) -> Option<RewriteInput> {
    let text = payload["question"].as_str().and_then(chat::sanitize)?;
    if text.chars().count() < 3 || text.chars().count() > MAX_CHAT_CHARS {
        return None;
    }

    let raw_options = payload["options"].as_array()?;
    if !(2..=6).contains(&raw_options.len()) {
        return None;
    }
    let mut options = Vec::with_capacity(raw_options.len());
    for option in raw_options {
        let clean = option.as_str().and_then(chat::sanitize)?;
        if clean.chars().count() > 80 {
            return None;
        }
        options.push(clean);
    }

    let correct = payload["correct_index"].as_u64()? as usize;
    if correct >= options.len() {
        return None;
    }

    Some(RewriteInput {
        text,
        options,
        correct,
    })
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
                "team": player.team,
                "eliminated": player.is_spectator,
            })
        })
        .collect();
    let mut data = json!({
        "scores": scores,
        "mode": room.settings.mode.as_str(),
        "persisted": room.settings.mode.persists(),
    });
    if let Some(teams) = team_standings(room) {
        data["teams"] = teams;
    }
    json!({"type": "game_over", "data": data}).to_string()
}

/// Team totals for team mode, ordered by score; `None` for other modes.
fn team_standings(room: &Room) -> Option<Value> {
    if !room.settings.mode.is_team() {
        return None;
    }
    let mut totals = [0_i64; TEAM_NAMES.len()];
    for player in room.players.values() {
        if let Some(team) = player.team {
            if let Some(total) = totals.get_mut(team as usize) {
                *total += player.score;
            }
        }
    }
    if room.settings.mode.is_coop() {
        return Some(Value::Array(vec![json!({
            "team": 0,
            "name": "Все вместе",
            "score": totals[0]
        })]));
    }
    let teams: Vec<Value> = TEAM_NAMES
        .iter()
        .enumerate()
        .map(|(index, name)| json!({"team": index, "name": name, "score": totals[index]}))
        .collect();
    Some(Value::Array(teams))
}

/// The proposal list as every client sees it: ids, votes and freshness.
fn proposals_event(room: &Room) -> Value {
    let proposals = room
        .proposals
        .iter()
        .map(ProposalInfo::from)
        .collect::<Vec<_>>();
    json!({ "proposals": proposals })
}

/// Reject a proposal at creation time when accepting it could not work, so
/// the lobby never shows a proposal the host can only refuse.
fn proposal_rejection(state: &AppState, kind: ProposalType, value: &str) -> Option<String> {
    let range = |min: u64, max: u64| -> Option<String> {
        match value.trim().parse::<u64>() {
            Ok(number) if (min..=max).contains(&number) => None,
            _ => Some(format!("Нужно число от {min} до {max}")),
        }
    };
    match kind {
        ProposalType::Topic => {
            let length = value.trim().chars().count();
            (!(1..=60).contains(&length)).then(|| "Тема: от 1 до 60 символов".to_string())
        }
        ProposalType::Mode => GameMode::parse(value).is_none().then(|| {
            format!(
                "Неизвестный режим «{value}». Доступны: {}",
                GameMode::ALL
                    .iter()
                    .map(|mode| mode.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }),
        ProposalType::Difficulty => (!matches!(value, "easy" | "medium" | "hard"))
            .then(|| "Сложность: easy, medium или hard".to_string()),
        ProposalType::QuestionCount => range(5, 30),
        ProposalType::NumOptions => range(2, 6),
        ProposalType::TimerSecs => range(10, 120),
        ProposalType::AiProvider => {
            let available = state.ai_router.available_providers();
            (!available.iter().any(|name| name == value)).then(|| {
                if available.is_empty() {
                    "ИИ-провайдеры не настроены: задайте ключ в окружении".to_string()
                } else {
                    format!("Провайдер «{value}» недоступен: {}", available.join(", "))
                }
            })
        }
    }
}

/// Apply an accepted proposal and describe the change for the broadcast.
fn apply_proposal(room: &mut Room, proposal: &Proposal) -> Value {
    let value = proposal.value.trim();
    match proposal.proposal_type {
        ProposalType::Topic => {
            room.settings.topic = value.to_string();
            json!({ "setting": "topic", "value": room.settings.topic })
        }
        ProposalType::Mode => {
            if let Some(mode) = GameMode::parse(value) {
                room.settings.mode = mode;
            }
            json!({ "setting": "mode", "value": room.settings.mode.as_str() })
        }
        ProposalType::Difficulty => {
            room.settings.difficulty = value.to_string();
            json!({ "setting": "difficulty", "value": room.settings.difficulty })
        }
        ProposalType::QuestionCount => {
            if let Ok(count) = value.parse::<usize>() {
                room.settings.question_count = count.clamp(5, 30);
            }
            json!({ "setting": "question_count", "value": room.settings.question_count })
        }
        ProposalType::NumOptions => {
            if let Ok(options) = value.parse::<usize>() {
                room.settings.num_options = options.clamp(2, 6);
            }
            json!({ "setting": "num_options", "value": room.settings.num_options })
        }
        ProposalType::TimerSecs => {
            if let Ok(secs) = value.parse::<u64>() {
                room.settings.timer_secs = secs.clamp(10, 120);
            }
            json!({ "setting": "timer_secs", "value": room.settings.timer_secs })
        }
        ProposalType::AiProvider => {
            room.settings.ai_provider = Some(value.to_string());
            json!({ "setting": "ai_provider", "value": room.settings.ai_provider })
        }
    }
}

pub(crate) fn players_event(room: &Room) -> String {
    let players: Vec<Value> = room
        .players
        .values()
        .map(|player| {
            json!({
                "id": player.sid,
                "name": player.display_name,
                "score": player.score,
                "lives": player.lives,
                "infinite_lives": player.infinite_lives,
                "team": player.team,
                "streak": player.streak,
                "eliminated": player.is_spectator,
                "disconnected": player.is_disconnected(),
                "is_host": player.user_id == Some(room.host_user_id),
                "muted": player.muted,
                "hijacked": player.hijacked,
                "is_ready": true,
            })
        })
        .collect();
    json!({"type": "players_updated", "data": {"players": players}}).to_string()
}

fn question_event(room: &Room) -> Option<String> {
    let question = room.questions.get(room.current_q)?;
    let time_limit = room.settings.timer_secs;

    // Calculate remaining time based on server-side deadline (anti-cheat).
    let time_remaining = room
        .deadline
        .map(|deadline| {
            let now = std::time::Instant::now();
            if deadline > now {
                deadline.duration_since(now).as_secs()
            } else {
                0
            }
        })
        .unwrap_or(time_limit);

    Some(
        json!({
            "type": "question_emitted",
            "data": {
                "question": {
                    "question": question["question"],
                    "options": question["options"],
                    "time_limit": time_limit,
                    "time_remaining": time_remaining,
                },
                "time_limit": time_limit,
                "time_remaining": time_remaining,
                "index": room.current_q,
                "total": room.questions.len(),
                "source": room.question_source,
                "mode": room.settings.mode.as_str(),
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
        let out_of_lives = room.settings.mode.lives().is_some()
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
                mode: room.settings.mode.as_str().to_string(),
                players: room.players.len() as i32,
                duration_secs: duration,
            };
            // Sandbox is a practice room: it must not move the leaderboard.
            finished = room.settings.mode.persists().then_some((results, info));
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
fn fit_count(mut questions: Vec<Value>, count: usize, topic: &str) -> Vec<Value> {
    let count = count.max(1);
    questions.truncate(count);

    // Providers sometimes return fewer questions than asked. Modes like the
    // marathon promise a fixed length, so top the set up from the offline bank
    // instead of silently playing a shorter game.
    if questions.len() < count {
        let seen: HashSet<String> = questions
            .iter()
            .filter_map(|question| question["question"].as_str().map(str::to_owned))
            .collect();
        let missing = count - questions.len();
        let padding: Vec<Value> = fallback_questions(topic, count)
            .into_iter()
            .filter_map(|question| serde_json::to_value(question).ok())
            .filter(|question| {
                question["question"]
                    .as_str()
                    .map(|text| !seen.contains(text))
                    .unwrap_or(false)
            })
            .take(missing)
            .collect();
        questions.extend(padding);
    }

    // Last resort when the bank has no new material left: repeat the set.
    let base = questions.len();
    let mut index = 0;
    while base > 0 && questions.len() < count {
        let extra = questions[index % base].clone();
        questions.push(extra);
        index += 1;
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
        assert_eq!(fit_count(questions(10), 5, "тема").len(), 5);
    }

    #[test]
    fn a_short_set_is_padded_up_to_the_requested_count() {
        let padded = fit_count(questions(3), 5, "тема");
        assert_eq!(padded.len(), 5);
        assert_eq!(padded[0]["question"].as_str().unwrap(), "q0");
        for question in &padded[3..] {
            assert!(question["correct_index"].is_number(), "{question}");
        }
    }

    #[test]
    fn an_empty_set_is_filled_from_the_offline_bank() {
        let filled = fit_count(Vec::new(), 4, "тема");
        assert_eq!(filled.len(), 4);
        assert!(filled.iter().all(|q| q["question"].is_string()));
    }

    #[test]
    fn trimming_keeps_the_first_questions() {
        let trimmed = fit_count(questions(4), 2, "тема");
        let first = trimmed[0]["question"].as_str().unwrap();
        assert_eq!(first, "q0");
    }
}
