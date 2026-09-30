use anyhow::{Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};

/// Decode a JWT's payload without verifying its signature
pub fn payload(token: &str) -> Result<serde_json::Value> {
    let segment = token.split('.').nth(1).context("Token is not a JWT.")?;
    let bytes = URL_SAFE_NO_PAD
        .decode(segment.trim_end_matches('='))
        .context("JWT payload is not base64url.")?;
    serde_json::from_slice(&bytes).context("JWT payload is not JSON.")
}

/// The `exp` claim, if the token is a JWT with a non-zero expiry
pub fn exp(token: &str) -> Option<i64> {
    payload(token).ok()?["exp"].as_i64().filter(|&e| e > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_payload() {
        let body = URL_SAFE_NO_PAD.encode(r#"{"exp":1700000000,"groups":["a"]}"#);
        let token = format!("header.{body}.sig");
        assert_eq!(
            payload(&token).ok().map(|p| p["groups"][0].clone()),
            Some("a".into())
        );
        assert_eq!(exp(&token), Some(1_700_000_000));
        assert_eq!(exp("not-a-jwt"), None);
        assert_eq!(
            exp(&format!("h.{}.s", URL_SAFE_NO_PAD.encode(r#"{"exp":0}"#))),
            None
        );
    }
}
