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
        .route("/rooms/detail", get(room_detail))
        .route("/rooms/close", post(close_room))
        .route("/rooms/kick", post(kick_player))
        .route("/rooms/announce", post(room_announce))
        .route("/rooms/chat_lock", post(set_chat_lock))
        .route("/rooms/chat_clear", post(clear_chat))
        .route("/sessions", get(list_sessions))
        .route("/sessions/revoke", post(revoke_session))
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

// ── Shared helpers ──────────────────────────────────────────

/// Drop every session of `user_id` — cache first, then the database rows.
/// A ban or a demotion that leaves the old session alive is theatre.
async fn revoke_sessions(state: &AppState, user_id: i64) -> AppResult<u64> {
    state.session_cache.invalidate_user(user_id);
    let result = sqlx::query(
        "UPDATE sessions SET revoked_at = NOW()
         WHERE user_id = $1 AND revoked_at IS NULL",
    )
    .bind(user_id)
    .execute(state.db.pool())
    .await?;
    Ok(result.rows_affected())
}

/// Accept either a numeric id or a username. The ban table stores ids as text,
/// so a username written straight into it would never match anything.
async fn resolve_user_id(state: &AppState, identifier: &str) -> AppResult<i64> {
    let trimmed = identifier.trim();
    if let Ok(id) = trimmed.parse::<i64>() {
        return Ok(id);
    }
    sqlx::query_scalar::<_, i64>("SELECT id FROM users WHERE lower(username::text) = lower($1)")
        .bind(trimmed)
        .fetch_optional(state.db.pool())
        .await?
        .ok_or_else(|| AppError::NotFound("Пользователь не найден"))
}

/// Remove a user from every live room. Their socket gets `kicked` and the room
/// sees them leave, exactly as if they had closed the tab.
fn kick_from_rooms(state: &AppState, user_id: i64, message: &str) -> usize {
    let mut kicked = 0;
    for snapshot in state.rooms.list_public() {
        let Some(room) = state.rooms.get_arc(&snapshot.code) else {
            continue;
        };
        let mut room = room.write();
        let sids: Vec<String> = room
            .players
            .values()
            .filter(|player| player.user_id == Some(user_id))
            .map(|player| player.sid.clone())
            .collect();
        if sids.is_empty() {
            continue;
        }
        for sid in sids {
            if let Some(player) = room.players.remove(&sid) {
                if let Some(outbox) = player.outbox {
                    let _ = outbox.send(
                        serde_json::json!({
                            "type": "kicked",
                            "data": {"message": message}
                        })
                        .to_string(),
                    );
                }
                kicked += 1;
            }
        }
        let event = crate::ws::players_event(&room);
        room.touch();
        let _ = room.tx.send(event);
    }
    kicked
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

    let live_sessions: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM sessions WHERE revoked_at IS NULL AND expires_at > NOW()",
    )
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
        "live_sessions": live_sessions,
        "live_rooms": rooms.len(),
        "live_players": rooms.iter().map(|r| r.player_count).sum::<usize>(),
    })))
}

// ── Users ───────────────────────────────────────────────────

#[derive(Deserialize)]
struct UserQuery {
    q: Option<String>,
    role: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
}

