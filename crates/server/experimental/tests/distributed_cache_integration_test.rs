//! Integration tests for distributed AI cache.
//!
//! Run with: cargo test --test distributed_cache_integration_test -- --ignored
//! Requires Redis running on localhost:6379 (use docker-compose up)

use std::sync::Arc;
use std::time::Duration;

use brainstorm::infrastructure::RedisClient;
use brainstorm::services::ai::distributed_cache::DistributedAiCache;
use brainstorm::services::ai::GenerationResponse;

#[tokio::test]
#[ignore = "requires Redis server"]
async fn test_distributed_cache_with_redis() {
    let redis = Arc::new(
        RedisClient::new("redis://127.0.0.1:6379")
            .await
            .expect("Failed to connect to Redis"),
    );

    let cache = DistributedAiCache::new(redis, "test_integration");

    // Create test response
    let response = GenerationResponse {
        questions: vec![serde_json::json!({
            "question": "What is 2+2?",
            "options": ["3", "4", "5", "6"],
            "correct_index": 1,
            "explanation": "Basic arithmetic"
        })],
        total_generated: 1,
        cached: false,
    };

    let key = "test:math:easy:1".to_string();

    // Store in cache
    cache.set(key.clone(), response.clone()).await;

    // Retrieve from cache
    let retrieved = cache.get(&key).await;
    assert!(retrieved.is_some(), "Should retrieve cached response");

    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.questions.len(), 1);
    assert_eq!(retrieved.total_generated, 1);
    assert!(retrieved.cached, "Should be marked as cached");

    // Cleanup
    cache.clear().await.unwrap();
}

#[tokio::test]
#[ignore = "requires Redis server"]
async fn test_distributed_cache_hint() {
    let redis = Arc::new(
        RedisClient::new("redis://127.0.0.1:6379")
            .await
            .expect("Failed to connect to Redis"),
    );

    let cache = DistributedAiCache::new(redis, "test_integration");

    let key = "test:hint:what_is_2_plus_2".to_string();
    let hint = "Think about basic arithmetic".to_string();

    // Store hint
    cache.set_hint(key.clone(), hint.clone()).await;

    // Retrieve hint
    let retrieved = cache.get_hint(&key).await;
    assert!(retrieved.is_some(), "Should retrieve cached hint");
    assert_eq!(retrieved.unwrap(), hint);

    // Cleanup
    cache.clear().await.unwrap();
}

#[tokio::test]
async fn test_distributed_cache_local_fallback() {
    // Create cache with empty Redis URL (local-only mode)
    let redis = Arc::new(
        RedisClient::new("").await.expect("Failed to create Redis client"),
    );

    let cache = DistributedAiCache::new(redis, "test_local");

    let response = GenerationResponse {
        questions: vec![serde_json::json!({
            "question": "Test question?",
            "options": ["A", "B"],
            "correct_index": 0,
            "explanation": "Test"
        })],
        total_generated: 1,
        cached: false,
    };

    let key = "test:local:key".to_string();

    // Store in cache (should use local fallback)
    cache.set(key.clone(), response.clone()).await;

    // Retrieve from cache
    let retrieved = cache.get(&key).await;
    assert!(retrieved.is_some(), "Should retrieve from local cache");

    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.questions.len(), 1);
}

#[tokio::test]
#[ignore = "requires Redis server"]
async fn test_distributed_cache_ttl() {
    let redis = Arc::new(
        RedisClient::new("redis://127.0.0.1:6379")
            .await
            .expect("Failed to connect to Redis"),
    );

    let cache = DistributedAiCache::new(redis, "test_ttl");

    let response = GenerationResponse {
        questions: vec![serde_json::json!({"question": "TTL test"})],
        total_generated: 1,
        cached: false,
    };

    let key = "test:ttl:key".to_string();

    // Store with default TTL (1 hour)
    cache.set(key.clone(), response).await;

    // Should be available immediately
    let retrieved = cache.get(&key).await;
    assert!(retrieved.is_some());

    // Note: Testing actual TTL expiration would require waiting 1 hour,
    // which is impractical. In production, Redis handles TTL automatically.

    // Cleanup
    cache.clear().await.unwrap();
}

#[tokio::test]
#[ignore = "requires Redis server"]
async fn test_distributed_cache_stats() {
    let redis = Arc::new(
        RedisClient::new("redis://127.0.0.1:6379")
            .await
            .expect("Failed to connect to Redis"),
    );

    let cache = DistributedAiCache::new(redis, "test_stats");

    // Initially empty
    let stats = cache.stats().await;
    let initial_questions = stats.questions_cached;
    let initial_hints = stats.hints_cached;

    // Add some entries
    let response = GenerationResponse {
        questions: vec![serde_json::json!({"question": "Q1"})],
        total_generated: 1,
        cached: false,
    };
    cache.set("key1".to_string(), response).await;

    cache.set_hint("hint1".to_string(), "Hint 1".to_string()).await;

    // Stats should reflect local cache (Redis stats not tracked separately)
    let stats = cache.stats().await;
    assert_eq!(stats.questions_cached, initial_questions + 1);
    assert_eq!(stats.hints_cached, initial_hints + 1);

    // Cleanup
    cache.clear().await.unwrap();
}
