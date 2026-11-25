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
use crate::error::AppError;
use crate::middleware::auth::{SessionCache, SESSION_COOKIE};
use crate::middleware::rate_limit::RateLimiter;
use crate::services::ai::router::AiRouter;
use crate::services::ai::{AiCapability, ProviderConfig, ProviderType};

/// Shared application state passed to all handlers.
#[derive(Clone)]
pub struct AppState {
    pub settings: Arc<Settings>,
    pub db: DbHandle,
    pub rooms: RoomRegistry,
    pub ai_router: Arc<AiRouter>,
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

    // Load .env file if it exists (supports ../.env for monorepo setup)
    if let Err(e) = dotenvy::from_filename("../.env") {
        // Not an error — .env is optional
        tracing::debug!(error = %e, "No .env file in parent directory");
    }
    if let Err(e) = dotenvy::dotenv() {
        tracing::debug!(error = %e, "No .env file in current directory");
    }

    info!("loading configuration");
    let settings = Settings::load().unwrap_or_else(|e| {
        eprintln!("ERROR: configuration failed to load: {e}");
        std::process::exit(1);
    });
    info!(env = ?settings.env, listen = %settings.listen, "configuration loaded");

    // Initialise database pool + run migrations.
    info!("connecting to database");
    let pool = db::init_pool(&settings).await?;

    // Build AI router with multiple providers.
    info!("initializing AI router");
    let ai_configs: Vec<ProviderConfig> = settings
        .ai_providers
        .iter()
        .map(|c| {
            let provider_type = match c.provider_type.to_lowercase().as_str() {
                "gigachat" => ProviderType::GigaChat,
                "openai" => ProviderType::OpenAI,
                "ollama" => ProviderType::Ollama,
                "anthropic" => ProviderType::Anthropic,
                _ => ProviderType::Custom,
            };

            ProviderConfig {
                name: c.name.clone(),
                provider_type,
                api_key: c.api_key.clone(),
                api_endpoint: c.api_endpoint.clone(),
                model: c.model.clone(),
                max_concurrent: c.max_concurrent,
                timeout_secs: c.timeout_secs,
                capabilities: vec![
                    AiCapability::QuestionGeneration,
                    AiCapability::HintGeneration,
                    AiCapability::ExplanationGeneration,
                ],
                cost_per_1k_tokens: 0.0,
                enabled: c.enabled,
            }
        })
        .collect();

    let ai_router = Arc::new(AiRouter::new(ai_configs));

    // Build shared state.
    let session_cache = SessionCache::new(Duration::from_secs(300), pool.clone());
    let rooms = RoomRegistry::new(settings.max_rooms);

    let state = AppState {
        settings: Arc::new(settings.clone()),
        db: DbHandle::new(pool),
        rooms: rooms.clone(),
        ai_router,
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
        .route(
            "/ws",
            get(ws::ws_handler).layer(axum::middleware::from_fn_with_state(
                state.clone(),
                ws_auth_middleware,
            )),
        )
        .layer(cors)
        .layer(axum::Extension(rate_limiter))
        .layer(axum::Extension(session_cache.clone()))
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

/// Middleware for WebSocket authentication.
/// Extracts session from cookie, validates it, and injects AuthContext.
async fn ws_auth_middleware(
    State(state): State<AppState>,
    mut req: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, AppError> {
    // Extract session cookie
    let cookie_header = req
        .headers()
        .get(axum::http::header::COOKIE)
        .and_then(|v| v.to_str().ok());

    let session_token = cookie_header
        .and_then(|cookies| {
            cookies.split(';').find_map(|cookie| {
                let cookie = cookie.trim();
                if cookie.starts_with(&format!("{}=", SESSION_COOKIE)) {
                    cookie.strip_prefix(&format!("{}=", SESSION_COOKIE))
                } else {
                    None
                }
            })
        })
        .ok_or_else(|| AppError::Unauthorized)?;

    // Validate session and get user context
    let auth_context = state.session_cache.get_or_load(session_token).await?;

    // Inject AuthContext into request extensions
    req.extensions_mut().insert(auth_context);

    Ok(next.run(req).await)
}
