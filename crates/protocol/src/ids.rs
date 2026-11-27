//! ID newtypes for type safety.
//!
//! These newtypes provide compile-time guarantees that we don't mix up
//! different kinds of IDs (room codes, user IDs, question IDs, etc.).

use serde::{Deserialize, Serialize};
use std::fmt;

/// Room code (6-character alphanumeric string).
///
/// # Examples
///
/// ```
/// use forgebrain_protocol::RoomCode;
///
/// let code = RoomCode::new("ABC123");
/// assert_eq!(code.as_str(), "ABC123");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RoomCode(String);

impl RoomCode {
    /// Create a new room code from a string.
    ///
    /// The code must be exactly 6 alphanumeric characters.
    pub fn new(code: impl Into<String>) -> Self {
        let code = code.into();
        debug_assert_eq!(code.len(), 6, "Room code must be 6 characters");
        debug_assert!(
            code.chars().all(|c| c.is_alphanumeric()),
            "Room code must be alphanumeric"
        );
        Self(code.to_uppercase())
    }

    /// Get the code as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert into the inner string.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl fmt::Display for RoomCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<RoomCode> for String {
    fn from(code: RoomCode) -> Self {
        code.0
    }
}

/// User ID (i64 from database).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(i64);

impl UserId {
    /// Create a new user ID.
    pub fn new(id: i64) -> Self {
        Self(id)
    }

    /// Get the ID as i64.
    pub fn as_i64(&self) -> i64 {
        self.0
    }
}

impl From<i64> for UserId {
    fn from(id: i64) -> Self {
        Self(id)
    }
}

impl From<UserId> for i64 {
    fn from(id: UserId) -> Self {
        id.0
    }
}

/// Question ID (index in the question list).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct QuestionId(usize);

impl QuestionId {
    /// Create a new question ID.
    pub fn new(index: usize) -> Self {
        Self(index)
    }

    /// Get the index.
    pub fn as_usize(&self) -> usize {
        self.0
    }
}

impl From<usize> for QuestionId {
    fn from(index: usize) -> Self {
        Self(index)
    }
}

impl From<QuestionId> for usize {
    fn from(id: QuestionId) -> Self {
        id.0
    }
}

/// Session ID (UUID string).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(String);

impl SessionId {
    /// Create a new session ID.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the ID as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_room_code_new() {
        let code = RoomCode::new("abc123");
        assert_eq!(code.as_str(), "ABC123");
    }

    #[test]
    fn test_room_code_serialization() {
        let code = RoomCode::new("XYZ789");
        let json = serde_json::to_string(&code).unwrap();
        assert_eq!(json, "\"XYZ789\"");

        let decoded: RoomCode = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, code);
    }

    #[test]
    fn test_user_id_conversions() {
        let id = UserId::new(42);
        assert_eq!(id.as_i64(), 42);

        let from_i64: UserId = 100.into();
        assert_eq!(from_i64.as_i64(), 100);

        let to_i64: i64 = id.into();
        assert_eq!(to_i64, 42);
    }

    #[test]
    fn test_question_id() {
        let id = QuestionId::new(5);
        assert_eq!(id.as_usize(), 5);

        let from_usize: QuestionId = 10.into();
        assert_eq!(from_usize.as_usize(), 10);
    }
}
