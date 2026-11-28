//! Room lifecycle HTTP routes.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::middleware;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use serde::Deserialize;
use validator::Validate;

use crate::domain::mode::GameMode;
use crate::domain::room::RoomSettings;
use crate::error::{AppError, AppResult, ValidationError};
use crate::middleware::auth::{auth_middleware, AuthContext};
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_rooms))
        .route(
            "/",
            post(create_room).layer(middleware::from_fn(auth_middleware)),
        )
        .route("/:code", get(get_room))
}

async fn list_rooms(State(state): State<AppState>) -> AppResult<impl IntoResponse> {
    let rooms = state.rooms.list_public();
    Ok(Json(serde_json::json!({"ok": true, "rooms": rooms})))
}

#[derive(Debug, Deserialize, Validate)]
struct CreateRoomInput {
    #[validate(length(min = 1, max = 60, message = "Тема: от 1 до 60 символов"))]
    topic: Option<String>,
    mode: Option<String>,
    difficulty: Option<String>,
    max_players: Option<usize>,
    question_count: Option<usize>,
    num_options: Option<usize>,
    #[serde(alias = "timer_seconds")]
    timer_secs: Option<u64>,


    ai_provider: Option<String>,

}

async fn create_room(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Json(input): Json<CreateRoomInput>,
) -> AppResult<impl IntoResponse> {
    input
        .validate()
        .map_err(|e| AppError::Validation(ValidationError::new("topic", e.to_string())))?;

    let mode = match input.mode.as_deref() {
        None => GameMode::Classic,
        Some(raw) => GameMode::parse(raw).ok_or_else(|| {
            AppError::Validation(ValidationError::new(
                "mode",
                format!(
                    "Неизвестный режим «{raw}». Доступны: {}",
                    GameMode::ALL
                        .iter()
                        .map(|mode| mode.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            ))
        })?,
    };

    let mut settings = RoomSettings {
        topic: input.topic.unwrap_or_else(|| "Общие знания".into()),
        mode,
        max_players: input.max_players.unwrap_or(state.settings.max_players),
        ..Default::default()
    };
    if let Some(difficulty) = input.difficulty {
        settings.difficulty = difficulty;
    }
    if let Some(count) = input.question_count {
        settings.question_count = count;
    }
    if let Some(options) = input.num_options {
        settings.num_options = options;
    }
    if let Some(secs) = input.timer_secs {
        settings.timer_secs = secs;
    }


    if let Some(ai_provider) = input.ai_provider {
        // Validate that the provider exists
        let available = state.ai_router.available_providers();
        if !available.contains(&ai_provider) {
            return Err(AppError::Validation(ValidationError::new(
                "ai_provider",
                format!(
                    "Провайдер «{ai_provider}» недоступен. Доступны: {}",
                    available.join(", ")
                ),
            )));
        }
        settings.ai_provider = Some(ai_provider);
    }


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
