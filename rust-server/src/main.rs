//! BrainStorm Rust server entry point.
//!
//! Boots the Axum HTTP/WS server with Postgres, runs migrations, and
//! serves the application. Configuration is loaded from `config/default.toml`
//! and environment variables.

mod config;
mod db;
mod domain;
mod error;
mod middleware;
mod routes;
mod security;
mod services;
mod ws;

use std::sync::Arc;
use std::time::Duration;

use axum::extract::connect_info::IntoMakeServiceWithConnectInfo;
use axum::routing::get;
use axum::Router;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, fmt};

use crate::config::Settings;
use crate::domain::room::RoomRegistry;
use crate::middleware::auth::SessionCache;
use crate::middleware::rate_limit::RateLimiter;
use crate::services::ai_client::AiClient;

/// Shared application state passed to all handlers.
#[derive(Clone)]
pub struct AppState {
    pub settings: Arc<Settings>,
    pub db: DbHandle,
    pub rooms: RoomRegistry,
    pub ai: AiClient,
    pub session_cache: SessionCache,
}

/// Database handle wrapper.
#[derive(Clone)]
pub struct DbHandle {
    pool: sqlx::PgPool,
}

impl DbHandle {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &sqlx::PgPool {
        &self.pool
    }

    pub async fn health_check(&self) -> error::AppResult<()> {
        db::health_check(&self.pool).await
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialise tracing (JSON format for structured logging).
    fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("brainstorm=info")),
        )
        .json()
        .init();

    info!("loading configuration");
    let settings = Settings::load().unwrap_or_else(|e| {
        eprintln!("ERROR: configuration failed to load: {e}");
        std::process::exit(1);
    });
    info!(env = ?settings.env, listen = %settings.listen, "configuration loaded");

    // Initialise database pool + run migrations.
    info!("connecting to database");
    let pool = db::init_pool(&settings).await?;

    // Build shared state.
    let session_cache = SessionCache::new(Duration::from_secs(300), pool.clone());
    let rooms = RoomRegistry::new(settings.max_rooms);
    let ai = AiClient::new(&settings, pool.clone());

    let state = AppState {
        settings: Arc::new(settings.clone()),
        db: DbHandle::new(pool),
        rooms: rooms.clone(),
        ai,
        session_cache: session_cache.clone(),
    };

    // Build router.
    let cors = middleware::cors::build_cors_layer(&settings);
    let rate_limiter = Arc::new(RateLimiter::new(
        settings.auth_rate_limit,
        settings.auth_rate_window,
    ));

    let app = Router::new()
        .merge(routes::build_router(state.clone()))
        .route("/ws", get(ws::ws_handler))
        .layer(cors)
        .layer(axum::Extension(rate_limiter))
        .layer(axum::Extension(session_cache))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Background: periodic room cleanup.
    let cleanup_rooms = rooms.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            let removed = cleanup_rooms.cleanup_stale(Duration::from_secs(300));
            if removed > 0 {
                info!(removed, "cleaned up stale rooms");
            }
        }
    });

    // Start server.
    info!(addr = %settings.listen, "starting HTTP server");
    let listener = TcpListener::bind(settings.listen).await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await?;

    Ok(())
}
