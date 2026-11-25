//! AI provider management routes.
//!
//! Allows runtime switching between AI providers, viewing available providers,
//! and checking cache statistics.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/providers", get(list_providers))
        .route("/providers/switch", post(switch_provider))
        .route("/cache/stats", get(cache_stats))
        .route("/cache/clear", post(clear_cache))
}

/// List available AI providers.
async fn list_providers(
    State(state): State<AppState>,
) -> AppResult<impl IntoResponse> {
    let providers = state.ai_router.available_providers();
    
    Ok(Json(serde_json::json!({
        "ok": true,
        "providers": providers,
        "default": state.settings.default_ai_provider,
    })))
}

/// Switch default AI provider.
#[derive(Deserialize)]
struct SwitchProviderInput {
    provider: String,
}

async fn switch_provider(
    State(mut state): State<AppState>,
    Json(input): Json<SwitchProviderInput>,
) -> AppResult<impl IntoResponse> {
    let success = state.ai_router.set_default_provider(&input.provider);
    
    if success {
        Ok(Json(serde_json::json!({
            "ok": true,
            "message": format!("Default provider switched to: {}", input.provider),
        })))
    } else {
        Err(AppError::NotFound("Provider not found"))
    }
}

/// Get AI cache statistics.
async fn cache_stats(
    State(state): State<AppState>,
) -> AppResult<impl IntoResponse> {
    let stats = state.ai_router.cache_stats().await;
    
    Ok(Json(serde_json::json!({
        "ok": true,
        "cache": stats,
    })))
}

/// Clear AI cache.
async fn clear_cache(
    State(state): State<AppState>,
) -> AppResult<impl IntoResponse> {
    state.ai_router.clear_cache().await;
    
    Ok(Json(serde_json::json!({
        "ok": true,
        "message": "AI cache cleared",
    })))
}
