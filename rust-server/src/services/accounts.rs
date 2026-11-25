//! Account service: user lookup, stats update, coins.

use sqlx::PgPool;
use crate::error::AppResult;

/// Update a user's aggregate statistics after a game.
pub async fn update_stats(
    pool: &PgPool,
    user_id: i64,
    score: i64,
    won: bool,
    xp_gain: i64,
) -> AppResult<()> {
    sqlx::query(
        "UPDATE users SET
            games_played = games_played + 1,
            total_score = total_score + $2,
            wins = wins + $3,
            xp = xp + $4,
            last_seen = NOW()
         WHERE id = $1",
    )
    .bind(user_id)
    .bind(score)
    .bind(if won { 1 } else { 0 })
    .bind(xp_gain.max(0))
    .execute(pool)
    .await?;
    Ok(())
}

/// Add coins to a user's balance.
pub async fn add_coins(pool: &PgPool, user_id: i64, amount: i32) -> AppResult<i32> {
    let row = sqlx::query_scalar::<_, i32>(
        "UPDATE users SET coins = GREATEST(0, coins + $2) WHERE id = $1 RETURNING coins",
    )
    .bind(user_id)
    .bind(amount)
    .fetch_one(pool)
    .await?;
    Ok(row)
}
