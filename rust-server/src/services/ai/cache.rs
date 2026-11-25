//! AI response cache with TTL.
//!
//! Caches generated questions, hints, and explanations to reduce API calls
//! and costs. Uses in-memory cache with automatic expiration.

use std::sync::Arc;
use std::time::{Duration, Instant};

use dashmap::DashMap;
use tokio::sync::Mutex;
use tracing::debug;

use super::GenerationResponse;

/// Cache entry with expiration.
struct CacheEntry<T> {
    value: T,
    expires_at: Instant,
}

/// AI response cache.
pub struct AiCache {
    questions: Arc<DashMap<String, CacheEntry<GenerationResponse>>>,
    hints: Arc<DashMap<String, CacheEntry<String>>>,
    cleanup_interval: Duration,
}

impl AiCache {
    pub fn new(ttl: Duration) -> Self {
        let cache = Self {
            questions: Arc::new(DashMap::new()),
            hints: Arc::new(DashMap::new()),
            cleanup_interval: Duration::from_secs(60),
        };

        // Start background cleanup task
        let questions = Arc::clone(&cache.questions);
        let hints = Arc::clone(&cache.hints);
        let interval = cache.cleanup_interval;

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            loop {
                interval_timer.tick().await;
                Self::cleanup(&questions, &hints);
            }
        });

        cache
    }

    fn cleanup(
        questions: &DashMap<String, CacheEntry<GenerationResponse>>,
        hints: &DashMap<String, CacheEntry<String>>,
    ) {
        let now = Instant::now();
        let before_q = questions.len();
        let before_h = hints.len();

        questions.retain(|_, entry| entry.expires_at > now);
        hints.retain(|_, entry| entry.expires_at > now);

        let removed_q = before_q - questions.len();
        let removed_h = before_h - hints.len();

        if removed_q > 0 || removed_h > 0 {
            debug!(
                removed_questions = removed_q,
                removed_hints = removed_h,
                "Cache cleanup completed"
            );
        }
    }

    /// Get cached questions.
    pub async fn get(&self, key: &str) -> Option<GenerationResponse> {
        let entry = self.questions.get(key)?;
        if entry.expires_at > Instant::now() {
            Some(entry.value.clone())
        } else {
            None
        }
    }

    /// Cache questions.
    pub async fn set(&self, key: String, value: GenerationResponse) {
        let expires_at = Instant::now() + Duration::from_secs(3600); // 1 hour
        self.questions.insert(
            key,
            CacheEntry {
                value,
                expires_at,
            },
        );
    }

    /// Get cached hint.
    pub async fn get_hint(&self, key: &str) -> Option<String> {
        let entry = self.hints.get(key)?;
        if entry.expires_at > Instant::now() {
            Some(entry.value.clone())
        } else {
            None
        }
    }

    /// Cache hint.
    pub async fn set_hint(&self, key: String, value: String) {
        let expires_at = Instant::now() + Duration::from_secs(1800); // 30 minutes
        self.hints.insert(
            key,
            CacheEntry {
                value,
                expires_at,
            },
        );
    }

    /// Clear all cache.
    pub async fn clear(&self) {
        self.questions.clear();
        self.hints.clear();
    }

    /// Get cache statistics.
    pub async fn stats(&self) -> CacheStats {
        CacheStats {
            questions_cached: self.questions.len(),
            hints_cached: self.hints.len(),
        }
    }
}

/// Cache statistics.
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub questions_cached: usize,
    pub hints_cached: usize,
}
