//! Redis integration tests.
//!
//! Run with: cargo test --test redis_integration_test -- --ignored
//! Requires Redis running on localhost:6379 (use docker-compose up)

use brainstorm::infrastructure::RedisClient;

#[tokio::test]
#[ignore = "requires Redis server"]
async fn test_redis_connection() {
    let client = RedisClient::new("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect to Redis");

    assert!(client.is_connected().await, "Redis should be connected");
}

#[tokio::test]
#[ignore = "requires Redis server"]
async fn test_redis_set_get_delete() {
    let client = RedisClient::new("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect to Redis");

    let key = "test:integration:key";
    let value = "test_value_123";

    // Set with TTL
    client.set(key, value, Some(60)).await.unwrap();

    // Get
    let retrieved = client.get(key).await.unwrap();
    assert_eq!(retrieved, Some(value.to_string()));

    // Exists
    let exists = client.exists(key).await.unwrap();
    assert!(exists);

    // Delete
    client.del(key).await.unwrap();

    // Verify deleted
    let retrieved = client.get(key).await.unwrap();
    assert_eq!(retrieved, None);
}

#[tokio::test]
#[ignore = "requires Redis server"]
async fn test_redis_set_nx() {
    let client = RedisClient::new("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect to Redis");

    let key = "test:integration:nx";

    // Cleanup first
    let _ = client.del(key).await;

    // First SETNX should succeed
    let result1 = client.set_nx(key, "value1", 60).await.unwrap();
    assert!(result1, "First SETNX should succeed");

    // Second SETNX should fail (key exists)
    let result2 = client.set_nx(key, "value2", 60).await.unwrap();
    assert!(!result2, "Second SETNX should fail");

    // Value should still be value1
    let value = client.get(key).await.unwrap();
    assert_eq!(value, Some("value1".to_string()));

    // Cleanup
    client.del(key).await.unwrap();
}

#[tokio::test]
#[ignore = "requires Redis server"]
async fn test_redis_incr() {
    let client = RedisClient::new("redis://127.0.0.1:6379")
        .await
        .expect("Failed to connect to Redis");

    let key = "test:integration:counter";

    // Cleanup first
    let _ = client.del(key).await;

    // Increment
    let val1 = client.incr(key).await.unwrap();
    assert_eq!(val1, 1);

    let val2 = client.incr(key).await.unwrap();
    assert_eq!(val2, 2);

    let val3 = client.incr(key).await.unwrap();
    assert_eq!(val3, 3);

    // Cleanup
    client.del(key).await.unwrap();
}

#[tokio::test]
async fn test_redis_empty_url() {
    // Empty URL should create a client in local-only mode
    let client = RedisClient::new("")
        .await
        .expect("Empty URL should not fail");

    assert!(!client.is_connected().await);
}
