//! Application error types.
//!
//! Every error type maps to an HTTP status via [`AppError::into_response`].
//! Domain errors never leak internal details — the client sees only a
//! stable machine-readable code and a localised message.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use tracing::error;

/// Top-level error for the whole application.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("validation error: {0}")]
    Validation(#[from] ValidationError),

    #[error("authentication required")]
    Unauthorized,

    #[error("forbidden: {0}")]
    Forbidden(&'static str),

    #[error("resource not found: {0}")]
    NotFound(&'static str),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("rate limit exceeded")]
    RateLimited,

    #[error("room is not in the correct state for this action")]
    InvalidState,

    #[error("AI backend unavailable: {0}")]
    AiUnavailable(String),

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("configuration error: {0}")]
    Config(#[from] crate::config::ConfigError),

    #[error("internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

/// Field-level validation failure.
#[derive(Debug, thiserror::Error)]
#[error("field `{field}`: {message}")]
pub struct ValidationError {
    pub field: &'static str,
    pub message: String,
}

impl ValidationError {
    pub fn new(field: &'static str, message: impl Into<String>) -> Self {
        Self { field, message: message.into() }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            AppError::Validation(e) => (
                StatusCode::BAD_REQUEST,
                "validation_error",
                e.to_string(),
            ),
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                "Требуется авторизация".to_string(),
            ),
            AppError::Forbidden(reason) => (
                StatusCode::FORBIDDEN,
                "forbidden",
                reason.to_string(),
            ),
            AppError::NotFound(what) => (
                StatusCode::NOT_FOUND,
                "not_found",
                format!("Не найдено: {}", what),
            ),
            AppError::Conflict(msg) => (
                StatusCode::CONFLICT,
                "conflict",
                msg.clone(),
            ),
            AppError::RateLimited => (
                StatusCode::TOO_MANY_REQUESTS,
                "rate_limited",
                "Слишком много запросов".to_string(),
            ),
            AppError::InvalidState => (
                StatusCode::CONFLICT,
                "invalid_state",
                "Комната не в нужном состоянии".to_string(),
            ),
            AppError::AiUnavailable(msg) => (
                StatusCode::BAD_GATEWAY,
                "ai_unavailable",
                msg.clone(),
            ),
            AppError::Database(e) => {
                error!(error = %e, "database error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "db_error",
                    "Внутренняя ошибка базы данных".to_string(),
                )
            }
            AppError::Config(e) => {
                error!(error = %e, "configuration error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "config_error",
                    "Ошибка конфигурации".to_string(),
                )
            }
            AppError::Internal(e) => {
                error!(error = %e, "internal error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "Внутренняя ошибка сервера".to_string(),
                )
            }
        };

        let body = json!({
            "ok": false,
            "error": {
                "code": code,
                "message": message,
            },
        });
        (status, axum::Json(body)).into_response()
    }
}

/// Convenience alias for handlers that may fail with [`AppError`].
pub type AppResult<T> = Result<T, AppError>;
