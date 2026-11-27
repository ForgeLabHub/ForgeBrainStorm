//! Health and readiness probes.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;

use crate::error::AppResult;
use crate::metrics;
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .route("/livez", get(livez))
        .route("/readyz", get(ready))
        .route("/metrics", get(metrics_endpoint))
}

/// Liveness probe — always returns 200 if the process is up.
async fn health() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

/// Liveness probe (alias for /health, Kubernetes convention).
async fn livez() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
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

/// Prometheus metrics endpoint.
async fn metrics_endpoint() -> impl IntoResponse {
    let body = metrics::encode_metrics();
    (
        StatusCode::OK,
        [("content-type", "text/plain; charset=utf-8")],
        body,
    )
}
