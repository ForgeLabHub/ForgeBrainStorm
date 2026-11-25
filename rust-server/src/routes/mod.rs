//! HTTP route handlers.

pub mod admin;
pub mod ai;
pub mod auth;
pub mod health;
pub mod leaderboard;
pub mod profile;
pub mod rooms;

use axum::Router;

use crate::AppState;

/// Build the application router with all routes and middleware.
pub fn build_router(state: AppState) -> Router<AppState> {
    Router::new()
        .merge(health::routes())
        .nest("/api/auth", auth::routes(state.session_cache.clone()))
        .nest("/api/admin", admin::routes(state.session_cache.clone()))
        .nest("/api/leaderboard", leaderboard::routes())
        .nest("/api/rooms", rooms::routes())
        .nest("/api/ai", ai::routes())
        .nest("/api/profile", profile::routes())
}
