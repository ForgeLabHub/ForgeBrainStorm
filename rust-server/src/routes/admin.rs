//! Administration panel API.
//!
//! Every route here sits behind [`crate::middleware::auth::admin_middleware`],
//! which reads the account role from the database — there is no activation key
//! to present, and a session cannot promote itself.

use axum::extract::{Query, State};
use axum::middleware;
use axum::routing::{get, post};
use axum::{Extension, Json, Router};
use serde::Deserialize;

use crate::domain::role::Role;
use crate::error::{AppError, AppResult, ValidationError};
use crate::middleware::auth::{AuthContext, SessionCache};
use crate::services::audit;
use crate::AppState;

pub fn routes(session_cache: SessionCache) -> Router<AppState> {
    Router::new()
        .route("/stats", get(stats))
        .route("/users", get(list_users))
        .route("/users/role", post(set_role))
        .route("/users/reset", post(reset_user))
        .route("/users/delete", post(delete_user))
        .route("/bans", get(list_bans))
        .route("/ban", post(ban_user))
        .route("/unban", post(unban_user))
        .route("/rooms", get(list_rooms))
        .route("/rooms/close", post(close_room))
        .route("/broadcast", post(broadcast))
        .route("/audit_log", get(audit_log))
        .route("/ai", get(ai_status))
        .route("/ai/switch", post(switch_provider))
        .route("/ai/cache/clear", post(clear_ai_cache))
        .route_layer(middleware::from_fn(
            crate::middleware::auth::admin_middleware,
        ))
        .layer(Extension(session_cache))
}

// ── Dashboard ───────────────────────────────────────────────

async fn stats(State(state): State<AppState>) -> AppResult<impl axum::response::IntoResponse> {
    let (users, admins, cheaters): (i64, i64, i64) = sqlx::query_as(
        "SELECT COUNT(*),
                COUNT(*) FILTER (WHERE role = 'admin'),
                COUNT(*) FILTER (WHERE role = 'cheater')
         FROM users",
    )
    .fetch_one(state.db.pool())
    .await?;

    let (games, score): (i64, i64) =
        sqlx::query_as("SELECT COUNT(*), COALESCE(SUM(score), 0)::BIGINT FROM user_games")
            .fetch_one(state.db.pool())
            .await?;

    let active_bans: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM bans WHERE expires_at > NOW()")
        .fetch_one(state.db.pool())
        .await?;

    let rooms = state.rooms.list_public();
    Ok(Json(serde_json::json!({
        "users": users,
        "admins": admins,
        "cheaters": cheaters,
        "games": games,
        "total_score": score,
        "active_bans": active_bans,
        "live_rooms": rooms.len(),
        "live_players": rooms.iter().map(|r| r.player_count).sum::<usize>(),
    })))
}

// ── Users ───────────────────────────────────────────────────

#[derive(Deserialize)]
struct UserQuery {
    q: Option<String>,
}

async fn list_users(
    State(state): State<AppState>,
    Query(query): Query<UserQuery>,
) -> AppResult<impl axum::response::IntoResponse> {
    let pattern = format!("%{}%", query.q.unwrap_or_default());
    let rows = sqlx::query_as::<_, (i64, String, String, i64, i32, i32, i32, bool)>(
        "SELECT u.id, u.username, u.role, u.xp, u.coins, u.games_played, u.wins,
                EXISTS (
                    SELECT 1 FROM bans b
                    WHERE b.identifier = u.id::text AND b.kind = 'user'
                      AND b.expires_at > NOW()
                ) AS banned
         FROM users u
         WHERE u.username ILIKE $1
         ORDER BY u.xp DESC, u.id
         LIMIT 100",
    )
    .bind(&pattern)
    .fetch_all(state.db.pool())
    .await?;

    let users: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "user_id": r.0,
                "username": r.1,
                "role": r.2,
                "xp": r.3,
                "coins": r.4,
                "games_played": r.5,
                "wins": r.6,
                "banned": r.7,
            })
        })
        .collect();
    Ok(Json(users))
}

#[derive(Deserialize)]
struct RoleInput {
    user_id: i64,
    role: String,
}

