//! ForgeBrain Protocol - shared types for backend and frontend (WASM).
//!
//! This crate contains:
//! - WebSocket message types (client <-> server)
//! - Request/Response DTOs
//! - ID newtypes (RoomCode, UserId, QuestionId)
//! - Shared validators
//!
//! # WASM Compatibility
//!
//! This crate is designed to compile to `wasm32-unknown-unknown` for use in
//! frontend applications. All types are serializable with serde.
//!
//! # Example
//!
//! ```rust,no_run
//! use forgebrain_protocol::{ClientMessage, RoomCode};
//!
//! let msg = ClientMessage::JoinRoom { code: RoomCode::new("ABC123") };
//! let json = serde_json::to_string(&msg).unwrap();
//! ```

#![deny(missing_docs)]
// Allow unwrap in tests - production code should use proper error handling
#![cfg_attr(not(test), deny(clippy::unwrap_used))]

pub mod ids;
pub mod messages;
pub mod validation;

pub use ids::*;
pub use messages::*;
pub use validation::*;
