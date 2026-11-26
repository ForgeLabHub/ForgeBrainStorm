//! Infrastructure layer: external services and distributed systems.
//!
//! This module contains clients for external services like Redis,
//! message queues, and other infrastructure components.

pub mod redis_client;

pub use redis_client::RedisClient;
