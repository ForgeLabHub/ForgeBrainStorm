//! Append-only audit log service.

use sqlx::PgPool;
use crate::error::AppResult;

/// Record an admin action in the audit log.
pub async fn log(
    pool: &PgPool,
    actor_id: i64,
    actor_name: &str,
    action: &str,
    target: &str,
    reason: &str,
) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO audit_log (actor_id, actor_name, action, target, reason)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(actor_id)
    .bind(actor_name)
    .bind(action)
    .bind(target)
    .bind(reason)
    .execute(pool)
    .await?;
    Ok(())
}
