//! Database pool initialisation and migrations.

use sqlx::postgres::{PgPool, PgPoolOptions};
use tracing::info;

use crate::config::Settings;
use crate::error::AppError;

/// Build a connection pool and run pending migrations.
pub async fn init_pool(settings: &Settings) -> Result<PgPool, AppError> {
    let pool = PgPoolOptions::new()
        .max_connections(settings.db_pool_size)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect(&settings.database_url)
        .await?;

    info!("running database migrations");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("migration failed: {e}")))?;
    info!("migrations complete");

    Ok(pool)
}

/// Liveness probe: verify the database is reachable.
pub async fn health_check(pool: &PgPool) -> Result<(), AppError> {
    sqlx::query("SELECT 1").execute(pool).await?;
    Ok(())
}
