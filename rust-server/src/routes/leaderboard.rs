//! Leaderboard routes.

use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;

use crate::error::{AppError, AppResult};
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/top", get(top_players))
        .route("/rank/{username}", get(player_rank))
}

#[derive(Deserialize)]
struct TopQuery {
    limit: Option<i32>,
}

async fn top_players(
    State(state): State<AppState>,
    Query(query): Query<TopQuery>,
) -> AppResult<impl axum::response::IntoResponse> {
    let limit = query.limit.unwrap_or(50).min(200);
    let rows = sqlx::query_as::<_, (i64, String, i64, i32, i32, i32, chrono::DateTime<chrono::Utc>)>(
        "SELECT user_id, username, total_score, games_played, wins, rank, last_seen
         FROM leaderboard_ranked ORDER BY rank LIMIT $1",
    )
    .bind(limit)
    .fetch_all(state.db.pool())
    .await?;

    let players: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "user_id": r.0, "username": r.1, "total_score": r.2,
                "games_played": r.3, "wins": r.4, "rank": r.5, "last_seen": r.6,
            })
        })
        .collect();
    Ok(Json(serde_json::json!({"ok": true, "players": players})))
}

async fn player_rank(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> AppResult<impl axum::response::IntoResponse> {
    let row = sqlx::query_as::<_, (i64, String, i64, i32, i32, i32)>(
        "SELECT user_id, username, total_score, games_played, wins, rank
         FROM leaderboard_ranked WHERE username = $1",
    )
    .bind(&username)
    .fetch_optional(state.db.pool())
    .await?;

    match row {
        Some(r) => Ok(Json(serde_json::json!({
            "ok": true,
            "player": {
                "user_id": r.0, "username": r.1, "total_score": r.2,
                "games_played": r.3, "wins": r.4, "rank": r.5,
            }
        }))),
        None => Err(AppError::NotFound("Игрок не найден")),
    }
}
