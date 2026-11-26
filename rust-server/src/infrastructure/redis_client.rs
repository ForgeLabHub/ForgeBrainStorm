//! Redis client for distributed caching, pub/sub, and session management.
//!
//! Provides graceful degradation: if Redis is unavailable, operations
//! return errors that callers can handle (e.g., fall back to in-memory cache).

use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, Client, Commands};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Redis client wrapper with connection management.
#[derive(Clone)]
pub struct RedisClient {
    manager: Arc<RwLock<Option<ConnectionManager>>>,
    connected: Arc<RwLock<bool>>,
}

impl RedisClient {
    /// Create a new Redis client and attempt to connect.
    pub async fn new(redis_url: &str) -> Result<Self> {
        if redis_url.is_empty() {
            info!("Redis URL is empty, running in local-only mode");
            return Ok(Self {
                manager: Arc::new(RwLock::new(None)),
                connected: Arc::new(RwLock::new(false)),
            });
        }

        let client = Client::open(redis_url)
            .with_context(|| format!("Failed to create Redis client for {}", redis_url))?;

        let manager = ConnectionManager::new(client)
            .await
            .with_context(|| "Failed to create Redis connection manager")?;

        info!("Connected to Redis at {}", redis_url);

        Ok(Self {
            manager: Arc::new(RwLock::new(Some(manager))),
            connected: Arc::new(RwLock::new(true)),
        })
    }

    /// Check if Redis is connected.
    pub async fn is_connected(&self) -> bool {
        *self.connected.read().await
    }

    /// Set a key-value pair with optional TTL.
    pub async fn set(&self, key: &str, value: &str, ttl_secs: Option<u64>) -> Result<()> {
        let manager_guard = self.manager.read().await;
        let mut conn = manager_guard
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Redis not connected"))?
            .clone();

        if let Some(ttl) = ttl_secs {
            conn.set_ex(key, value, ttl)
                .await
                .with_context(|| format!("Failed to SET {} with TTL", key))?;
        } else {
            conn.set(key, value)
                .await
                .with_context(|| format!("Failed to SET {}", key))?;
        }

        debug!("Redis SET {} (TTL: {:?})", key, ttl_secs);
        Ok(())
    }

    /// Get a value by key.
    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        let manager_guard = self.manager.read().await;
        let mut conn = manager_guard
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Redis not connected"))?
            .clone();

        let value: Option<String> = conn
            .get(key)
            .await
            .with_context(|| format!("Failed to GET {}", key))?;

        debug!("Redis GET {} = {:?}", key, value.is_some());
        Ok(value)
    }

    /// Delete a key.
    pub async fn del(&self, key: &str) -> Result<()> {
        let manager_guard = self.manager.read().await;
        let mut conn = manager_guard
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Redis not connected"))?
            .clone();

        conn.del(key)
            .await
            .with_context(|| format!("Failed to DEL {}", key))?;

        debug!("Redis DEL {}", key);
        Ok(())
    }

    /// Publish a message to a channel.
    pub async fn publish(&self, channel: &str, message: &str) -> Result<()> {
        let manager_guard = self.manager.read().await;
        let mut conn = manager_guard
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Redis not connected"))?
            .clone();

        conn.publish(channel, message)
            .await
            .with_context(|| format!("Failed to PUBLISH to {}", channel))?;

        debug!("Redis PUBLISH {} ({} bytes)", channel, message.len());
        Ok(())
    }

    /// Subscribe to a channel and process messages.
    ///
    /// This blocks until the subscription is closed or an error occurs.
    /// The callback is called for each received message.
    pub async fn subscribe<F, Fut>(
        &self,
        channel: &str,
        mut callback: F,
    ) -> Result<()>
    where
        F: FnMut(String) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = Result<()>> + Send,
    {
        let manager_guard = self.manager.read().await;
        let conn = manager_guard
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Redis not connected"))?
            .clone();
        drop(manager_guard);

        let mut pubsub = conn.into_pubsub();
        pubsub
            .subscribe(channel)
            .await
            .with_context(|| format!("Failed to SUBSCRIBE to {}", channel))?;

        info!("Subscribed to Redis channel: {}", channel);

        let mut on_message = pubsub.on_message();
        while let Some(msg) = tokio_stream::StreamExt::next(&mut on_message).await {
            let payload: String = msg
                .get_payload()
                .with_context(|| "Failed to get message payload")?;

            debug!("Redis SUBSCRIBE {} received {} bytes", channel, payload.len());

            if let Err(e) = callback(payload).await {
                error!("Error processing Redis message: {}", e);
            }
        }

        Ok(())
    }

    /// Increment a counter and return the new value.
    pub async fn incr(&self, key: &str) -> Result<i64> {
        let manager_guard = self.manager.read().await;
        let mut conn = manager_guard
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Redis not connected"))?
            .clone();

        let value: i64 = conn
            .incr(key, 1)
            .await
            .with_context(|| format!("Failed to INCR {}", key))?;

        debug!("Redis INCR {} = {}", key, value);
        Ok(value)
    }

    /// Check if a key exists.
    pub async fn exists(&self, key: &str) -> Result<bool> {
        let manager_guard = self.manager.read().await;
        let mut conn = manager_guard
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Redis not connected"))?
            .clone();

        let exists: bool = conn
            .exists(key)
            .await
            .with_context(|| format!("Failed to EXISTS {}", key))?;

        Ok(exists)
    }

    /// Set a key with expiration only if it doesn't exist (atomic).
    pub async fn set_nx(&self, key: &str, value: &str, ttl_secs: u64) -> Result<bool> {
        let manager_guard = self.manager.read().await;
        let mut conn = manager_guard
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Redis not connected"))?
            .clone();

        let result: bool = redis::cmd("SET")
            .arg(key)
            .arg(value)
            .arg("NX")
            .arg("EX")
            .arg(ttl_secs)
            .query_async(&mut *conn)
            .await
            .with_context(|| format!("Failed to SETNX {}", key))?;

        debug!("Redis SETNX {} = {} (TTL: {})", key, result, ttl_secs);
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "requires Redis server"]
    async fn test_redis_connection() {
        let client = RedisClient::new("redis://127.0.0.1:6379")
            .await
            .expect("Failed to connect to Redis");

        assert!(client.is_connected().await);
    }

    #[tokio::test]
    #[ignore = "requires Redis server"]
    async fn test_redis_set_get() {
        let client = RedisClient::new("redis://127.0.0.1:6379")
            .await
            .expect("Failed to connect to Redis");

        client.set("test:key", "test_value", Some(10)).await.unwrap();
        let value = client.get("test:key").await.unwrap();
        assert_eq!(value, Some("test_value".to_string()));

        client.del("test:key").await.unwrap();
        let value = client.get("test:key").await.unwrap();
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_empty_redis_url() {
        let client = RedisClient::new("")
            .await
            .expect("Failed to create client with empty URL");

        assert!(!client.is_connected().await);
    }
}
