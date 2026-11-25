//! Security primitives: HMAC, constant-time comparison, CSRF tokens.

use base64::Engine;
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

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

/// Generate an HMAC-SHA256 token for rejoin/session verification.
pub fn generate_hmac(secret: &str, data: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC can take any key length");
    mac.update(data.as_bytes());
    let result = mac.finalize();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(result.into_bytes())
}

/// Verify an HMAC token.
pub fn verify_hmac(secret: &str, data: &str, token: &str) -> bool {
    let expected = generate_hmac(secret, data);
    safe_equal(&expected, token)
}

/// Generate a cryptographically random token (for sessions, CSRF, etc).
pub fn random_token(len_bytes: usize) -> String {
    use rand::RngCore;
    let mut bytes = vec![0u8; len_bytes];
    rand::thread_rng().fill_bytes(&mut bytes);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&bytes)
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

    #[test]
    fn hmac_roundtrip() {
        let token = generate_hmac("secret", "data");
        assert!(verify_hmac("secret", "data", &token));
        assert!(!verify_hmac("secret", "wrong", &token));
    }

    #[test]
    fn random_token_length() {
        let token = random_token(32);
        assert!(!token.is_empty());
    }
}
