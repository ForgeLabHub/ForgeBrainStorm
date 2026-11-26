//! Administration routes.

use axum::extract::State;
use axum::middleware;
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use serde::Deserialize;

use crate::error::{AppError, AppResult};
use crate::middleware::auth::{AuthContext, SessionCache};
use crate::services::audit;
use crate::AppState;

pub fn routes(session_cache: SessionCache) -> Router<AppState> {
    let admin_routes = Router::new()
        .route("/rooms", get(list_rooms))
        .route("/users", get(list_users))
        .route("/ban", post(ban_user))
        .route("/unban", post(unban_user))
        .route("/reset_user", post(reset_user))
        .route("/audit_log", get(audit_log))
        .route_layer(middleware::from_fn(
            crate::middleware::auth::admin_middleware,
        ));

    Router::new()
        .route("/activate", post(activate))
        .route("/logout", post(admin_logout))
        .merge(admin_routes)
        .layer(Extension(session_cache))
}

// ── Handlers ────────────────────────────────────────────────

#[derive(Deserialize)]
struct ActivateInput {
    key: String,
}

async fn activate(
    State(state): State<AppState>,
    Json(input): Json<ActivateInput>,
) -> AppResult<impl axum::response::IntoResponse> {
    if crate::security::safe_equal(&input.key, &state.settings.admin_secret_key) {
        return Ok(Json(serde_json::json!({"ok": true, "role": "admin"})));
    }
    Err(AppError::Forbidden("Неверный ключ"))
}

async fn admin_logout() -> impl axum::response::IntoResponse {
    Json(serde_json::json!({"ok": true}))
}

async fn list_rooms(
    Extension(_auth): Extension<AuthContext>,
) -> AppResult<impl axum::response::IntoResponse> {
    // TODO: list from in-memory room registry
    Ok(Json(Vec::<()>::new()))
}

async fn list_users(
    Extension(_auth): Extension<AuthContext>,
    State(state): State<AppState>,
) -> AppResult<impl axum::response::IntoResponse> {
    let rows = sqlx::query_as::<_, (i64, String, i64, i32, i32)>(
        "SELECT user_id, username, total_score, games_played, wins
         FROM leaderboard_ranked ORDER BY rank LIMIT 200",
    )
    .fetch_all(state.db.pool())
    .await?;

    let users: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "user_id": r.0, "username": r.1,
                "total_score": r.2, "games_played": r.3, "wins": r.4,
            })
        })
        .collect();
    Ok(Json(users))
}

#[derive(Deserialize)]
struct BanInput {
    identifier: String,
    kind: String,
    reason: Option<String>,
    duration_minutes: Option<i32>,
}

async fn ban_user(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Json(input): Json<BanInput>,
) -> AppResult<impl axum::response::IntoResponse> {
    let duration = input.duration_minutes.unwrap_or(60);
    sqlx::query(
        "INSERT INTO bans (identifier, kind, reason, issued_by, expires_at)
         VALUES ($1, $2, $3, $4, NOW() + make_interval(mins => $5))
         ON CONFLICT (identifier, kind) DO UPDATE SET
            reason = EXCLUDED.reason, expires_at = EXCLUDED.expires_at",
    )
    .bind(&input.identifier)
    .bind(&input.kind)
    .bind(input.reason.as_deref().unwrap_or(""))
    .bind(auth.user_id)
    .bind(duration)
    .execute(state.db.pool())
    .await?;

    audit::log(
        state.db.pool(),
        auth.user_id,
        &auth.username,
        "ban",
        &input.identifier,
        input.reason.as_deref().unwrap_or(""),
    )
    .await?;

    Ok(Json(serde_json::json!({"ok": true})))
}

#[derive(Deserialize)]
struct UnbanInput {
    identifier: String,
    kind: Option<String>,
}

async fn unban_user(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Json(input): Json<UnbanInput>,
) -> AppResult<impl axum::response::IntoResponse> {
    let kind = input.kind.as_deref().unwrap_or("user");
    sqlx::query("DELETE FROM bans WHERE identifier = $1 AND kind = $2")
        .bind(&input.identifier)
        .bind(kind)
        .execute(state.db.pool())
        .await?;

    audit::log(
        state.db.pool(),
        auth.user_id,
        &auth.username,
        "unban",
        &input.identifier,
        "",
    )
    .await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

#[derive(Deserialize)]
struct ResetInput {
    username: String,
}

async fn reset_user(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Json(input): Json<ResetInput>,
) -> AppResult<impl axum::response::IntoResponse> {
    sqlx::query(
        "UPDATE leaderboard SET total_score = 0, wins = 0, games_played = 0, total_time = 0
         WHERE username = $1",
    )
    .bind(&input.username)
    .execute(state.db.pool())
    .await?;

    audit::log(
        state.db.pool(),
        auth.user_id,
        &auth.username,
        "reset_stats",
        &input.username,
        "",
    )
    .await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

async fn audit_log(
    Extension(_auth): Extension<AuthContext>,
    State(state): State<AppState>,
) -> AppResult<impl axum::response::IntoResponse> {
    let rows = sqlx::query_as::<
        _,
        (
            i64,
            Option<i64>,
            String,
            String,
            String,
            String,
            chrono::DateTime<chrono::Utc>,
        ),
    >(
        "SELECT id, actor_id, actor_name, action, target, reason, created_at
         FROM audit_log ORDER BY created_at DESC LIMIT 100",
    )
    .fetch_all(state.db.pool())
    .await?;

    let entries: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "id": r.0, "actor_id": r.1, "actor_name": r.2,
                "action": r.3, "target": r.4, "reason": r.5, "created_at": r.6,
            })
        })
        .collect();
    Ok(Json(entries))
}
