//! Health and readiness probes.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;

use crate::error::AppResult;
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/readyz", get(ready))
}

/// Liveness probe — always returns 200 if the process is up.
async fn health() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

/// Readiness probe — verifies the database is reachable.
async fn ready(State(state): State<AppState>) -> AppResult<impl IntoResponse> {
    state.db.health_check().await?;
    Ok((
        StatusCode::OK,
        Json(json!({
            "status": "ready",
            "db": "ok",
        })),
    ))
}
