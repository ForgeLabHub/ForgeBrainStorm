//! Database pool initialisation and migrations.

use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use tracing::info;

use crate::config::Settings;
use crate::error::AppError;

/// Build a connection pool and run pending migrations.
pub async fn init_pool(settings: &Settings) -> Result<PgPool, AppError> {
    let url = normalize_url(&settings.database_url);
    let mut connect_options: PgConnectOptions = url.parse()?;

    if uses_transaction_pooler(&url) {
        // Supabase's Supavisor (and PgBouncer in transaction mode) hands the
        // connection to another client between statements, so named prepared
        // statements do not survive. Disabling the cache is the documented fix.
        info!("transaction pooler detected: prepared statement cache disabled");
        connect_options = connect_options.statement_cache_capacity(0);
    }

    let pool = PgPoolOptions::new()
        .max_connections(settings.db_pool_size)
        .acquire_timeout(std::time::Duration::from_secs(5))
        .connect_with(connect_options)
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

/// Hosted Postgres (Supabase, Neon, Render, RDS) refuses plaintext connections,
/// and a missing `sslmode` is the most common first-run failure. Local and
/// Docker hosts are left alone so `postgres://localhost/...` keeps working.
fn normalize_url(url: &str) -> String {
    if url.contains("sslmode=") || !needs_tls(url) {
        return url.to_owned();
    }
    let separator = if url.contains('?') { '&' } else { '?' };
    format!("{url}{separator}sslmode=require")
}

fn needs_tls(url: &str) -> bool {
    let host = url
        .split("://")
        .nth(1)
        .and_then(|rest| rest.split(['/', '?']).next())
        .unwrap_or_default()
        .rsplit('@')
        .next()
        .unwrap_or_default();

    host.contains(".supabase.co")
        || host.contains(".supabase.com")
        || host.contains("neon.tech")
        || host.contains(".render.com")
        || host.contains(".rds.amazonaws.com")
}

/// True when the URL points at a pooler that cannot keep prepared statements.
fn uses_transaction_pooler(url: &str) -> bool {
    url.contains("pooler.supabase.com") || url.contains(":6543")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supabase_urls_get_tls_by_default() {
        let url = normalize_url("postgres://user:pw@db.abcdefgh.supabase.co:5432/postgres");
        assert!(url.ends_with("?sslmode=require"), "{url}");
    }

    #[test]
    fn an_explicit_sslmode_is_kept() {
        let url = "postgres://u:p@db.x.supabase.co:5432/postgres?sslmode=verify-full";
        assert_eq!(normalize_url(url), url);
        let with_params = "postgres://u:p@db.x.supabase.co:5432/postgres?application_name=bs";
        assert_eq!(
            normalize_url(with_params),
            "postgres://u:p@db.x.supabase.co:5432/postgres?application_name=bs&sslmode=require"
        );
    }

    #[test]
    fn local_development_needs_no_tls() {
        assert_eq!(
            normalize_url("postgres://brainstorm:pw@localhost:5432/brainstorm"),
            "postgres://brainstorm:pw@localhost:5432/brainstorm"
        );
    }

    #[test]
    fn the_supabase_pooler_is_recognised() {
        assert!(uses_transaction_pooler(
            "postgres://postgres.abc:pw@aws-0-eu-central-1.pooler.supabase.com:6543/postgres"
        ));
        assert!(uses_transaction_pooler("postgres://u:p@host:6543/postgres"));
        assert!(!uses_transaction_pooler(
            "postgres://u:p@host:5432/postgres"
        ));
        assert!(!uses_transaction_pooler(
            "postgres://u:p@localhost:5432/brainstorm"
        ));
    }
}
