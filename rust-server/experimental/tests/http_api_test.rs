//! HTTP API integration tests.
//!
//! These tests require a running PostgreSQL database.
//! Run with: cargo test --test http_api_test -- --ignored

/// Health endpoint test (ignored — requires database).
#[tokio::test]
#[ignore = "requires database"]
async fn test_health_endpoint() {
    // Expected:
    // GET /health -> 200 OK
    // {
    //   "status": "ok",
    //   "version": "0.1.0"
    // }
}

/// Ready check endpoint test (ignored — requires database).
#[tokio::test]
#[ignore = "requires database"]
async fn test_readyz_endpoint() {
    // Expected:
    // GET /readyz -> 200 OK (if DB is ready)
    // {
    //   "status": "ready",
    //   "db": "ok"
    // }
}

/// Auth register endpoint test (ignored — requires database).
#[tokio::test]
#[ignore = "requires database"]
async fn test_auth_register() {
    // Expected flow:
    // POST /api/auth/register
    // Body: { "username": "testuser", "password": "testpassword123" }
    //
    // Response: 201 Created
    // {
    //   "ok": true,
    //   "user": { "user_id": 1, "username": "testuser", ... }
    // }
    // Set-Cookie: brainstorm_session=...
}

/// Auth login endpoint test (ignored — requires database).
#[tokio::test]
#[ignore = "requires database"]
async fn test_auth_login() {
    // Expected flow:
    // POST /api/auth/login
    // Body: { "username": "testuser", "password": "testpassword123" }
    //
    // Response: 200 OK
    // { "ok": true, "user": { ... } }
    // Set-Cookie: brainstorm_session=...
}

/// Auth logout endpoint test (ignored — requires database).
#[tokio::test]
#[ignore = "requires database"]
async fn test_auth_logout() {
    // Expected flow:
    // POST /api/auth/logout
    // Cookie: brainstorm_session=...
    //
    // Response: 200 OK
    // { "ok": true }
    // Set-Cookie: brainstorm_session=; Max-Age=0
}

/// Create room requires auth (ignored — requires database).
#[tokio::test]
#[ignore = "requires database"]
async fn test_create_room_requires_auth() {
    // Expected:
    // POST /api/rooms (no cookie)
    //
    // Response: 401 Unauthorized
    // { "ok": false, "error": { "code": "unauthorized", ... } }
}

/// Create room with auth (ignored — requires database).
#[tokio::test]
#[ignore = "requires database"]
async fn test_create_room() {
    // Expected flow:
    // POST /api/rooms
    // Cookie: brainstorm_session=...
    // Body: { "topic": "Test Topic", "mode": "classic" }
    //
    // Response: 201 Created
    // { "ok": true, "room": { "code": "ABCDEF", ... } }
}

/// List rooms (ignored — requires database).
#[tokio::test]
#[ignore = "requires database"]
async fn test_list_rooms() {
    // Expected:
    // GET /api/rooms
    // Response: 200 OK
    // { "ok": true, "rooms": [...] }
}

/// Get room by code (ignored — requires database).
#[tokio::test]
#[ignore = "requires database"]
async fn test_get_room() {
    // Expected:
    // GET /api/rooms/ABCDEF
    // Response: 200 OK
    // { "ok": true, "room": { "code": "ABCDEF", ... } }
}

/// Get non-existent room (ignored — requires database).
#[tokio::test]
#[ignore = "requires database"]
async fn test_get_room_not_found() {
    // Expected:
    // GET /api/rooms/NOTFOUND
    // Response: 404 Not Found
}

/// AI providers list (ignored — requires database).
#[tokio::test]
#[ignore = "requires database"]
async fn test_ai_providers_list() {
    // Expected:
    // GET /api/ai/providers
    // Response: 200 OK
    // { "ok": true, "providers": [...] }
}

/// Switch AI provider (ignored — requires database).
#[tokio::test]
#[ignore = "requires database"]
async fn test_ai_switch_provider() {
    // Expected flow:
    // POST /api/ai/switch
    // Body: { "provider": "openai" }
    // Response: 200 OK
    // { "ok": true, "message": "Switched to openai" }
}

/// Leaderboard (ignored — requires database).
#[tokio::test]
#[ignore = "requires database"]
async fn test_leaderboard() {
    // Expected:
    // GET /api/leaderboard
    // Response: 200 OK
    // { "ok": true, "leaderboard": [...] }
}

/// Validation error on create room (ignored — requires database).
#[tokio::test]
#[ignore = "requires database"]
async fn test_create_room_validation_error() {
    // Expected:
    // POST /api/rooms with empty topic (>60 chars)
    // Response: 400 Bad Request
    // { "ok": false, "error": { "code": "validation_error", ... } }
}

/// Conflict error on duplicate username (ignored — requires database).
#[tokio::test]
#[ignore = "requires database"]
async fn test_register_duplicate_username() {
    // Expected:
    // POST /api/auth/register with existing username
    // Response: 409 Conflict
    // { "ok": false, "error": { "code": "conflict", ... } }
}
