//! Profile routes: per-user game history.

use axum::extract::{Query, State};
use axum::middleware;
use axum::routing::get;
use axum::{Extension, Json, Router};
use serde::Deserialize;

use crate::error::AppResult;
use crate::middleware::auth::{auth_middleware, AuthContext};
use crate::services::history;
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/games", get(list_games))
        .route_layer(middleware::from_fn(auth_middleware))
}

#[derive(Debug, Deserialize)]
struct GamesQuery {
    limit: Option<i64>,
}

async fn list_games(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthContext>,
    Query(query): Query<GamesQuery>,
) -> AppResult<impl axum::response::IntoResponse> {
    let limit = query.limit.unwrap_or(10).clamp(1, 50);
    let games = history::recent_games(state.db.pool(), auth.user_id, limit).await?;
    Ok(Json(serde_json::json!({ "ok": true, "games": games })))
}
