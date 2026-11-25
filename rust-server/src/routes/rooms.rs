//! Room lifecycle HTTP routes.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use validator::Validate;

use crate::domain::room::{RoomRegistry, RoomSettings};
use crate::error::{AppError, AppResult, ValidationError};
use crate::middleware::auth::AuthContext;
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_rooms))
        .route("/", post(create_room))
        .route("/{code}", get(get_room))
}

async fn list_rooms(
    State(state): State<AppState>,
) -> AppResult<impl IntoResponse> {
    let rooms = state.rooms.list_public();
    Ok(Json(serde_json::json!({"ok": true, "rooms": rooms})))
}

#[derive(Debug, Deserialize, Validate)]
struct CreateRoomInput {
    #[validate(length(min = 1, max = 60, message = "Тема: от 1 до 60 символов"))]
    topic: Option<String>,
    mode: Option<String>,
    max_players: Option<usize>,
}

async fn create_room(
    State(state): State<AppState>,
    auth: AuthContext,
    Json(input): Json<CreateRoomInput>,
) -> AppResult<impl IntoResponse> {
    input.validate().map_err(|e| {
        AppError::Validation(ValidationError::new("topic", e.to_string()))
    })?;

    let settings = RoomSettings {
        topic: input.topic.unwrap_or_else(|| "Общие знания".into()),
        mode: input.mode.unwrap_or_else(|| "classic".into()),
        max_players: input.max_players.unwrap_or(state.settings.max_players),
        ..Default::default()
    };

    let room = state.rooms.create(auth.user_id, &auth.username, settings)?;
    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "ok": true,
            "room": {
                "code": room.code,
                "host": room.host_name,
                "mode": room.mode,
                "topic": room.topic,
            }
        })),
    ))
}

async fn get_room(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> AppResult<impl IntoResponse> {
    let room = state
        .rooms
        .get(&code.to_uppercase())
        .ok_or(AppError::NotFound("Комната не найдена"))?;
    Ok(Json(serde_json::json!({"ok": true, "room": room})))
}
