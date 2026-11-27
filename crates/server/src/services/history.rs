//! Finished-game history: persisted for the profile page and admin audits.

use serde::Serialize;
use sqlx::PgPool;
use tracing::warn;

use crate::error::AppResult;

/// Room-level context shared by every player of one finished game.
pub struct GameInfo {
    pub room_code: String,
    pub topic: String,
    pub mode: String,
    pub players: i32,
    pub duration_secs: i32,
}

/// One player's result inside a finished game.
pub struct GameOutcome {
    pub user_id: i64,
    pub username: String,
    pub score: i64,
    pub place: i32,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct RecentGame {
    pub room_code: String,
    pub topic: String,
    pub mode: String,
    pub score: i64,
    pub place: i32,
    pub players: i32,
    pub duration_secs: i32,
    pub played_at: chrono::DateTime<chrono::Utc>,
}

/// Persist the finished game for every authenticated player.
pub async fn save_game(pool: &PgPool, info: &GameInfo, outcomes: &[GameOutcome]) -> AppResult<()> {
    for outcome in outcomes {
        if let Err(error) = sqlx::query(
            "INSERT INTO user_games
                (user_id, room_code, topic, mode, score, place, players, duration_secs)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(outcome.user_id)
        .bind(&info.room_code)
        .bind(&info.topic)
        .bind(&info.mode)
        .bind(outcome.score)
        .bind(outcome.place)
        .bind(info.players)
        .bind(info.duration_secs)
        .execute(pool)
        .await
        {
            warn!(user_id = outcome.user_id, %error, "failed to save user game");
        }
    }

    let players_json = serde_json::to_value(prepare_players(outcomes)).unwrap_or_default();
    if let Err(error) = sqlx::query(
        "INSERT INTO room_history (room_code, mode, topic, duration_secs, players_json)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(&info.room_code)
    .bind(&info.mode)
    .bind(&info.topic)
    .bind(info.duration_secs)
    .bind(players_json)
    .execute(pool)
    .await
    {
        warn!(room = %info.room_code, %error, "failed to save room history");
    }

    Ok(())
}

fn prepare_players(outcomes: &[GameOutcome]) -> Vec<serde_json::Value> {
    outcomes
        .iter()
        .map(|outcome| {
            serde_json::json!({
                "user_id": outcome.user_id,
                "username": outcome.username,
                "score": outcome.score,
                "place": outcome.place,
            })
        })
        .collect()
}

/// Most recent finished games for one user, newest first.
pub async fn recent_games(pool: &PgPool, user_id: i64, limit: i64) -> AppResult<Vec<RecentGame>> {
    let games = sqlx::query_as::<_, RecentGame>(
        "SELECT room_code, topic, mode, score, place, players, duration_secs, played_at
         FROM user_games WHERE user_id = $1
         ORDER BY played_at DESC LIMIT $2",
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(games)
}
