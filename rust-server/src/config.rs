//! Application configuration.
//!
//! Loaded from `config/default.toml` (optional), environment variables,
//! and validated at startup. Refuses to boot in production with insecure
//! defaults.

use std::net::SocketAddr;
use std::time::Duration;

use figment::{Figment, providers::{Env, Format, Toml}};
use serde::Deserialize;

/// Top-level application configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    /// `development` or `production`.
    pub env: Environment,

    /// Bind address for the HTTP/WS server.
    #[serde(default = "default_listen_addr")]
    pub listen: SocketAddr,

    /// Secret used for HMAC signatures (sessions, rejoin tokens, CSRF).
    /// Must be ≥ 32 bytes in production.
    pub secret_key: String,

    /// Admin activation key (compared in constant time).
    #[serde(default)]
    pub admin_secret_key: String,

    /// Database URL (postgres).
    pub database_url: String,

    /// Maximum number of pooled database connections.
    #[serde(default = "default_db_pool_size")]
    pub db_pool_size: u32,

    /// Multi-AI provider configurations.
    #[serde(default)]
    pub ai_providers: Vec<AiProviderConfig>,

    /// Default AI provider name.
    #[serde(default)]
    pub default_ai_provider: String,

    /// CORS allowed origins (empty = deny all; `*` is forbidden in production).
    #[serde(default)]
    pub cors_origins: Vec<String>,

    /// Whether to trust `X-Forwarded-*` headers from upstream proxies.
    #[serde(default)]
    pub trust_proxy: bool,

    /// Session cookie security flag (must be true in production).
    #[serde(default = "default_true")]
    pub session_cookie_secure: bool,

    /// Session TTL in seconds.
    #[serde(default = "default_session_ttl")]
    pub session_ttl_secs: u64,

    /// Redis URL (for rate-limit / session store / cache). Optional in dev.
    #[serde(default)]
    pub redis_url: String,

    /// Enable HTTP rate limiting.
    #[serde(default = "default_true")]
    pub rate_limit_enabled: bool,

    /// Auth rate limit: requests per window.
    #[serde(default = "default_auth_rate_limit")]
    pub auth_rate_limit: u32,

    /// Auth rate limit window in seconds.
    #[serde(default = "default_auth_rate_window")]
    pub auth_rate_window: u64,

    /// Max concurrent rooms.
    #[serde(default = "default_max_rooms")]
    pub max_rooms: usize,

    /// Max players per room.
    #[serde(default = "default_max_players")]
    pub max_players: usize,

    /// Disconnect grace period in seconds.
    #[serde(default = "default_grace_period")]
    pub disconnect_grace_secs: u64,
}

/// AI provider configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct AiProviderConfig {
    pub name: String,
    pub provider_type: String,
    pub api_key: Option<String>,
    pub api_endpoint: Option<String>,
    pub model: String,
    pub max_concurrent: usize,
    pub timeout_secs: u64,
    pub enabled: bool,
}

impl Default for AiProviderConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            provider_type: String::new(),
            api_key: None,
            api_endpoint: None,
            model: String::new(),
            max_concurrent: 4,
            timeout_secs: 30,
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Development,
    Production,
}

impl Environment {
    pub fn is_production(self) -> bool {
        self == Self::Production
    }
}

fn default_listen_addr() -> SocketAddr {
    "0.0.0.0:5000".parse().expect("valid default socket addr")
}

fn default_db_pool_size() -> u32 { 20 }
fn default_true() -> bool { true }
fn default_session_ttl() -> u64 { 86_400 }
fn default_auth_rate_limit() -> u32 { 10 }
fn default_auth_rate_window() -> u64 { 60 }
fn default_max_rooms() -> usize { 200 }
fn default_max_players() -> usize { 30 }
fn default_grace_period() -> u64 { 30 }