async fn list_users(
    State(state): State<AppState>,
    Query(query): Query<UserQuery>,
) -> AppResult<impl axum::response::IntoResponse> {
    let pattern = format!("%{}%", query.q.unwrap_or_default());
    let role = query.role.filter(|role| Role::parse(role).is_some());
    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let offset = query.offset.unwrap_or(0).max(0);

    let rows = sqlx::query_as::<_, (i64, String, String, i64, i32, i32, i32, bool)>(
        "SELECT u.id, u.username, u.role, u.xp, u.coins, u.games_played, u.wins,
                EXISTS (
                    SELECT 1 FROM bans b
                    WHERE b.identifier = u.id::text AND b.kind = 'user'
                      AND b.expires_at > NOW()
                ) AS banned
         FROM users u
         WHERE u.username::text ILIKE $1
           AND ($2::TEXT IS NULL OR u.role = $2)
         ORDER BY u.xp DESC, u.id
         LIMIT $3 OFFSET $4",
    )
    .bind(&pattern)
    .bind(&role)
    .bind(limit)
    .bind(offset)
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
        return Err(AppError::Forbidden(
            "Нельзя снять роль администратора с себя",
        ));
    }

    let target: Option<String> =
        sqlx::query_scalar("UPDATE users SET role = $2 WHERE id = $1 RETURNING username")
            .bind(input.user_id)
            .bind(role.as_str())
            .fetch_optional(state.db.pool())
            .await?;
    let target = target.ok_or_else(|| AppError::NotFound("Пользователь не найден"))?;

    // The role is cached with the session, so the change only takes effect
    // after a re-login. Forcing it here also stops a demoted admin from
    // finishing whatever they had open with admin rights.
    let revoked = revoke_sessions(&state, input.user_id).await?;

    audit::log(
        state.db.pool(),
        auth.user_id,
        &auth.username,
        "set_role",
        &target,
        role.as_str(),
    )
    .await?;
    Ok(Json(
        serde_json::json!({"ok": true, "role": role.as_str(), "sessions_revoked": revoked}),
    ))
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

    // Sessions cascade with the user row, but the in-memory cache would keep
    // answering for them until its TTL runs out.
    state.session_cache.invalidate_user(input.user_id);

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
    let reason = input.reason.as_deref().unwrap_or("");

    // A ban keyed by username is a ban that matches nothing: the session check
    // compares against `users.id`. Resolve first, then store the id.
    let (identifier, user_id) = if kind == "user" {
        let id = resolve_user_id(&state, &input.identifier).await?;
        if id == auth.user_id {
            return Err(AppError::Forbidden("Нельзя заблокировать себя"));
        }
        (id.to_string(), Some(id))
    } else {
        let ip: std::net::IpAddr = input.identifier.trim().parse().map_err(|_| {
            AppError::Validation(ValidationError::new("identifier", "Неверный IP-адрес"))
        })?;
        (ip.to_string(), None)
    };

    sqlx::query(
        "INSERT INTO bans (identifier, kind, reason, issued_by, expires_at)
         VALUES ($1, $2, $3, $4, NOW() + make_interval(mins => $5))
         ON CONFLICT (identifier, kind) DO UPDATE SET
            reason = EXCLUDED.reason, expires_at = EXCLUDED.expires_at",
    )
    .bind(&identifier)
    .bind(kind)
    .bind(reason)
    .bind(auth.user_id)
    .bind(duration)
    .execute(state.db.pool())
    .await?;

    let mut revoked = 0;
    let mut kicked = 0;
    if let Some(user_id) = user_id {
        revoked = revoke_sessions(&state, user_id).await?;
        kicked = kick_from_rooms(&state, user_id, "Аккаунт заблокирован администратором");
    }

    audit::log(
        state.db.pool(),
        auth.user_id,
        &auth.username,
        "ban",
        &identifier,
        reason,
    )
    .await?;
    Ok(Json(
        serde_json::json!({"ok": true, "sessions_revoked": revoked, "kicked": kicked}),
    ))
}

