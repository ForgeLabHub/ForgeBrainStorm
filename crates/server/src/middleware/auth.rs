//! Session-based authentication middleware.
//!
//! Extracts the session token from the `brainstorm_session` cookie,
//! validates it against the database (with an in-memory cache), and
//! injects [`AuthContext`] into request extensions.

use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use axum_extra::extract::CookieJar;
use dashmap::DashMap;
use sqlx::PgPool;

use crate::domain::role::Role;
use crate::error::{AppError, AppResult};

/// Name of the session cookie.
pub const SESSION_COOKIE: &str = "brainstorm_session";

/// Authenticated user context, injected into request extensions.
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: i64,
    pub username: String,
    pub role: Role,
    pub session_token: String,
}

impl AuthContext {
    pub fn is_admin(&self) -> bool {
        self.role.is_admin()
    }

    pub fn is_cheater(&self) -> bool {
        self.role.is_cheater()
    }
}

/// In-memory cache for validated sessions, with its own pool reference.
#[derive(Clone)]
pub struct SessionCache {
    inner: Arc<DashMap<String, CachedSession>>,
    ttl: Duration,
    pool: PgPool,
}

#[derive(Debug, Clone)]
struct CachedSession {
    ctx: AuthContext,
    expires_at: Instant,
}

impl SessionCache {
    pub fn new(ttl: Duration, pool: PgPool) -> Self {
        Self {
            inner: Arc::new(DashMap::new()),
            ttl,
            pool,
        }
    }

    pub async fn get_or_load(&self, token: &str) -> AppResult<AuthContext> {
        if let Some(cached) = self.inner.get(token) {
            if cached.expires_at > Instant::now() {
                return Ok(cached.ctx.clone());
            }
        }

        // The ban check rides along with the session lookup: a banned account
        // must not keep playing on a session it already had. Cached sessions
        // skip this query, which is why banning also calls `invalidate_user`.
        let row = sqlx::query_as::<_, (i64, String, String, bool)>(
            "SELECT u.id, u.username, u.role,
                    EXISTS (
                        SELECT 1 FROM bans b
                        WHERE b.kind = 'user' AND b.identifier = u.id::text
                          AND b.expires_at > NOW()
                    ) AS banned
             FROM sessions s
             JOIN users u ON u.id = s.user_id
             WHERE s.token = $1 AND s.expires_at > NOW() AND s.revoked_at IS NULL",
        )
        .bind(token)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(AppError::Unauthorized)?;

        if row.3 {
            return Err(AppError::Forbidden("Аккаунт заблокирован"));
        }

        let ctx = AuthContext {
            user_id: row.0,
            username: row.1,
            role: Role::parse(&row.2).unwrap_or_default(),
            session_token: token.to_string(),
        };

        self.inner.insert(
            token.to_string(),
            CachedSession {
                ctx: ctx.clone(),
                expires_at: Instant::now() + self.ttl,
            },
        );

        Ok(ctx)
    }

    pub fn invalidate(&self, token: &str) {
        self.inner.remove(token);
    }

    /// Drop every cached session of one account. Needed after a ban or a role
    /// change: the cache would otherwise serve the old role, or ignore the ban,
    /// for the rest of its TTL.
    pub fn invalidate_user(&self, user_id: i64) {
        self.inner.retain(|_, cached| cached.ctx.user_id != user_id);
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

/// Middleware function: validates the session cookie and injects [`AuthContext`].
pub async fn auth_middleware(
    jar: CookieJar,
    mut request: Request,
    next: Next,
) -> AppResult<Response> {
    let cache = request
        .extensions()
        .get::<SessionCache>()
        .expect("SessionCache must be in extensions")
        .clone();

    let token = jar
        .get(SESSION_COOKIE)
        .map(|c| c.value().to_string())
        .ok_or(AppError::Unauthorized)?;

    let ctx = cache.get_or_load(&token).await?;
    request.extensions_mut().insert(ctx);
    Ok(next.run(request).await)
}

/// Middleware: validate that the account role is `admin`.
pub async fn admin_middleware(
    jar: CookieJar,
    mut request: Request,
    next: Next,
) -> AppResult<Response> {
    let cache = request
        .extensions()
        .get::<SessionCache>()
        .expect("SessionCache must be in extensions")
        .clone();

    let token = jar
        .get(SESSION_COOKIE)
        .map(|c| c.value().to_string())
        .ok_or(AppError::Unauthorized)?;

    let ctx = cache.get_or_load(&token).await?;
    if !ctx.role.can_administer() {
        return Err(AppError::Forbidden("Требуется доступ администратора"));
    }
    request.extensions_mut().insert(ctx);
    Ok(next.run(request).await)
}
