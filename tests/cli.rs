// Integration tests only use a few of the package's dependencies
#![allow(unused_crate_dependencies)]

use std::path::PathBuf;
use std::process::{Command, Output};

use anyhow::Result;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};

fn jwt(exp: i64) -> String {
    let body = URL_SAFE_NO_PAD.encode(format!(r#"{{"exp":{exp}}}"#));
    format!("header.{body}.sig")
}

/// Run `chepstow token` with a cached access token that has 30s left.
/// Returns the output and the cache file path.
fn token_near_expiry(test: &str, server: &mut mockito::Server) -> Result<(Output, PathBuf)> {
    let issuer = server.url();
    server
        .mock("GET", "/.well-known/openid-configuration")
        .with_header("content-type", "application/json")
        .with_body(format!(
            r#"{{"device_authorization_endpoint":"{issuer}/device","token_endpoint":"{issuer}/token"}}"#
        ))
        .create();

    let home = std::env::temp_dir().join(format!("chepstow-{}-{test}", std::process::id()));
    // Where `dirs::cache_dir()` points given HOME and XDG_CACHE_HOME
    let cache_dir = if cfg!(target_os = "macos") {
        home.join("Library/Caches")
    } else {
        home.clone()
    }
    .join("chepstow");
    std::fs::create_dir_all(&cache_dir)?;
    let cache_file = cache_dir.join("dev.json");
    let now = chrono::Utc::now().timestamp();
    let cache = serde_json::json!({
        "issuer": issuer,
        "client_id": "chepstow",
        "access_token": jwt(now + 30),
        "refresh_token": "old-refresh",
        "expires_at": now + 30,
        "refresh_expires_at": null,
    });
    std::fs::write(&cache_file, cache.to_string())?;

    let output = Command::new(env!("CARGO_BIN_EXE_chepstow"))
        .arg("token")
        .env("HOME", &home)
        .env("XDG_CACHE_HOME", &home)
        .env_remove("CHEPSTOW_ENV")
        .output()?;
    Ok((output, cache_file))
}

#[test]
fn token_refreshes_near_expiry() -> Result<()> {
    let mut server = mockito::Server::new();
    let new_token = jwt(chrono::Utc::now().timestamp() + 300);
    let refresh = server
        .mock("POST", "/token")
        .match_body(mockito::Matcher::AllOf(vec![
            mockito::Matcher::Regex("grant_type=refresh_token".into()),
            mockito::Matcher::Regex("refresh_token=old-refresh".into()),
        ]))
        .with_header("content-type", "application/json")
        .with_body(format!(
            r#"{{"access_token":"{new_token}","token_type":"Bearer","expires_in":300}}"#
        ))
        .create();

    let (output, cache_file) = token_near_expiry("refresh", &mut server)?;

    refresh.assert();
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout)?, format!("{new_token}\n"));
    let cache: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(cache_file)?)?;
    assert_eq!(cache["access_token"], new_token.as_str());
    assert_eq!(cache["refresh_token"], "old-refresh");
    Ok(())
}

#[test]
fn token_fails_quietly_when_refresh_rejected() -> Result<()> {
    let mut server = mockito::Server::new();
    server
        .mock("POST", "/token")
        .with_status(400)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error":"invalid_grant","error_description":"Token is not active"}"#)
        .create();

    let (output, _) = token_near_expiry("rejected", &mut server)?;

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8(output.stderr)?.contains("Run `chepstow login`"));
    Ok(())
}

#[test]
fn token_fails_when_not_logged_in() -> Result<()> {
    let home = std::env::temp_dir().join(format!("chepstow-{}-none", std::process::id()));
    let output = Command::new(env!("CARGO_BIN_EXE_chepstow"))
        .arg("token")
        .env("HOME", &home)
        .env("XDG_CACHE_HOME", &home)
        .env_remove("CHEPSTOW_ENV")
        .output()?;

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr)?,
        "You are not logged in. Run `chepstow login` to obtain an access token.\n"
    );
    Ok(())
}