async fn unban_user(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Json(input): Json<BanInput>,
) -> AppResult<impl axum::response::IntoResponse> {
    let kind = input.kind.as_deref().unwrap_or("user");
    // The ban row is keyed by id, so a username has to be resolved first —
    // otherwise "unban" silently matches nothing and the user stays locked out.
    let identifier = if kind == "user" {
        resolve_user_id(&state, &input.identifier)
            .await?
            .to_string()
    } else {
        input.identifier.clone()
    };
    sqlx::query("DELETE FROM bans WHERE identifier = $1 AND kind = $2")
        .bind(&identifier)
        .bind(kind)
        .execute(state.db.pool())
        .await?;
    audit::log(
        state.db.pool(),
        auth.user_id,
        &auth.username,
        "unban",
        &identifier,
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
        // Mark first: the sockets keep their `Arc` after the registry entry is
        // gone, and the read loop refuses further messages once `closed` is set.
        let mut room = room.write();
        room.closed = true;
        let _ = room.tx.send(
            serde_json::json!({
                "type": "error",
                "data": {"message": "Комната закрыта администратором", "code": "room_closed"}
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
struct KickInput {
    code: String,
    /// Numeric id of the account to remove. Guests have no id, so a name works
    /// too — the same rule the cheater powers use.
    user_id: Option<i64>,
    username: Option<String>,
    reason: Option<String>,
}

async fn kick_player(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Json(input): Json<KickInput>,
) -> AppResult<impl axum::response::IntoResponse> {
    let code = input.code.to_uppercase();
    let Some(room) = state.rooms.get_arc(&code) else {
        return Err(AppError::NotFound("Комната не найдена"));
    };

    let needle = input.username.as_deref().map(|name| name.to_lowercase());
    let (name, sid) = {
        let room = room.read();
        let found = room.players.values().find(|player| {
            input.user_id.is_some_and(|id| player.user_id == Some(id))
                || needle
                    .as_deref()
                    .is_some_and(|name| player.display_name.to_lowercase() == name)
        });
        match found {
            Some(player) => (player.display_name.clone(), player.sid.clone()),
            None => return Err(AppError::NotFound("Игрок не в комнате")),
        }
    };

    let reason = input
        .reason
        .as_deref()
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .unwrap_or("Вас удалили из комнаты");
    {
        let mut room = room.write();
        if let Some(player) = room.players.remove(&sid) {
            if let Some(outbox) = player.outbox {
                let _ = outbox.send(
                    serde_json::json!({"type": "kicked", "data": {"message": reason}}).to_string(),
                );
            }
        }
        let event = crate::ws::players_event(&room);
        room.touch();
        let _ = room.tx.send(event);
    }

    audit::log(
        state.db.pool(),
        auth.user_id,
        &auth.username,
        "kick",
        &format!("{code}/{name}"),
        reason,
    )
    .await?;
    Ok(Json(serde_json::json!({"ok": true, "player": name})))
}

async fn room_detail(
    State(state): State<AppState>,
    Query(query): Query<RoomRef>,
) -> AppResult<impl axum::response::IntoResponse> {
    let code = query.code.to_uppercase();
    let Some(room) = state.rooms.get_arc(&code) else {
        return Err(AppError::NotFound("Комната не найдена"));
    };
    let room = room.read();

    let mut players: Vec<serde_json::Value> = room
        .players
        .values()
        .map(|player| {
            serde_json::json!({
                "sid": player.sid,
                "user_id": player.user_id,
                "name": player.display_name,
                "score": player.score,
                "lives": player.lives,
                "team": player.team,
                "spectator": player.is_spectator,
                "connected": !player.is_disconnected(),
                "muted": player.muted,
                "jinxed": player.jinxed,
                "hijacked": player.hijacked,
                "blind_seconds": player.blind_until.map(|until| {
                    until.saturating_duration_since(std::time::Instant::now()).as_secs()
                }),
                "cheat_left": player.is_cheat.then(|| player.cheat.to_json()),
            })
        })
        .collect();
    players.sort_by_key(|player| std::cmp::Reverse(player["score"].as_i64().unwrap_or(0)));

    Ok(Json(serde_json::json!({
        "code": room.code,
        "state": room.state,
        "host": room.host_name,
        "host_user_id": room.host_user_id,
        "topic": room.settings.topic,
        "mode": room.settings.mode,
        "difficulty": room.settings.difficulty,
        "question_count": room.questions.len(),
        "current_question": room.current_q,
        "seconds_left": room.deadline.map(|deadline| {
            deadline.saturating_duration_since(std::time::Instant::now()).as_secs()
        }),
        "chat_locked": room.chat_locked,
        "chat_messages": room.chat.len(),
        "question_source": room.question_source,
        "players": players,
    })))
}

#[derive(Deserialize)]
struct AnnounceInput {
    code: String,
    message: String,
}

async fn room_announce(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Json(input): Json<AnnounceInput>,
) -> AppResult<impl axum::response::IntoResponse> {
    let text = input.message.trim().chars().take(200).collect::<String>();
    if text.is_empty() {
        return Err(AppError::Validation(ValidationError::new(
            "message",
            "Пустое сообщение",
        )));
    }
    let code = input.code.to_uppercase();
    let Some(room) = state.rooms.get_arc(&code) else {
        return Err(AppError::NotFound("Комната не найдена"));
    };
    let delivered = room
        .read()
        .tx
        .send(serde_json::json!({"type": "announcement", "data": {"message": text}}).to_string())
        .is_ok();
    audit::log(
        state.db.pool(),
        auth.user_id,
        &auth.username,
        "room_announce",
        &code,
        &text,
    )
    .await?;
    Ok(Json(
        serde_json::json!({"ok": true, "delivered": delivered}),
    ))
}

#[derive(Deserialize)]
struct ChatLockInput {
    code: String,
    locked: bool,
}

async fn set_chat_lock(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Json(input): Json<ChatLockInput>,
) -> AppResult<impl axum::response::IntoResponse> {
    let code = input.code.to_uppercase();
    let Some(room) = state.rooms.get_arc(&code) else {
        return Err(AppError::NotFound("Комната не найдена"));
    };
    {
        let mut room = room.write();
        room.chat_locked = input.locked;
        room.touch();
        let _ = room.tx.send(
            serde_json::json!({
                "type": "chat_locked",
                "data": {"locked": input.locked}
            })
            .to_string(),
        );
    }
    audit::log(
        state.db.pool(),
        auth.user_id,
        &auth.username,
        if input.locked {
            "chat_lock"
        } else {
            "chat_unlock"
        },
        &code,
        "",
    )
    .await?;
    Ok(Json(
        serde_json::json!({"ok": true, "locked": input.locked}),
    ))
}

async fn clear_chat(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Json(input): Json<RoomRef>,
) -> AppResult<impl axum::response::IntoResponse> {
    let code = input.code.to_uppercase();
    let Some(room) = state.rooms.get_arc(&code) else {
        return Err(AppError::NotFound("Комната не найдена"));
    };
    {
        let mut room = room.write();
        room.chat.clear();
        room.touch();
        let _ = room
            .tx
            .send(serde_json::json!({"type": "chat_cleared", "data": {}}).to_string());
    }
    audit::log(
        state.db.pool(),
        auth.user_id,
        &auth.username,
        "chat_clear",
        &code,
        "",
    )
    .await?;
    Ok(Json(serde_json::json!({"ok": true})))
}

// ── Sessions ────────────────────────────────────────────────

#[derive(Deserialize)]
struct SessionQuery {
    user_id: Option<i64>,
}

async fn list_sessions(
    State(state): State<AppState>,
    Query(query): Query<SessionQuery>,
) -> AppResult<impl axum::response::IntoResponse> {
    // The session token is a credential, so the panel only ever sees a hash
    // prefix as a row id.
    let rows = sqlx::query_as::<
        _,
        (
            String,
            i64,
            String,
            chrono::DateTime<chrono::Utc>,
            chrono::DateTime<chrono::Utc>,
            Option<chrono::DateTime<chrono::Utc>>,
            Option<String>,
        ),
    >(
        "SELECT left(md5(s.token), 12), s.user_id, u.username, s.created_at, s.expires_at,
                s.revoked_at, host(s.ip_address)
         FROM sessions s
         JOIN users u ON u.id = s.user_id
         WHERE ($1::BIGINT IS NULL OR s.user_id = $1)
         ORDER BY s.created_at DESC
         LIMIT 200",
    )
    .bind(query.user_id)
    .fetch_all(state.db.pool())
    .await?;

    let sessions: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "id": r.0,
                "user_id": r.1,
                "username": r.2,
                "created_at": r.3,
                "expires_at": r.4,
                "revoked_at": r.5,
                "ip": r.6,
            })
        })
        .collect();
    Ok(Json(sessions))
}

#[derive(Deserialize)]
struct SessionRevokeInput {
    user_id: Option<i64>,
    token: Option<String>,
}

async fn revoke_session(
    Extension(auth): Extension<AuthContext>,
    State(state): State<AppState>,
    Json(input): Json<SessionRevokeInput>,
) -> AppResult<impl axum::response::IntoResponse> {
    if let Some(token) = input.token.as_deref().filter(|token| !token.is_empty()) {
        state.session_cache.invalidate(token);
        sqlx::query(
            "UPDATE sessions SET revoked_at = NOW() WHERE token = $1 AND revoked_at IS NULL",
        )
        .bind(token)
        .execute(state.db.pool())
        .await?;
        audit::log(
            state.db.pool(),
            auth.user_id,
            &auth.username,
            "revoke_session",
            "token",
            "",
        )
        .await?;
        return Ok(Json(serde_json::json!({"ok": true, "revoked": 1})));
    }

    let user_id = input.user_id.ok_or_else(|| {
        AppError::Validation(ValidationError::new("user_id", "Нужен user_id или token"))
    })?;
    let revoked = revoke_sessions(&state, user_id).await?;
    audit::log(
        state.db.pool(),
        auth.user_id,
        &auth.username,
        "revoke_sessions",
        &user_id.to_string(),
        "",
    )
    .await?;
    Ok(Json(serde_json::json!({"ok": true, "revoked": revoked})))
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

#[derive(Deserialize)]
struct AuditQuery {
    action: Option<String>,
    limit: Option<i64>,
    offset: Option<i64>,
}

async fn audit_log(
    State(state): State<AppState>,
    Query(query): Query<AuditQuery>,
) -> AppResult<impl axum::response::IntoResponse> {
    let action = query
        .action
        .map(|action| action.trim().to_string())
        .filter(|action| !action.is_empty());
    let limit = query.limit.unwrap_or(100).clamp(1, 500);
    let offset = query.offset.unwrap_or(0).max(0);

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
         FROM audit_log
         WHERE ($1::TEXT IS NULL OR action = $1)
         ORDER BY created_at DESC LIMIT $2 OFFSET $3",
    )
    .bind(&action)
    .bind(limit)
    .bind(offset)
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
    Ok(Json(
        serde_json::json!({"ok": true, "provider": input.provider}),
    ))
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