impl Settings {
    /// Load configuration from defaults, optional TOML file, and environment.
    pub fn load() -> Result<Self, ConfigError> {
        let figment = Figment::new()
            .merge(Toml::file("config/default.toml"))
            .merge(Env::prefixed("BRAINSTORM_").split("__"));

        let mut settings: Self = figment.extract()?;

        // Apply PORT → listen addr override if only PORT is set.
        if let Ok(port) = std::env::var("PORT") {
            if let Ok(port) = port.parse::<u16>() {
                settings.listen.set_port(port);
            }
        }

        settings.validate()?;
        Ok(settings)
    }

    /// Validate invariants; refuse to boot with dangerous defaults in production.
    fn validate(&self) -> Result<(), ConfigError> {
        if self.secret_key.is_empty() {
            return Err(ConfigError::Missing("SECRET_KEY"));
        }
        if self.env.is_production() {
            if self.secret_key.len() < 32 {
                return Err(ConfigError::Insecure(
                    "SECRET_KEY must be at least 32 bytes in production",
                ));
            }
            if self.secret_key == "dev-secret-change-me" {
                return Err(ConfigError::Insecure(
                    "SECRET_KEY is still the development default in production",
                ));
            }
            if self.cors_origins.iter().any(|o| o == "*") {
                return Err(ConfigError::Insecure(
                    "CORS_ORIGINS must not contain '*' in production",
                ));
            }
            if !self.session_cookie_secure {
                return Err(ConfigError::Insecure(
                    "SESSION_COOKIE_SECURE must be true in production",
                ));
            }
            if self.admin_secret_key.is_empty() {
                return Err(ConfigError::Missing("ADMIN_SECRET_KEY"));
            }
        }
        if self.database_url.is_empty() {
            return Err(ConfigError::Missing("DATABASE_URL"));
        }
        Ok(())
    }

    pub fn session_ttl(&self) -> Duration {
        Duration::from_secs(self.session_ttl_secs)
    }

    pub fn disconnect_grace(&self) -> Duration {
        Duration::from_secs(self.disconnect_grace_secs)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("configuration error: {0}")]
    Figment(#[from] figment::Error),
    #[error("missing required configuration: {0}")]
    Missing(&'static str),
    #[error("insecure configuration: {0}")]
    Insecure(&'static str),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_settings() -> Settings {
        Settings {
            env: Environment::Development,
            listen: default_listen_addr(),
            secret_key: "test-secret-at-least-32-bytes-long-ok".into(),
            admin_secret_key: String::new(),
            database_url: "postgres://localhost/brainstorm".into(),
            db_pool_size: 5,
            gigachat_credentials: String::new(),
            gigachat_scope: default_gigachat_scope(),
            gigachat_model: default_gigachat_model(),
            ai_concurrency: 4,
            cors_origins: vec!["*".into()],
            trust_proxy: false,
            session_cookie_secure: true,
            session_ttl_secs: default_session_ttl(),
            redis_url: String::new(),
            rate_limit_enabled: true,
            auth_rate_limit: 10,
            auth_rate_window: 60,
            max_rooms: 200,
            max_players: 30,
            disconnect_grace_secs: 30,
        }
    }

    #[test]
    fn dev_accepts_weak_defaults() {
        let s = minimal_settings();
        assert!(s.validate().is_ok());
    }

    #[test]
    fn prod_rejects_short_secret() {
        let mut s = minimal_settings();
        s.env = Environment::Production;
        s.secret_key = "short".into();
        assert!(matches!(s.validate(), Err(ConfigError::Insecure(_))));
    }

    #[test]
    fn prod_rejects_wildcard_cors() {
        let mut s = minimal_settings();
        s.env = Environment::Production;
        s.admin_secret_key = "admin-secret".into();
        assert!(matches!(s.validate(), Err(ConfigError::Insecure(_))));
    }

    #[test]
    fn rejects_missing_database() {
        let mut s = minimal_settings();
        s.database_url = String::new();
        assert!(matches!(s.validate(), Err(ConfigError::Missing(_))));
    }
}
