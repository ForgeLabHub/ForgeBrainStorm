//! Room lifecycle HTTP routes.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::middleware;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use serde::Deserialize;
use validator::Validate;

use crate::domain::room::RoomSettings;
use crate::error::{AppError, AppResult, ValidationError};
use crate::middleware::auth::{AuthContext, SessionCache};
use crate::AppState;

pub fn routes() -> Router<AppState> {
    // Public routes — no auth required.
    let public = Router::new()
        .route("/", get(list_rooms))
        .route("/:code", get(get_room));

    // Protected routes — require auth.
    let protected = Router::new()
        .route("/", post(create_room))
        .route_layer(middleware::from_fn(crate::middleware::auth::auth_middleware));

    Router::new().merge(public).merge(protected)
}

/// List public rooms (no auth).
async fn list_rooms(
    State(state): State<AppState>,
) -> AppResult<impl IntoResponse> {
    let rooms = state.rooms.list_public();
    Ok(Json(serde_json::json!({ "ok": true, "rooms": rooms })))
}

#[derive(Debug, Deserialize, Validate)]
struct CreateRoomInput {
    #[validate(length(min = 1, max = 60, message = "Тема: от 1 до 60 символов"))]
    topic: Option<String>,
    mode: Option<String>,
    max_players: Option<usize>,
}

/// Create a room — requires authentication.
///
/// `AuthContext` is injected by `auth_middleware`. The middleware is attached
/// to the protected sub-router above.
async fn create_room(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
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

/// Get a specific room by code (no auth).
async fn get_room(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> AppResult<impl IntoResponse> {
    let room = state
        .rooms
        .get(&code.to_uppercase())
        .ok_or(AppError::NotFound("Комната не найдена"))?;
    Ok(Json(serde_json::json!({ "ok": true, "room": room })))
}
