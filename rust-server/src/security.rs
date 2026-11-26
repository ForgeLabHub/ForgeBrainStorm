//! Security primitives: constant-time comparison.

/// Constant-time string comparison (prevents timing attacks).
pub fn safe_equal(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut result = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        result |= x ^ y;
    }
    result == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_equal_works() {
        assert!(safe_equal("abc", "abc"));
        assert!(!safe_equal("abc", "abd"));
        assert!(!safe_equal("abc", "abcd"));
    }
}
