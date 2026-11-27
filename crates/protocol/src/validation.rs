//! Shared validators for protocol types.
//!
//! These validators ensure that data conforms to expected constraints
//! before being processed by the server or client.

use thiserror::Error;

/// Validation error.
#[derive(Debug, Error)]
pub enum ValidationError {
    /// Room code is invalid.
    #[error("invalid room code: {0}")]
    InvalidRoomCode(String),
    /// Answer option index is out of bounds.
    #[error("option index {index} out of bounds (max {max})")]
    OptionIndexOutOfBounds {
        /// The invalid index.
        index: u8,
        /// Maximum valid index.
        max: u8,
    },
    /// Message type is unknown.
    #[error("unknown message type: {0}")]
    UnknownMessageType(String),
    /// Required field is missing.
    #[error("missing required field: {0}")]
    MissingField(String),
}

/// Validate a room code.
///
/// Room codes must be exactly 6 alphanumeric characters.
///
/// # Examples
///
/// ```
/// use forgebrain_protocol::validate_room_code;
///
/// assert!(validate_room_code("ABC123").is_ok());
/// assert!(validate_room_code("abc123").is_ok()); // case-insensitive
/// assert!(validate_room_code("ABC12").is_err()); // too short
/// assert!(validate_room_code("ABC1234").is_err()); // too long
/// assert!(validate_room_code("ABC-12").is_err()); // non-alphanumeric
/// ```
pub fn validate_room_code(code: &str) -> Result<(), ValidationError> {
    if code.len() != 6 {
        return Err(ValidationError::InvalidRoomCode(
            "must be exactly 6 characters".to_string(),
        ));
    }

    if !code.chars().all(|c| c.is_alphanumeric()) {
        return Err(ValidationError::InvalidRoomCode(
            "must be alphanumeric".to_string(),
        ));
    }

    Ok(())
}

/// Validate an answer option index.
///
/// Option indices must be within the range [0, num_options).
///
/// # Examples
///
/// ```
/// use forgebrain_protocol::validate_option_index;
///
/// assert!(validate_option_index(0, 4).is_ok());
/// assert!(validate_option_index(3, 4).is_ok());
/// assert!(validate_option_index(4, 4).is_err()); // out of bounds
/// ```
pub fn validate_option_index(index: u8, num_options: u8) -> Result<(), ValidationError> {
    if index >= num_options {
        return Err(ValidationError::OptionIndexOutOfBounds {
            index,
            max: num_options - 1,
        });
    }
    Ok(())
}

/// Validate a question.
///
/// Questions must have:
/// - Non-empty question text
/// - At least 2 options
/// - All options non-empty
///
/// # Examples
///
/// ```
/// use forgebrain_protocol::validate_question;
///
/// let valid = ("What is 2+2?", vec!["3", "4", "5"]);
/// assert!(validate_question(&valid).is_ok());
///
/// let invalid = ("", vec!["3", "4"]);
/// assert!(validate_question(&invalid).is_err());
/// ```
pub fn validate_question(question: &(&str, Vec<&str>)) -> Result<(), ValidationError> {
    let (text, options) = question;

    if text.trim().is_empty() {
        return Err(ValidationError::MissingField("question text".to_string()));
    }

    if options.len() < 2 {
        return Err(ValidationError::MissingField(
            "at least 2 options required".to_string(),
        ));
    }

    if options.iter().any(|opt| opt.trim().is_empty()) {
        return Err(ValidationError::MissingField(
            "all options must be non-empty".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_room_code_valid() {
        assert!(validate_room_code("ABC123").is_ok());
        assert!(validate_room_code("abc123").is_ok());
        assert!(validate_room_code("XYZ789").is_ok());
    }

    #[test]
    fn test_validate_room_code_invalid_length() {
        assert!(validate_room_code("ABC12").is_err());
        assert!(validate_room_code("ABC1234").is_err());
        assert!(validate_room_code("").is_err());
    }

    #[test]
    fn test_validate_room_code_invalid_chars() {
        assert!(validate_room_code("ABC-12").is_err());
        assert!(validate_room_code("ABC 12").is_err());
        assert!(validate_room_code("ABC!12").is_err());
    }

    #[test]
    fn test_validate_option_index_valid() {
        assert!(validate_option_index(0, 4).is_ok());
        assert!(validate_option_index(3, 4).is_ok());
        assert!(validate_option_index(0, 2).is_ok());
        assert!(validate_option_index(1, 2).is_ok());
    }

    #[test]
    fn test_validate_option_index_invalid() {
        assert!(validate_option_index(4, 4).is_err());
        assert!(validate_option_index(10, 4).is_err());
    }

    #[test]
    fn test_validate_question_valid() {
        let question = ("What is 2+2?", vec!["3", "4", "5"]);
        assert!(validate_question(&question).is_ok());
    }

    #[test]
    fn test_validate_question_empty_text() {
        let question = ("", vec!["3", "4"]);
        assert!(validate_question(&question).is_err());
    }

    #[test]
    fn test_validate_question_too_few_options() {
        let question = ("What?", vec!["4"]);
        assert!(validate_question(&question).is_err());
    }

    #[test]
    fn test_validate_question_empty_option() {
        let question = ("What?", vec!["4", ""]);
        assert!(validate_question(&question).is_err());
    }
}
