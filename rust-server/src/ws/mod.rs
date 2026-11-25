//! WebSocket handler for real-time gameplay.
//!
//! Implements the full game protocol: room management, question generation,
//! answer submission, scoring, timers, and broadcast events.

pub mod protocol;
pub mod game_loop;
pub mod room_manager;

use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use futures::{SinkExt, StreamExt};
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

use crate::error::AppResult;
use crate::middleware::auth::AuthContext;
use crate::AppState;

use self::protocol::{ClientMessage, ServerMessage};
use self::room_manager::RoomManager;

/// Upgrade HTTP to WebSocket.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    axum::Extension(auth): axum::Extension<AuthContext>,
) -> impl IntoResponse {
    info!(user_id = auth.user_id, "WebSocket upgrade requested");
    ws.on_upgrade(move |socket| handle_socket(socket, state, auth))
}

/// Main WebSocket connection handler.
async fn handle_socket(socket: WebSocket, state: AppState, auth: AuthContext) {
    let user_id = auth.user_id;
    let username = auth.username.clone();

    info!(user_id, username = %username, "WebSocket connected");

    let (mut writer, mut reader) = socket.split();
    let room_manager = Arc::new(RoomManager::new(state.clone()));

    // Create a channel for sending messages to this client
    let (tx, mut rx) = tokio::sync::mpsc::channel::<ServerMessage>(100);

    // Spawn task to forward messages from rx to WebSocket writer
    let writer_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&msg) {
                if writer.send(Message::Text(json)).await.is_err() {
                    break;
                }
            }
        }
    });

    // Track current room for cleanup and broadcast subscription
    let mut current_room_code: Option<String> = None;
    let mut broadcast_task: Option<tokio::task::JoinHandle<()>> = None;

    // Main message loop
    while let Some(msg) = reader.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                debug!(user_id, "Received: {}", text);

                match serde_json::from_str::<ClientMessage>(&text) {
                    Ok(client_msg) => {
                        let result = handle_client_message(
                            client_msg,
                            &state,
                            &room_manager,
                            &tx,
                            user_id,
                            &username,
                            &mut current_room_code,
                            &mut broadcast_task,
                        )
                        .await;

                        if let Err(e) = result {
                            warn!(user_id, error = %e, "Error handling message");
                        }
                    }
                    Err(e) => {
                        warn!(user_id, error = %e, "Failed to parse client message");
                        let _ = tx
                            .send(ServerMessage::Error {
                                message: format!("Invalid message format: {}", e),
                            })
                            .await;
                    }
                }
            }
            Ok(Message::Close(_)) => {
                info!(user_id, "WebSocket closed by client");
                break;
            }
            Err(e) => {
                error!(user_id, error = %e, "WebSocket error");
                break;
            }
            _ => {}
        }
    }

    // Cleanup: leave room if in one
    if let Some(room_code) = current_room_code {
        room_manager.leave_room(&room_code, user_id).await;
    }

    // Abort broadcast forwarding task
    if let Some(task) = broadcast_task {
        task.abort();
    }

    // Abort writer task
    writer_task.abort();

    info!(user_id, username = %username, "WebSocket disconnected");
}

