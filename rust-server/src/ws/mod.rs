//! WebSocket handler for real-time gameplay.

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

use crate::error::AppResult;
use crate::middleware::auth::AuthContext;
use crate::AppState;

/// Upgrade HTTP to WebSocket.
/// Auth is required — extracted via `Extension<AuthContext>` (set by auth middleware).
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(_state): State<AppState>,
    axum::Extension(auth): axum::Extension<AuthContext>,
) -> impl IntoResponse {
    info!(user_id = auth.user_id, "WebSocket upgrade requested");
    ws.on_upgrade(move |socket| handle_socket(socket, _state, auth))
}

async fn handle_socket(socket: WebSocket, state: AppState, auth: AuthContext) {
    info!(user_id = auth.user_id, "WebSocket connected");
    let (mut writer, mut reader) = socket.split();

    // TODO: full protocol implementation.
    // 1. Read client messages (join_room, submit_answer, chat, etc.)
    // 2. Subscribe to room's broadcast channel
    // 3. Forward broadcast events to writer
    // 4. Handle disconnect with grace period

    while let Some(msg) = reader.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                debug!(user_id = auth.user_id, "received: {}", text);
                // TODO: parse and dispatch
            }
            Ok(Message::Close(_)) | Err(_) => break,
            _ => {}
        }
    }
    info!(user_id = auth.user_id, "WebSocket disconnected");
}

/// Client → server messages.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ClientMessage {
    JoinRoom { code: String, name: String },
    CreateRoom { topic: Option<String>, mode: Option<String> },
    StartGame,
    SubmitAnswer { answer_index: usize },
    ChatMessage { text: String },
    UseJoker,
    GetHint,
}

/// Server → client messages.
#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum ServerMessage {
    RoomJoined { code: String, is_host: bool },
    PlayersUpdated { players: Vec<serde_json::Value> },
    QuestionEmitted { question: serde_json::Value, time_limit: u64 },
    QuestionResult { correct_index: usize, explanation: String },
    ChatMessage { name: String, text: String, ts: i64 },
    GameOver { scores: serde_json::Value },
    Error { message: String },
}
