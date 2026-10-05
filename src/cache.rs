use std::io::Write as _;
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt as _;
use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

/// Tokens for one environment. Times are unix seconds.
#[derive(Serialize, Deserialize)]
pub struct Cache {
    pub issuer: String,
    pub client_id: String,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: i64,
    pub refresh_expires_at: Option<i64>,
}

fn path(env: &str) -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("chepstow")
        .join(format!("{env}.json"))
}

pub fn load(env: &str) -> Result<Cache> {
    let path = path(env);
    let contents = match std::fs::read_to_string(&path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            bail!("You are not logged in. Run `chepstow auth` to obtain an access token.")
        }
        r => r.with_context(|| format!("Could not read token cache `{}`.", path.display()))?,
    };
    serde_json::from_str(&contents).context("Could not parse token cache.")
}

pub fn save(env: &str, cache: &Cache) -> Result<()> {
    let path = path(env);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)
            .with_context(|| format!("Could not create cache directory `{}`.", dir.display()))?;
    }
    let mut f = std::fs::OpenOptions::new();
    #[cfg(unix)]
    f.mode(0o600); // u=rw,g=,o=
    f.write(true)
        .truncate(true)
        .create(true)
        .open(&path)
        .with_context(|| format!("Could not open cache file `{}`.", path.display()))?
        .write_all(&serde_json::to_vec_pretty(cache)?)
        .context("Could not write to cache.")
}

pub fn delete(env: &str) -> Result<()> {
    match std::fs::remove_file(path(env)) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        r => r.context("Could not delete cache file."),
    }
}
