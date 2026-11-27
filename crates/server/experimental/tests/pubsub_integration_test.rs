//! Redis Pub/Sub integration tests.
//!
//! Run with: cargo test --test pubsub_integration_test -- --ignored
//! Requires Redis running on localhost:6379 (use docker-compose up)

use std::sync::Arc;
use std::time::Duration;

use brainstorm::infrastructure::{generate_instance_id, PubSubManager, RedisClient};
use brainstorm::ws::protocol::ServerMessage;
use tokio::sync::broadcast;

#[tokio::test]
#[ignore = "requires Redis server"]
async fn test_pubsub_publish_subscribe() {
    // Create two PubSubManagers to simulate two instances
    let redis = Arc::new(
        RedisClient::new("redis://127.0.0.1:6379")
            .await
            .expect("Failed to connect to Redis"),
    );

    let instance_a = generate_instance_id();
    let instance_b = generate_instance_id();

    let pubsub_a = Arc::new(PubSubManager::new(instance_a.clone(), redis.clone()));
    let pubsub_b = Arc::new(PubSubManager::new(instance_b.clone(), redis.clone()));

    let room_code = "TEST01";

    // Create broadcast channels for each instance
    let (tx_a, mut rx_a) = broadcast::channel(16);
    let (tx_b, mut rx_b) = broadcast::channel(16);

    // Subscribe both instances to the room
    pubsub_a
        .subscribe(room_code, tx_a.clone(), |_| {})
        .await
        .expect("Failed to subscribe instance A");

    pubsub_b
        .subscribe(room_code, tx_b.clone(), |_| {})
        .await
        .expect("Failed to subscribe instance B");

    // Give subscriptions time to establish
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Instance A publishes a message
    let message = ServerMessage::GameStarted {};
    pubsub_a
        .publish(room_code, message.clone())
        .await
        .expect("Failed to publish message");

    // Instance B should receive the message
    let received = tokio::time::timeout(Duration::from_secs(2), rx_b.recv())
        .await
        .expect("Timeout waiting for message")
        .expect("Failed to receive message");

    // Verify the message
    match received {
        ServerMessage::GameStarted {} => {
            // Success!
        }
        _ => panic!("Unexpected message type"),
    }

    // Instance A should NOT receive its own message (deduplication)
    let result = tokio::time::timeout(Duration::from_millis(500), rx_a.recv()).await;
    assert!(
        result.is_err(),
        "Instance A should not receive its own message"
    );

    // Cleanup
    pubsub_a.unsubscribe(room_code).await;
    pubsub_b.unsubscribe(room_code).await;
}

#[tokio::test]
#[ignore = "requires Redis server"]
async fn test_pubsub_multiple_messages() {
    let redis = Arc::new(
        RedisClient::new("redis://127.0.0.1:6379")
            .await
            .expect("Failed to connect to Redis"),
    );

    let instance_a = generate_instance_id();
    let instance_b = generate_instance_id();

    let pubsub_a = Arc::new(PubSubManager::new(instance_a.clone(), redis.clone()));
    let pubsub_b = Arc::new(PubSubManager::new(instance_b.clone(), redis.clone()));

    let room_code = "TEST02";

    let (tx_a, mut rx_a) = broadcast::channel(16);
    let (tx_b, mut rx_b) = broadcast::channel(16);

    pubsub_a
        .subscribe(room_code, tx_a.clone(), |_| {})
        .await
        .expect("Failed to subscribe");
    pubsub_b
        .subscribe(room_code, tx_b.clone(), |_| {})
        .await
        .expect("Failed to subscribe");

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Publish multiple messages from instance A
    for i in 0..5 {
        let message = ServerMessage::ChatMessage {
            user_id: 1,
            username: "test_user".to_string(),
            text: format!("Message {}", i),
            timestamp: chrono::Utc::now().timestamp(),
        };
        pubsub_a
            .publish(room_code, message)
            .await
            .expect("Failed to publish");
    }

    // Instance B should receive all 5 messages
    for i in 0..5 {
        let received = tokio::time::timeout(Duration::from_secs(2), rx_b.recv())
            .await
            .expect("Timeout waiting for message")
            .expect("Failed to receive message");

        match received {
            ServerMessage::ChatMessage { text, .. } => {
                assert_eq!(text, format!("Message {}", i));
            }
            _ => panic!("Unexpected message type"),
        }
    }

    // Cleanup
    pubsub_a.unsubscribe(room_code).await;
    pubsub_b.unsubscribe(room_code).await;
}

#[tokio::test]
#[ignore = "requires Redis server"]
async fn test_pubsub_unsubscribe() {
    let redis = Arc::new(
        RedisClient::new("redis://127.0.0.1:6379")
            .await
            .expect("Failed to connect to Redis"),
    );

    let instance_a = generate_instance_id();
    let pubsub_a = Arc::new(PubSubManager::new(instance_a.clone(), redis.clone()));

    let room_code = "TEST03";
    let (tx_a, _rx_a) = broadcast::channel(16);

    // Subscribe
    pubsub_a
        .subscribe(room_code, tx_a.clone(), |_| {})
        .await
        .expect("Failed to subscribe");

    assert!(pubsub_a.is_subscribed(room_code).await);
    assert_eq!(pubsub_a.subscription_count().await, 1);

    // Unsubscribe
    pubsub_a.unsubscribe(room_code).await;

    // Give time for cleanup
    tokio::time::sleep(Duration::from_millis(100)).await;

    assert!(!pubsub_a.is_subscribed(room_code).await);
    assert_eq!(pubsub_a.subscription_count().await, 0);
}

#[tokio::test]
async fn test_pubsub_graceful_degradation() {
    // Create PubSubManager with empty Redis URL (local-only mode)
    let redis = Arc::new(
        RedisClient::new("")
            .await
            .expect("Failed to create Redis client"),
    );

    let instance_id = generate_instance_id();
    let pubsub = Arc::new(PubSubManager::new(instance_id, redis));

    let room_code = "TEST04";
    let (tx, _rx) = broadcast::channel(16);

    // Subscribe should succeed (no-op in local-only mode)
    pubsub
        .subscribe(room_code, tx.clone(), |_| {})
        .await
        .expect("Subscribe should succeed in local-only mode");

    // Publish should succeed (no-op in local-only mode)
    let message = ServerMessage::GameStarted {};
    pubsub
        .publish(room_code, message)
        .await
        .expect("Publish should succeed in local-only mode");

    // Unsubscribe should succeed
    pubsub.unsubscribe(room_code).await;
}
