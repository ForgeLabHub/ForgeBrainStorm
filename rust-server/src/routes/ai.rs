//! AI provider management and question generation routes.

use axum::extract::State;
use axum::middleware;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use serde::Deserialize;
use validator::Validate;

use crate::error::{AppError, AppResult, ValidationError};
use crate::middleware::auth::{auth_middleware, AuthContext};
use crate::services::ai::{AiCapability, GenerationRequest};
use crate::AppState;

pub fn routes() -> Router<AppState> {
    let authenticated = Router::new()
        .route("/generate", post(generate_questions))
        .route("/providers/switch", post(switch_provider))
        .route("/cache/clear", post(clear_cache))
        .route_layer(middleware::from_fn(auth_middleware));

    Router::new()
        .route("/providers", get(list_providers))
        .route("/cache/stats", get(cache_stats))
        .merge(authenticated)
}

/// List available AI providers.
async fn list_providers(State(state): State<AppState>) -> AppResult<impl IntoResponse> {
    Ok(Json(serde_json::json!({
        "ok": true,
        "providers": state.ai_router.available_providers(),
        "default": state.settings.default_ai_provider,
    })))
}

#[derive(Debug, Deserialize, Validate)]
struct GenerateInput {
    #[validate(length(min = 1, max = 200, message = "Тема: от 1 до 200 символов"))]
    topic: String,
    count: Option<usize>,
    difficulty: Option<String>,
    num_options: Option<usize>,
}

/// Generate quiz questions through the AI router (with provider fallback).
async fn generate_questions(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Json(input): Json<GenerateInput>,
) -> AppResult<impl IntoResponse> {
    input
        .validate()
        .map_err(|e| AppError::Validation(ValidationError::new("topic", e.to_string())))?;

    let request = GenerationRequest {
        topic: input.topic,
        count: input.count.unwrap_or(10).clamp(1, 30),
        difficulty: input.difficulty.unwrap_or_else(|| "medium".into()),
        num_options: input.num_options.unwrap_or(4).clamp(2, 6),
        capabilities: vec![AiCapability::QuestionGeneration],
        exclude_hashes: Vec::new(),
    };

    let response = state
        .ai_router
        .generate_questions(&request)
        .await
        .map_err(|e| AppError::AiUnavailable(e.to_string()))?;

    tracing::info!(
        user_id = auth.user_id,
        provider = %response.provider_used,
        questions = response.questions.len(),
        "generated questions via AI router"
    );

    Ok(Json(serde_json::json!({
        "ok": true,
        "provider": response.provider_used,
        "tokens_used": response.tokens_used,
        "latency_ms": response.latency_ms,
        "questions": response.questions,
    })))
}

#[derive(Deserialize)]
struct SwitchProviderInput {
    provider: String,
}

/// Switch the default AI provider at runtime.
async fn switch_provider(
    State(state): State<AppState>,
    Json(input): Json<SwitchProviderInput>,
) -> AppResult<impl IntoResponse> {
    if state.ai_router.set_default_provider(&input.provider) {
        Ok(Json(serde_json::json!({
            "ok": true,
            "message": format!("Default provider switched to: {}", input.provider),
        })))
    } else {
        Err(AppError::NotFound("AI provider"))
    }
}

/// Get AI cache statistics.
async fn cache_stats(State(state): State<AppState>) -> AppResult<impl IntoResponse> {
    Ok(Json(serde_json::json!({
        "ok": true,
        "cache": state.ai_router.cache_stats(),
    })))
}

/// Clear the AI response cache.
async fn clear_cache(State(state): State<AppState>) -> AppResult<impl IntoResponse> {
    state.ai_router.clear_cache();
    Ok(Json(serde_json::json!({
        "ok": true,
        "message": "AI cache cleared",
    })))
}