async fn set_role(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Json(input): Json<RoleInput>,
) -> AppResult<impl axum::response::IntoResponse> {
    let role = Role::parse(&input.role)
        .ok_or_else(|| AppError::Validation(ValidationError::new("role", "Неизвестная роль")))?;
    if input.user_id == auth.user_id && role != Role::Admin {
        return Err(AppError::Forbidden("Нельзя снять роль администратора с себя"));
    }

    let target: Option<String> =
        sqlx::query_scalar("UPDATE users SET role = $2 WHERE id = $1 RETURNING username")
            .bind(input.user_id)
            .bind(role.as_str())
            .fetch_optional(state.db.pool())
            .await?;
    let target = target.ok_or_else(|| AppError::NotFound("Пользователь не найден"))?;

    audit::log(
        state.db.pool(),
        auth.user_id,
        &auth.username,
        "set_role",
        &target,
        role.as_str(),
    )
    .await?;
    Ok(Json(serde_json::json!({"ok": true, "role": role.as_str()})))
}

#[derive(Deserialize)]
struct UserRef {
    user_id: i64,
}

async fn reset_user(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Json(input): Json<UserRef>,
) -> AppResult<impl axum::response::IntoResponse> {
    let target: Option<String> = sqlx::query_scalar(
        "UPDATE users SET xp = 0, coins = 50, games_played = 0, wins = 0, total_score = 0
         WHERE id = $1 RETURNING username",
    )
    .bind(input.user_id)
    .fetch_optional(state.db.pool())
    .await?;
    let target = target.ok_or_else(|| AppError::NotFound("Пользователь не найден"))?;

    sqlx::query(
        "UPDATE leaderboard SET total_score = 0, wins = 0, games_played = 0, total_time = 0
         WHERE user_id = $1",
    )
    .bind(input.user_id)
    .execute(state.db.pool())
    .await?;

    audit::log(
        state.db.pool(),
        auth.user_id,
        &auth.username,
        "reset_stats",
        &target,
        "",
    )
    .await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

async fn delete_user(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Json(input): Json<UserRef>,
) -> AppResult<impl axum::response::IntoResponse> {
    if input.user_id == auth.user_id {
        return Err(AppError::Forbidden("Нельзя удалить свой аккаунт"));
    }
    let target: Option<String> =
        sqlx::query_scalar("DELETE FROM users WHERE id = $1 RETURNING username")
            .bind(input.user_id)
            .fetch_optional(state.db.pool())
            .await?;
    let target = target.ok_or_else(|| AppError::NotFound("Пользователь не найден"))?;

    audit::log(
        state.db.pool(),
        auth.user_id,
        &auth.username,
        "delete_user",
        &target,
        "",
    )
    .await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

// ── Bans ────────────────────────────────────────────────────

#[derive(Deserialize)]
struct BanInput {
    /// Numeric user id (stored as `kind = 'user'`) or an IP address.
    identifier: String,
    kind: Option<String>,
    reason: Option<String>,
    duration_minutes: Option<i32>,
}

async fn ban_user(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Json(input): Json<BanInput>,
) -> AppResult<impl axum::response::IntoResponse> {
    let kind = input.kind.as_deref().unwrap_or("user");
    if !matches!(kind, "user" | "ip") {
        return Err(AppError::Validation(ValidationError::new(
            "kind",
            "kind: user или ip",
        )));
    }
    let duration = input.duration_minutes.unwrap_or(60).clamp(1, 60 * 24 * 30);
    sqlx::query(
        "INSERT INTO bans (identifier, kind, reason, issued_by, expires_at)
         VALUES ($1, $2, $3, $4, NOW() + make_interval(mins => $5))
         ON CONFLICT (identifier, kind) DO UPDATE SET
            reason = EXCLUDED.reason, expires_at = EXCLUDED.expires_at",
    )
    .bind(&input.identifier)
    .bind(kind)
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

async fn unban_user(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Json(input): Json<BanInput>,
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

async fn list_bans(State(state): State<AppState>) -> AppResult<impl axum::response::IntoResponse> {
    let rows = sqlx::query_as::<_, (i64, String, String, String, chrono::DateTime<chrono::Utc>)>(
        "SELECT id, identifier, kind, reason, expires_at
         FROM bans WHERE expires_at > NOW()
         ORDER BY expires_at DESC LIMIT 100",
    )
    .fetch_all(state.db.pool())
    .await?;
    let bans: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "id": r.0, "identifier": r.1, "kind": r.2, "reason": r.3, "expires_at": r.4,
            })
        })
        .collect();
    Ok(Json(bans))
}