/// Handle a single client message.
async fn handle_client_message(
    msg: ClientMessage,
    state: &AppState,
    room_manager: &Arc<RoomManager>,
    tx: &tokio::sync::mpsc::Sender<ServerMessage>,
    user_id: i64,
    username: &str,
    current_room_code: &mut Option<String>,
    broadcast_task: &mut Option<tokio::task::JoinHandle<()>>,
) -> Result<(), String> {
    match msg {
        ClientMessage::CreateRoom {
            topic,
            mode,
            settings,
        } => {
            match room_manager
                .create_room(user_id, username, topic, mode, settings)
                .await
            {
                Ok(room_arc) => {
                    let room = room_arc.read();
                    let code = room.code.clone();
                    let host_name = room.host_name.clone();
                    let topic = room.settings.topic.clone();
                    let mode = room.settings.mode.clone();
                    drop(room);

                    *current_room_code = Some(code.clone());

                    // Subscribe to broadcast
                    subscribe_to_broadcast(room_arc, tx.clone(), broadcast_task);

                    let _ = tx
                        .send(ServerMessage::RoomCreated {
                            code,
                            host: host_name,
                            topic,
                            mode,
                        })
                        .await;
                }
                Err(e) => {
                    let _ = tx
                        .send(ServerMessage::Error {
                            message: format!("Failed to create room: {}", e),
                        })
                        .await;
                }
            }
        }

        ClientMessage::JoinRoom { code, name } => {
            let display_name = if name.is_empty() {
                username.to_string()
            } else {
                name
            };

            match room_manager
                .join_room(&code, user_id, username, &display_name)
                .await
            {
                Ok(room_arc) => {
                    *current_room_code = Some(code.clone());

                    // Subscribe to broadcast
                    subscribe_to_broadcast(room_arc.clone(), tx.clone(), broadcast_task);

                    let room = room_arc.read();
                    let is_host = room.host_user_id == user_id;
                    let topic = room.settings.topic.clone();
                    let mode = room.settings.mode.clone();

                    // Build current player list
                    let players: Vec<super::protocol::PlayerInfo> = room
                        .players
                        .values()
                        .map(|p| super::protocol::PlayerInfo {
                            user_id: p.user_id,
                            username: p.username.clone(),
                            display_name: p.display_name.clone(),
                            score: p.score,
                            lives: p.lives,
                            is_ready: p.is_ready,
                            is_host: p.user_id == room.host_user_id,
                        })
                        .collect();

                    drop(room);

                    let _ = tx
                        .send(ServerMessage::RoomJoined {
                            code,
                            is_host,
                            topic,
                            mode,
                            players,
                        })
                        .await;
                }
                Err(e) => {
                    let _ = tx
                        .send(ServerMessage::Error {
                            message: format!("Failed to join room: {}", e),
                        })
                        .await;
                }
            }
        }

        ClientMessage::LeaveRoom => {
            if let Some(room_code) = current_room_code.take() {
                // Stop broadcast subscription
                if let Some(task) = broadcast_task.take() {
                    task.abort();
                }

                room_manager.leave_room(&room_code, user_id).await;
                let _ = tx.send(ServerMessage::RoomLeft {}).await;
            }
        }

        ClientMessage::StartGame => {
            if let Some(room_code) = current_room_code.as_ref() {
                match room_manager.start_game(room_code, user_id).await {
                    Ok(_) => {
                        // GameStarted is broadcast via the room channel,
                        // which is already forwarded to the client.
                    }
                    Err(e) => {
                        let _ = tx
                            .send(ServerMessage::Error {
                                message: format!("Failed to start game: {}", e),
                            })
                            .await;
                    }
                }
            }
        }

        ClientMessage::SubmitAnswer {
            question_index,
            answer_index,
        } => {
            if let Some(room_code) = current_room_code.as_ref() {
                room_manager
                    .submit_answer(room_code, user_id, question_index, answer_index)
                    .await;
            }
        }

        ClientMessage::ChatMessage { text } => {
            if let Some(room_code) = current_room_code.as_ref() {
                room_manager
                    .send_chat(room_code, user_id, username, &text)
                    .await;
            }
        }

        ClientMessage::UseJoker => {
            if let Some(room_code) = current_room_code.as_ref() {
                match room_manager.use_joker(room_code, user_id).await {
                    Ok(removed_indices) => {
                        let _ = tx
                            .send(ServerMessage::JokerUsed { removed_indices })
                            .await;
                    }
                    Err(e) => {
                        let _ = tx
                            .send(ServerMessage::Error {
                                message: format!("Failed to use joker: {}", e),
                            })
                            .await;
                    }
                }
            }
        }

        ClientMessage::GetHint => {
            if let Some(room_code) = current_room_code.as_ref() {
                match room_manager.get_hint(room_code, user_id).await {
                    Ok(hint) => {
                        let _ = tx.send(ServerMessage::HintReceived { hint }).await;
                    }
                    Err(e) => {
                        let _ = tx
                            .send(ServerMessage::Error {
                                message: format!("Failed to get hint: {}", e),
                            })
                            .await;
                    }
                }
            }
        }
    }

    Ok(())
}

/// Subscribe to a room's broadcast channel and forward events to the client.
fn subscribe_to_broadcast(
    room_arc: Arc<parking_lot::RwLock<crate::domain::room::Room>>,
    tx: tokio::sync::mpsc::Sender<ServerMessage>,
    broadcast_task: &mut Option<tokio::task::JoinHandle<()>>,
) {
    // Abort existing broadcast task if any
    if let Some(task) = broadcast_task.take() {
        task.abort();
    }

    let mut broadcast_rx = room_arc.read().tx.subscribe();

    let task = tokio::spawn(async move {
        loop {
            match broadcast_rx.recv().await {
                Ok(event) => {
                    if tx.send(event).await.is_err() {
                        break; // Client disconnected
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    warn!(lagged = n, "Broadcast receiver lagged, skipping messages");
                    continue;
                }
                Err(broadcast::error::RecvError::Closed) => {
                    break; // Channel closed
                }
            }
        }
    });

    *broadcast_task = Some(task);
}
