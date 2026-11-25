//! WebSocket handler for real-time gameplay.
//!
//! Implements the full game protocol: room management, question generation,
//! answer submission, scoring, timers, and broadcast events.

pub mod protocol;
pub mod game_loop;
pub mod room_manager;

use std::sync::Arc;
use std::time::Duration;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use futures::{SinkExt, StreamExt};
use serde_json::json;
use tokio::sync::broadcast;
use tokio::time::timeout;
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
    
    // Track current room for cleanup
    let mut current_room_code: Option<String> = None;
    
    // Main message loop
    while let Some(msg) = reader.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                debug!(user_id, "Received: {}", text);
                
                match serde_json::from_str::<ClientMessage>(&text) {
                    Ok(client_msg) => {
                        handle_client_message(
                            client_msg,
                            &state,
                            &room_manager,
                            &tx,
                            user_id,
                            &username,
                            &mut current_room_code,
                        )
                        .await;
                    }
                    Err(e) => {
                        warn!(user_id, error = %e, "Failed to parse client message");
                        let _ = tx.send(ServerMessage::Error {
                            message: format!("Invalid message format: {}", e),
                        }).await;
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
) {
    match msg {
        ClientMessage::CreateRoom { topic, mode, settings } => {
            match room_manager.create_room(user_id, username, topic, mode, settings).await {
                Ok(room) => {
                    *current_room_code = Some(room.code.clone());
                    let _ = tx.send(ServerMessage::RoomCreated {
                        code: room.code,
                        host: room.host_name,
                        topic: room.topic,
                        mode: room.mode,
                    }).await;
                }
                Err(e) => {
                    let _ = tx.send(ServerMessage::Error {
                        message: format!("Failed to create room: {}", e),
                    }).await;
                }
            }
        }
        
        ClientMessage::JoinRoom { code, name } => {
            match room_manager.join_room(&code, user_id, username, &name).await {
                Ok(room) => {
                    *current_room_code = Some(code.clone());
                    
                    // Subscribe to room broadcast channel
                    let mut broadcast_rx = room.tx.subscribe();
                    let tx_clone = tx.clone();
                    
                    // Spawn task to forward broadcast messages
                    tokio::spawn(async move {
                        while let Ok(event) = broadcast_rx.recv().await {
                            let _ = tx_clone.send(event).await;
                        }
                    });
                    
                    let _ = tx.send(ServerMessage::RoomJoined {
                        code: room.code,
                        is_host: room.host_user_id == user_id,
                        topic: room.topic,
                        mode: room.mode,
                        players: room.players_list(),
                    }).await;
                }
                Err(e) => {
                    let _ = tx.send(ServerMessage::Error {
                        message: format!("Failed to join room: {}", e),
                    }).await;
                }
            }
        }
        
        ClientMessage::LeaveRoom => {
            if let Some(room_code) = current_room_code.take() {
                room_manager.leave_room(&room_code, user_id).await;
                let _ = tx.send(ServerMessage::RoomLeft {}).await;
            }
        }
        
        ClientMessage::StartGame => {
            if let Some(room_code) = current_room_code {
                match room_manager.start_game(room_code, user_id).await {
                    Ok(_) => {
                        let _ = tx.send(ServerMessage::GameStarted {}).await;
                    }
                    Err(e) => {
                        let _ = tx.send(ServerMessage::Error {
                            message: format!("Failed to start game: {}", e),
                        }).await;
                    }
                }
            }
        }
        
        ClientMessage::SubmitAnswer { question_index, answer_index } => {
            if let Some(room_code) = current_room_code {
                room_manager.submit_answer(room_code, user_id, question_index, answer_index).await;
            }
        }
        
        ClientMessage::ChatMessage { text } => {
            if let Some(room_code) = current_room_code {
                room_manager.send_chat(room_code, user_id, username, &text).await;
            }
        }
        
        ClientMessage::UseJoker => {
            if let Some(room_code) = current_room_code {
                match room_manager.use_joker(room_code, user_id).await {
                    Ok(joker_result) => {
                        let _ = tx.send(ServerMessage::JokerUsed {
                            removed_indices: joker_result.removed_indices,
                        }).await;
                    }
                    Err(e) => {
                        let _ = tx.send(ServerMessage::Error {
                            message: format!("Failed to use joker: {}", e),
                        }).await;
                    }
                }
            }
        }
        
        ClientMessage::GetHint => {
            if let Some(room_code) = current_room_code {
                match room_manager.get_hint(room_code, user_id).await {
                    Ok(hint) => {
                        let _ = tx.send(ServerMessage::HintReceived { hint }).await;
                    }
                    Err(e) => {
                        let _ = tx.send(ServerMessage::Error {
                            message: format!("Failed to get hint: {}", e),
                        }).await;
                    }
                }
            }
        }
    }
}
