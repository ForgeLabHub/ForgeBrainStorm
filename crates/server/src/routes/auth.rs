//! Authentication routes: register, login, logout, current user.

use argon2::{PasswordHasher, PasswordVerifier};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::CookieJar;
use serde::Deserialize;
use validator::Validate;

use crate::domain::role::Role;
use crate::error::{AppError, AppResult, ValidationError};
use crate::middleware::auth::{SessionCache, SESSION_COOKIE};
use crate::AppState;

pub fn routes(session_cache: SessionCache) -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/me", get(me))
        .layer(Extension(session_cache))
}

// ── DTOs ────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Validate)]
struct RegisterInput {
    #[validate(length(min = 2, max = 20, message = "Имя: от 2 до 20 символов"))]
    username: String,
    /// A role code is shorter than the ordinary password minimum by design.
    #[validate(length(min = 4, max = 128, message = "Пароль: от 12 до 128 символов"))]
    password: String,
}

/// Ordinary passwords must be long enough to be worth hashing; role codes are
/// fixed short strings that the operator handed out and are exempt.
fn password_is_acceptable(password: &str) -> bool {
    Role::from_registration_password(password).is_some() || password.chars().count() >= 12
}

#[derive(Debug, Deserialize)]
struct LoginInput {
    username: String,
    password: String,
}

// ── Handlers ────────────────────────────────────────────────

async fn register(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(input): Json<RegisterInput>,
) -> AppResult<impl IntoResponse> {
    input
        .validate()
        .map_err(|e| AppError::Validation(ValidationError::new("username", e.to_string())))?;
    if !password_is_acceptable(&input.password) {
        return Err(AppError::Validation(ValidationError::new(
            "password",
            "Пароль: от 12 до 128 символов",
        )));
    }

    let role = Role::from_registration_password(&input.password).unwrap_or_default();

    let salt =
        argon2::password_hash::SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
    let argon2_hasher = argon2::Argon2::default();
    let hash = argon2_hasher
        .hash_password(input.password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("hash error: {e}")))?
        .to_string();

    let user_id: i64 = sqlx::query_scalar(
        "INSERT INTO users (username, pwd_hash, role) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(&input.username)
    .bind(&hash)
    .bind(role.as_str())
    .fetch_one(state.db.pool())
    .await
    .map_err(|e: sqlx::Error| {
        if e.to_string().contains("duplicate") || e.to_string().contains("unique") {
            AppError::Conflict("Пользователь с таким именем уже существует".into())
        } else {
            AppError::Database(Box::new(e))
        }
    })?;

    let token = create_session(state.db.pool(), user_id).await?;
    let jar = set_session_cookie(jar, &token, &state);
    let user = fetch_user_response(state.db.pool(), user_id).await?;

    let mut response = (jar, Json(serde_json::json!({"ok": true, "user": user}))).into_response();
    *response.status_mut() = StatusCode::CREATED;
    Ok(response)
}

async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(input): Json<LoginInput>,
) -> AppResult<impl IntoResponse> {
    let row =
        sqlx::query_as::<_, (i64, String)>("SELECT id, pwd_hash FROM users WHERE username = $1")
            .bind(&input.username)
            .fetch_optional(state.db.pool())
            .await?;

    let (user_id, stored_hash) = match row {
        Some(r) => r,
        None => return Err(AppError::Unauthorized),
    };

    let parsed = argon2::PasswordHash::new(&stored_hash)
        .map_err(|e| AppError::Internal(anyhow::anyhow!("corrupt hash: {e}")))?;
    if argon2::Argon2::default()
        .verify_password(input.password.as_bytes(), &parsed)
        .is_err()
    {
        return Err(AppError::Unauthorized);
    }

    let token = create_session(state.db.pool(), user_id).await?;
    let jar = set_session_cookie(jar, &token, &state);
    let user = fetch_user_response(state.db.pool(), user_id).await?;

    Ok((jar, Json(serde_json::json!({"ok": true, "user": user}))))
}

async fn logout(State(state): State<AppState>, jar: CookieJar) -> impl IntoResponse {
    if let Some(cookie) = jar.get(SESSION_COOKIE) {
        let token = cookie.value();
        let _ = sqlx::query("UPDATE sessions SET revoked_at = NOW() WHERE token = $1")
            .bind(token)
            .execute(state.db.pool())
            .await;
        state.session_cache.invalidate(token);
    }
    let jar = jar.remove(Cookie::from(SESSION_COOKIE));
    (jar, Json(serde_json::json!({"ok": true})))
}

async fn me(
    Extension(cache): Extension<SessionCache>,
    State(state): State<AppState>,
    jar: CookieJar,
) -> AppResult<impl IntoResponse> {
    let token = jar
        .get(SESSION_COOKIE)
        .map(|c| c.value().to_string())
        .ok_or(AppError::Unauthorized)?;
    let ctx = cache.get_or_load(&token).await?;
    let user = fetch_user_response(state.db.pool(), ctx.user_id).await?;
    Ok(Json(
        serde_json::json!({"ok": true, "logged_in": true, "user": user}),
    ))
}

// ── Helpers ─────────────────────────────────────────────────

async fn create_session(pool: &sqlx::PgPool, user_id: i64) -> AppResult<String> {
    let token = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO sessions (token, user_id, expires_at)
         VALUES ($1, $2, NOW() + INTERVAL '1 day')",
    )
    .bind(&token)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(token)
}

fn set_session_cookie(jar: CookieJar, token: &str, state: &AppState) -> CookieJar {
    let mut cookie = Cookie::build((SESSION_COOKIE, token.to_string()))
        .path("/")
        .http_only(true)
        .same_site(axum_extra::extract::cookie::SameSite::Strict);
    if state.settings.session_cookie_secure {
        cookie = cookie.secure(true);
    }
    jar.add(cookie.build())
}

async fn fetch_user_response(pool: &sqlx::PgPool, user_id: i64) -> AppResult<serde_json::Value> {
    let row = sqlx::query_as::<_, (String, i64, i32, i32, i32, i64, String)>(
        "SELECT u.username, u.xp, u.coins, u.games_played, u.wins, \
         COALESCE(l.total_score, 0), u.role \
         FROM users u LEFT JOIN leaderboard l ON l.user_id = u.id WHERE u.id = $1",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    let level = ((row.1 as f64 / 100.0).sqrt().floor() as i32).max(1);
    Ok(serde_json::json!({
        "username": row.0,
        "xp": row.1,
        "coins": row.2,
        "games_played": row.3,
        "wins": row.4,
        "total_score": row.5,
        "level": level,
        "role": row.6,
    }))
}
