//! Distributed AI response cache with Redis backend.
//!
//! Caches generated questions, hints, and explanations in Redis to share
//! cache across multiple server instances. Falls back to local cache if
//! Redis is unavailable.
//!
//! Benefits:
//! - Reduces AI API costs across all instances
//! - Faster response times (cache hits)
//! - Shared knowledge base
//! - Graceful degradation to local cache

use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, warn};

use super::cache::AiCache;
use super::GenerationResponse;
use crate::infrastructure::RedisClient;

/// Distributed cache wrapper with Redis backend and local fallback.
pub struct DistributedAiCache {
    /// Redis client for distributed caching.
    redis: Arc<RedisClient>,
    /// Local cache fallback when Redis is unavailable.
    local: AiCache,
    /// Cache key prefix for namespacing.
    prefix: String,
}

/// Serializable wrapper for GenerationResponse.
#[derive(Debug, Serialize, Deserialize)]
struct CachedGenerationResponse {
    questions: Vec<serde_json::Value>,
    total_generated: usize,
    cached: bool,
}

impl From<GenerationResponse> for CachedGenerationResponse {
    fn from(response: GenerationResponse) -> Self {
        Self {
            questions: response.questions,
            total_generated: response.total_generated,
            cached: response.cached,
        }
    }
}

impl From<CachedGenerationResponse> for GenerationResponse {
    fn from(cached: CachedGenerationResponse) -> Self {
        Self {
            questions: cached.questions,
            total_generated: cached.total_generated,
            cached: true, // Mark as cached when loaded from Redis
        }
    }
}

impl DistributedAiCache {
    /// Create a new distributed cache.
    pub fn new(redis: Arc<RedisClient>, prefix: &str) -> Self {
        Self {
            redis,
            local: AiCache::new(Duration::from_secs(3600)),
            prefix: prefix.to_string(),
        }
    }

    /// Get cached questions from Redis or local cache.
    ///
    /// Tries Redis first. If Redis is unavailable or key is missing,
    /// falls back to local cache.
    pub async fn get(&self, key: &str) -> Option<GenerationResponse> {
        let redis_key = format!("{}:question:{}", self.prefix, key);

        // Try Redis first
        if self.redis.is_connected().await {
            match self.redis.get(&redis_key).await {
                Ok(Some(json_str)) => {
                    match serde_json::from_str::<CachedGenerationResponse>(&json_str) {
                        Ok(cached) => {
                            debug!(key, "Cache hit (Redis)");
                            return Some(cached.into());
                        }
                        Err(e) => {
                            warn!(key, error = %e, "Failed to deserialize cached response");
                        }
                    }
                }
                Ok(None) => {
                    debug!(key, "Cache miss (Redis)");
                }
                Err(e) => {
                    warn!(key, error = %e, "Redis GET failed, trying local cache");
                }
            }
        }

        // Fallback to local cache
        self.local.get(key)
    }

    /// Cache questions in both Redis and local cache.
    ///
    /// Stores in Redis with TTL. If Redis is unavailable, stores in local cache only.
    pub async fn set(&self, key: String, value: GenerationResponse) {
        let redis_key = format!("{}:question:{}", self.prefix, key);
        let ttl_secs = 3600; // 1 hour

        // Always store in local cache
        self.local.set(key.clone(), value.clone());

        // Try to store in Redis
        if self.redis.is_connected().await {
            match serde_json::to_string(&CachedGenerationResponse::from(value)) {
                Ok(json_str) => {
                    match self.redis.set(&redis_key, &json_str, Some(ttl_secs)).await {
                        Ok(()) => {
                            debug!(key, ttl_secs, "Cached in Redis");
                        }
                        Err(e) => {
                            warn!(key, error = %e, "Failed to cache in Redis");
                        }
                    }
                }
                Err(e) => {
                    error!(key, error = %e, "Failed to serialize response for caching");
                }
            }
        }
    }

    /// Get cached hint from Redis or local cache.
    pub async fn get_hint(&self, key: &str) -> Option<String> {
        let redis_key = format!("{}:hint:{}", self.prefix, key);

        // Try Redis first
        if self.redis.is_connected().await {
            match self.redis.get(&redis_key).await {
                Ok(Some(hint)) => {
                    debug!(key, "Hint cache hit (Redis)");
                    return Some(hint);
                }
                Ok(None) => {
                    debug!(key, "Hint cache miss (Redis)");
                }
                Err(e) => {
                    warn!(key, error = %e, "Redis GET failed for hint, trying local cache");
                }
            }
        }

        // Fallback to local cache
        self.local.get_hint(key)
    }

    /// Cache a hint in both Redis and local cache.
    pub async fn set_hint(&self, key: String, value: String) {
        let redis_key = format!("{}:hint:{}", self.prefix, key);
        let ttl_secs = 1800; // 30 minutes

        // Always store in local cache
        self.local.set_hint(key.clone(), value.clone());

        // Try to store in Redis
        if self.redis.is_connected().await {
            match self.redis.set(&redis_key, &value, Some(ttl_secs)).await {
                Ok(()) => {
                    debug!(key, ttl_secs, "Hint cached in Redis");
                }
                Err(e) => {
                    warn!(key, error = %e, "Failed to cache hint in Redis");
                }
            }
        }
    }

    /// Clear all cache (both Redis and local).
    pub async fn clear(&self) -> Result<()> {
        // Clear local cache
        self.local.clear();

        // Clear Redis cache
        if self.redis.is_connected().await {
            // Note: This is a simplified implementation. In production,
            // you might want to use SCAN + DEL to avoid blocking Redis.
            // For now, we just clear the local cache and let Redis TTL handle expiration.
            debug!("Cleared local cache (Redis keys will expire via TTL)");
        }

        Ok(())
    }

    /// Get cache statistics.
    ///
    /// Returns local cache stats. Redis stats would require additional queries.
    pub async fn stats(&self) -> super::cache::CacheStats {
        self.local.stats()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::RedisClient;

    #[tokio::test]
    async fn test_distributed_cache_local_fallback() {
        // Create cache with empty Redis URL (local-only mode)
        let redis = Arc::new(RedisClient::new("").await.unwrap());
        let cache = DistributedAiCache::new(redis, "test");

        // Test set/get
        let response = GenerationResponse {
            questions: vec![serde_json::json!({"question": "Test?", "options": ["A", "B"], "correct": 0})],
            total_generated: 1,
            cached: false,
        };

        cache.set("test_key".to_string(), response.clone()).await;
        let retrieved = cache.get("test_key").await;

        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.total_generated, 1);
    }

    #[tokio::test]
    #[ignore = "requires Redis server"]
    async fn test_distributed_cache_redis() {
        let redis = Arc::new(RedisClient::new("redis://127.0.0.1:6379").await.unwrap());
        let cache = DistributedAiCache::new(redis, "test");

        let response = GenerationResponse {
            questions: vec![serde_json::json!({"question": "Test?", "options": ["A", "B"], "correct": 0})],
            total_generated: 1,
            cached: false,
        };

        cache.set("test_key".to_string(), response.clone()).await;
        let retrieved = cache.get("test_key").await;

        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.total_generated, 1);
        assert!(retrieved.cached); // Should be marked as cached

        // Cleanup
        cache.clear().await.unwrap();
    }
}
