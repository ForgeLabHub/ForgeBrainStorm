//! Infrastructure layer: external services and distributed systems.
//!
//! This module contains clients for external services like Redis,
//! message queues, and other infrastructure components.

pub mod pubsub_manager;
pub mod redis_client;

pub use pubsub_manager::{generate_instance_id, PubSubManager, RoomEvent};
pub use redis_client::RedisClient;
