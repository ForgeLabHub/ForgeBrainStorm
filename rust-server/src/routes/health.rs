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

/// Readiness probe — verifies the database and Redis are reachable.
async fn ready(State(state): State<AppState>) -> AppResult<impl IntoResponse> {
    state.db.health_check().await?;

    let redis_status = if state.redis.is_connected().await {
        "ok"
    } else {
        "local_only"
    };

    let pubsub_subscriptions = state.pubsub.subscription_count().await;
    let instance_id = state.pubsub.instance_id().to_string();

    Ok((
        StatusCode::OK,
        Json(json!({
            "status": "ready",
            "db": "ok",
            "redis": redis_status,
            "pubsub": {
                "instance_id": instance_id,
                "subscriptions": pubsub_subscriptions,
            },
        })),
    ))
}