// ── Rooms ───────────────────────────────────────────────────

async fn list_rooms(State(state): State<AppState>) -> AppResult<impl axum::response::IntoResponse> {
    let mut rooms = state.rooms.list_public();
    rooms.sort_by_key(|room| std::cmp::Reverse(room.player_count));
    Ok(Json(rooms))
}

#[derive(Deserialize)]
struct RoomRef {
    code: String,
}

async fn close_room(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Json(input): Json<RoomRef>,
) -> AppResult<impl axum::response::IntoResponse> {
    let code = input.code.to_uppercase();
    let Some(room) = state.rooms.get_arc(&code) else {
        return Err(AppError::NotFound("Комната не найдена"));
    };
    {
        let room = room.read();
        let _ = room.tx.send(
            serde_json::json!({
                "type": "error",
                "data": {"message": "Комната закрыта администратором"}
            })
            .to_string(),
        );
    }
    state.rooms.remove(&code);
    audit::log(
        state.db.pool(),
        auth.user_id,
        &auth.username,
        "close_room",
        &code,
        "",
    )
    .await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

#[derive(Deserialize)]
struct BroadcastInput {
    message: String,
}

async fn broadcast(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Json(input): Json<BroadcastInput>,
) -> AppResult<impl axum::response::IntoResponse> {
    let text = input.message.trim().chars().take(200).collect::<String>();
    if text.is_empty() {
        return Err(AppError::Validation(ValidationError::new(
            "message",
            "Пустое сообщение",
        )));
    }
    let rooms = state.rooms.list_public();
    let mut delivered = 0;
    for snapshot in &rooms {
        if let Some(room) = state.rooms.get_arc(&snapshot.code) {
            let room = room.read();
            if room
                .tx
                .send(
                    serde_json::json!({"type": "announcement", "data": {"message": text}})
                        .to_string(),
                )
                .is_ok()
            {
                delivered += 1;
            }
        }
    }
    audit::log(
        state.db.pool(),
        auth.user_id,
        &auth.username,
        "broadcast",
        &format!("{delivered} комнат"),
        &text,
    )
    .await?;
    Ok(Json(serde_json::json!({"ok": true, "rooms": delivered})))
}

// ── Audit ───────────────────────────────────────────────────

async fn audit_log(State(state): State<AppState>) -> AppResult<impl axum::response::IntoResponse> {
    let rows = sqlx::query_as::<
        _,
        (
            i64,
            Option<i64>,
            Option<String>,
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

// ── AI provider controls (moved off the public site) ────────

async fn ai_status(State(state): State<AppState>) -> AppResult<impl axum::response::IntoResponse> {
    let cache = state.ai_router.cache_stats();
    Ok(Json(serde_json::json!({
        "providers": state.ai_router.available_providers(),
        "default": state.settings.default_ai_provider,
        "questions_cached": cache.questions_cached,
        "hints_cached": cache.hints_cached,
    })))
}

#[derive(Deserialize)]
struct ProviderInput {
    provider: String,
}

async fn switch_provider(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Json(input): Json<ProviderInput>,
) -> AppResult<impl axum::response::IntoResponse> {
    if !state.ai_router.set_default_provider(&input.provider) {
        return Err(AppError::Validation(ValidationError::new(
            "provider",
            "Провайдер недоступен",
        )));
    }
    audit::log(
        state.db.pool(),
        auth.user_id,
        &auth.username,
        "switch_ai_provider",
        &input.provider,
        "",
    )
    .await?;
    Ok(Json(serde_json::json!({"ok": true, "provider": input.provider})))
}

async fn clear_ai_cache(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
) -> AppResult<impl axum::response::IntoResponse> {
    state.ai_router.clear_cache();
    audit::log(
        state.db.pool(),
        auth.user_id,
        &auth.username,
        "clear_ai_cache",
        "",
        "",
    )
    .await?;
    Ok(Json(serde_json::json!({"ok": true})))
}
