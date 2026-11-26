//! Pub/Sub manager for distributed room synchronization.
//!
//! Enables horizontal scaling by broadcasting room events across multiple
//! server instances via Redis Pub/Sub. Each instance subscribes to rooms
//! it manages and forwards events to local WebSocket clients.
//!
//! ## Architecture
//!
//! ```text
//! Instance A                    Redis                     Instance B
//! ┌─────────────┐               ┌─────────┐               ┌─────────────┐
//! │ RoomManager │─publish()────▶│ Room:ABC│─subscribe()─▶│ PubSubMgr   │
//! │             │               │ channel │               │             │
//! │ broadcast   │◀─subscribe()──│         │◀─publish()────│ broadcast   │
//! │ (clients)   │               └─────────┘               │ (clients)   │
//! └─────────────┘                                          └─────────────┘
//! ```
//!
//! ## Deduplication
//!
//! Each published message includes the source `instance_id`. When a message
//! is received from Redis, the manager checks if it originated from this
//! instance. If so, it's ignored (already sent to local clients). Otherwise,
//! it's forwarded to the local broadcast channel.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Context, Result};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::infrastructure::RedisClient;
use crate::ws::protocol::ServerMessage;

/// Unique identifier for this server instance.
/// Generated at startup, used for deduplication.
pub fn generate_instance_id() -> String {
    Uuid::new_v4().to_string()
}

/// Wrapper around ServerMessage with metadata for distributed publishing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomEvent {
    /// Source instance ID (for deduplication).
    pub instance_id: String,
    /// Room code.
    pub room_code: String,
    /// The actual message to broadcast.
    pub message: ServerMessage,
    /// Timestamp (for debugging/ordering).
    pub timestamp: i64,
}

impl RoomEvent {
    pub fn new(instance_id: String, room_code: String, message: ServerMessage) -> Self {
        Self {
            instance_id,
            room_code,
            message,
            timestamp: chrono::Utc::now().timestamp_millis(),
        }
    }
}

/// Manages Redis Pub/Sub subscriptions for room synchronization.
///
/// Each room gets its own Redis channel: `room:{code}`.
/// The manager subscribes to channels for rooms managed by this instance
/// and forwards received events to local broadcast channels.
pub struct PubSubManager {
    /// This instance's unique ID.
    instance_id: String,
    /// Redis client for publishing and subscribing.
    redis: Arc<RedisClient>,
    /// Active subscriptions: room_code -> abort handle.
    subscriptions: Arc<RwLock<HashMap<String, tokio::sync::oneshot::Sender<()>>>>,
}

