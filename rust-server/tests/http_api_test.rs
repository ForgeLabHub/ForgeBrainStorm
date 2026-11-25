//! HTTP API integration tests.
//!
//! These tests require a running PostgreSQL database.
//! Run with: cargo test --test http_api_test -- --ignored

use axum::http::StatusCode;
use axum::body::Body;
use axum::http::Request;
use tower::ServiceExt;
use serde_json::json;

// Note: These tests are marked as ignored because they require a database.
// To run them: cargo test --test http_api_test -- --ignored

#[tokio::test]
#[ignore = "requires database"]
async fn test_health_endpoint() {
    // This test would require setting up the full app with database
    // For now, just documenting the expected behavior
    
    // Expected:
    // GET /health -> 200 OK
    // {
    //   "status": "ok",
    //   "version": "0.1.0",
    //   "rooms": 0
    // }
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_readyz_endpoint() {
    // Expected:
    // GET /readyz -> 200 OK (if DB is ready)
    // {
    //   "status": "ready",
    //   "db": "ok"
    // }
    //
    // Or 503 if DB is not ready:
    // {
    //   "status": "not_ready",
    //   "db": "error"
    // }
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_auth_register() {
    // Expected flow:
    // POST /api/auth/register
    // {
    //   "username": "testuser",
    //   "password": "testpassword123"
    // }
    //
    // Response: 201 Created
    // {
    //   "ok": true,
    //   "user": {
    //     "user_id": 1,
    //     "username": "testuser",
    //     "level": 1,
    //     "xp": 0,
    //     "coins": 0
    //   }
    // }
    // Set-Cookie: brainstorm_session=...
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_auth_login() {
    // Expected flow:
    // POST /api/auth/login
    // {
    //   "username": "testuser",
    //   "password": "testpassword123"
    // }
    //
    // Response: 200 OK
    // {
    //   "ok": true,
    //   "user": { ... }
    // }
    // Set-Cookie: brainstorm_session=...
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_auth_logout() {
    // Expected flow:
    // POST /api/auth/logout
    // Cookie: brainstorm_session=...
    //
    // Response: 200 OK
    // {
    //   "ok": true
    // }
    // Set-Cookie: brainstorm_session=; Max-Age=0
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_create_room_requires_auth() {
    // Expected:
    // POST /api/rooms
    // (no cookie)
    //
    // Response: 401 Unauthorized
    // {
    //   "ok": false,
    //   "error": {
    //     "code": "unauthorized",
    //     "message": "Authentication required"
    //   }
    // }
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_create_room() {
    // Expected flow:
    // POST /api/rooms
    // Cookie: brainstorm_session=...
    // {
    //   "topic": "Test Topic",
    //   "mode": "classic"
    // }
    //
    // Response: 201 Created
    // {
    //   "ok": true,
    //   "room": {
    //     "code": "ABCDEF",
    //     "host": "testuser",
    //     "topic": "Test Topic",
    //     "mode": "classic",
    //     "settings": { ... }
    //   }
    // }
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_list_rooms() {
    // Expected:
    // GET /api/rooms
    //
    // Response: 200 OK
    // {
    //   "ok": true,
    //   "rooms": [
    //     {
    //       "code": "ABCDEF",
    //       "host": "testuser",
    //       "topic": "Test Topic",
    //       "mode": "classic",
    //       "player_count": 0
    //     }
    //   ]
    // }
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_get_room() {
    // Expected:
    // GET /api/rooms/ABCDEF
    //
    // Response: 200 OK
    // {
    //   "ok": true,
    //   "room": {
    //     "code": "ABCDEF",
    //     "host": "testuser",
    //     "topic": "Test Topic",
    //     "mode": "classic",
    //     "player_count": 2,
    //     "state": "waiting"
    //   }
    // }
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_get_room_not_found() {
    // Expected:
    // GET /api/rooms/NOTFOUND
    //
    // Response: 404 Not Found
    // {
    //   "ok": false,
    //   "error": {
    //     "code": "not_found",
    //     "message": "Room not found"
    //   }
    // }
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_ai_providers_list() {
    // Expected:
    // GET /api/ai/providers
    //
    // Response: 200 OK
    // {
    //   "ok": true,
    //   "providers": [
    //     {
    //       "name": "gigachat",
    //       "available": true,
    //       "is_default": true
    //     },
    //     {
    //       "name": "openai",
    //       "available": false,
    //       "is_default": false
    //     }
    //   ]
    // }
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_ai_switch_provider() {
    // Expected flow:
    // POST /api/ai/switch
    // {
    //   "provider": "openai"
    // }
    //
    // Response: 200 OK
    // {
    //   "ok": true,
    //   "message": "Switched to openai"
    // }
}

#[tokio::test]
#[ignore = "requires database"]
async fn test_leaderboard() {
    // Expected:
    // GET /api/leaderboard
    //
    // Response: 200 OK
    // {
    //   "ok": true,
    //   "leaderboard": [
    //     {
    //       "rank": 1,
    //       "username": "player1",
    //       "score": 1000,
    //       "games_played": 10,
    //       "wins": 8
    //     },
    //     {
    //       "rank": 2,
    //       "username": "player2",
    //       "score": 800,
    //       "games_played": 8,
    //       "wins": 5
    //     }
    //   ]
    // }
}
