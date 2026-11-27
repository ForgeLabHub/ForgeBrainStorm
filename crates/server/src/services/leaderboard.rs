//! Leaderboard service: update aggregates, fetch rankings.

use crate::error::AppResult;
use sqlx::PgPool;

/// Update the leaderboard materialised view after a game.
pub async fn update_leaderboard(
    pool: &PgPool,
    user_id: i64,
    username: &str,
    score: i64,
    won: bool,
    duration_secs: i32,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO leaderboard (user_id, username, total_score, games_played, wins, total_time, last_seen)
         VALUES ($1, $2, $3, 1, $4, $5, NOW())
         ON CONFLICT (user_id) DO UPDATE SET
            total_score = leaderboard.total_score + EXCLUDED.total_score,
            games_played = leaderboard.games_played + 1,
            wins = leaderboard.wins + EXCLUDED.wins,
            total_time = leaderboard.total_time + EXCLUDED.total_time,
            last_seen = NOW()",
    )
    .bind(user_id)
    .bind(username)
    .bind(score)
    .bind(if won { 1 } else { 0 })
    .bind(duration_secs)
    .execute(pool)
    .await?;
    Ok(())
}