impl PubSubManager {
    /// Create a new Pub/Sub manager.
    pub fn new(instance_id: String, redis: Arc<RedisClient>) -> Self {
        Self {
            instance_id,
            redis,
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get this instance's ID.
    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    /// Publish a room event to Redis.
    ///
    /// This broadcasts the message to all other instances subscribed to this room.
    /// The message includes the source instance_id for deduplication.
    pub async fn publish(&self, room_code: &str, message: ServerMessage) -> Result<()> {
        if !self.redis.is_connected().await {
            debug!("Redis not connected, skipping publish for room {}", room_code);
            return Ok(());
        }

        let event = RoomEvent::new(
            self.instance_id.clone(),
            room_code.to_string(),
            message,
        );

        let json = serde_json::to_string(&event)
            .with_context(|| "Failed to serialize room event")?;

        let channel = format!("room:{}", room_code);
        self.redis.publish(&channel, &json).await?;

        debug!(
            room_code,
            instance_id = self.instance_id,
            "Published room event to Redis"
        );

        Ok(())
    }

    /// Subscribe to a room's Redis channel.
    ///
    /// Starts listening for events from other instances and forwards them
    /// to the provided broadcast sender (which sends to local WebSocket clients).
    ///
    /// Returns a handle that can be used to unsubscribe.
    pub async fn subscribe<F>(
        &self,
        room_code: &str,
        broadcast_tx: broadcast::Sender<ServerMessage>,
        on_message: F,
    ) -> Result<()>
    where
        F: Fn(ServerMessage) + Send + Sync + 'static,
    {
        if !self.redis.is_connected().await {
            debug!("Redis not connected, skipping subscribe for room {}", room_code);
            return Ok(());
        }

        let channel = format!("room:{}", room_code);
        let instance_id = self.instance_id.clone();
        let room_code_owned = room_code.to_string();

        // Create a oneshot channel for aborting the subscription
        let (abort_tx, mut abort_rx) = tokio::sync::oneshot::channel::<()>();

        // Clone redis client for the subscription task
        let redis = self.redis.clone();
        let broadcast_tx_clone = broadcast_tx.clone();

        // Spawn a task to listen for messages
        let handle = tokio::spawn(async move {
            info!(
                room_code = room_code_owned,
                instance_id,
                "Subscribing to Redis channel"
            );

            // Use Redis subscribe with a callback
            let result = redis
                .subscribe(&channel, move |payload: String| {
                    let instance_id = instance_id.clone();
                    let broadcast_tx = broadcast_tx_clone.clone();
                    let on_message = &on_message;

                    async move {
                        // Deserialize the event
                        let event: RoomEvent = match serde_json::from_str(&payload) {
                            Ok(e) => e,
                            Err(e) => {
                                error!("Failed to deserialize room event: {}", e);
                                return Ok(());
                            }
                        };

                        // Deduplication: ignore messages from this instance
                        if event.instance_id == instance_id {
                            debug!(
                                room_code = event.room_code,
                                "Ignoring own message (deduplication)"
                            );
                            return Ok(());
                        }

                        debug!(
                            room_code = event.room_code,
                            source_instance = event.instance_id,
                            "Received room event from Redis"
                        );

                        // Forward to local broadcast channel
                        if let Err(e) = broadcast_tx.send(event.message.clone()) {
                            // No receivers — room might have been cleaned up
                            debug!("Failed to forward to broadcast (no receivers): {}", e);
                        }

                        // Call the callback for additional processing
                        on_message(event.message);

                        Ok(())
                    }
                })
                .await;

            if let Err(e) = result {
                error!(
                    room_code = room_code_owned,
                    error = %e,
                    "Redis subscription error"
                );
            }

            info!(
                room_code = room_code_owned,
                "Redis subscription ended"
            );
        });

        // Store the abort handle
        self.subscriptions
            .write()
            .await
            .insert(room_code.to_string(), abort_tx);

        // Spawn a task to wait for abort signal
        let subscriptions = self.subscriptions.clone();
        let room_code_for_abort = room_code.to_string();
        tokio::spawn(async move {
            // Wait for either abort signal or subscription task to finish
            tokio::select! {
                _ = &mut abort_rx => {
                    debug!(room_code = room_code_for_abort, "Aborting Redis subscription");
                }
                _ = handle => {
                    debug!(room_code = room_code_for_abort, "Redis subscription task finished");
                }
            }

            // Clean up
            subscriptions.write().await.remove(&room_code_for_abort);
        });

        Ok(())
    }

    /// Unsubscribe from a room's Redis channel.
    pub async fn unsubscribe(&self, room_code: &str) {
        let mut subs = self.subscriptions.write().await;
        if let Some(abort_tx) = subs.remove(room_code) {
            let _ = abort_tx.send(());
            info!(room_code, "Unsubscribed from Redis channel");
        }
    }

    /// Check if subscribed to a room.
    pub async fn is_subscribed(&self, room_code: &str) -> bool {
        self.subscriptions.read().await.contains_key(room_code)
    }

    /// Get the number of active subscriptions.
    pub async fn subscription_count(&self) -> usize {
        self.subscriptions.read().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_instance_id() {
        let id1 = generate_instance_id();
        let id2 = generate_instance_id();
        assert_ne!(id1, id2);
        assert!(!id1.is_empty());
    }

    #[test]
    fn test_room_event_serialization() {
        let event = RoomEvent::new(
            "test-instance".to_string(),
            "ABC123".to_string(),
            ServerMessage::GameStarted {},
        );

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: RoomEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.instance_id, "test-instance");
        assert_eq!(deserialized.room_code, "ABC123");
    }
}
