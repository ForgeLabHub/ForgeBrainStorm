//! In-process token-bucket rate limiter.
//!
//! Per-identity (IP or session) sliding window. Multi-worker deployments
//! should swap this for a Redis-backed limiter without changing call sites.

use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::extract::{ConnectInfo, Request};
use axum::middleware::Next;
use axum::response::Response;
use dashmap::DashMap;

use crate::error::AppError;

#[derive(Clone)]
struct Bucket {
    tokens: f64,
    updated: Instant,
}

/// Token-bucket rate limiter, keyed by a string identity.
#[derive(Clone)]
pub struct RateLimiter {
    limit: f64,
    window: Duration,
    buckets: Arc<DashMap<String, Bucket>>,
    max_keys: usize,
}

impl RateLimiter {
    pub fn new(limit: u32, window_secs: u64) -> Self {
        Self {
            limit: limit as f64,
            window: Duration::from_secs(window_secs),
            buckets: Arc::new(DashMap::new()),
            max_keys: 10_000,
        }
    }

    /// Consume one token for `key`; returns `Err(AppError::RateLimited)` if exhausted.
    pub fn check(&self, key: &str) -> Result<(), AppError> {
        let now = Instant::now();
        self.evict_if_needed(now);

        let mut entry = self
            .buckets
            .entry(key.to_string())
            .or_insert_with(|| Bucket {
                tokens: self.limit,
                updated: now,
            });

        let elapsed = now.duration_since(entry.updated).as_secs_f64();
        let rate = self.limit / self.window.as_secs_f64();
        entry.tokens = (entry.tokens + elapsed * rate).min(self.limit);
        entry.updated = now;

        if entry.tokens < 1.0 {
            return Err(AppError::RateLimited);
        }
        entry.tokens -= 1.0;
        Ok(())
    }

    fn evict_if_needed(&self, now: Instant) {
        if self.buckets.len() <= self.max_keys {
            return;
        }
        let cutoff = now - self.window * 2;
        let keys_to_remove: Vec<String> = self
            .buckets
            .iter()
            .filter(|e| e.updated < cutoff)
            .map(|e| e.key().clone())
            .collect();
        for key in keys_to_remove {
            self.buckets.remove(&key);
        }
    }
}

/// Axum middleware that rate-limits by client IP.
pub async fn rate_limit_middleware(
    limiter: axum::extract::Extension<Arc<RateLimiter>>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let ip = request
        .extensions()
        .get::<ConnectInfo<std::net::SocketAddr>>()
        .map(|c| c.0.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    limiter.check(&ip)?;
    Ok(next.run(request).await)
}
