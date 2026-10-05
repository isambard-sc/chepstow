use anyhow::{bail, Context, Result};
use clap::{CommandFactory as _, Parser, Subcommand, ValueEnum};

mod cache;
mod jwt;
mod oidc;

#[derive(Clone, Copy, ValueEnum)]
enum Env {
    Dev,
    Prod,
}

#[derive(Parser)]
#[command(version, about)]
/// Log in to Isambard and get a token for the inference service
struct Args {
    /// Environment, which sets the default issuer and base URL
    #[arg(long, env = "CHEPSTOW_ENV", value_enum, default_value_t = Env::Dev, global = true)]
    env: Env,
    /// OIDC issuer URL
    #[arg(long, env = "CHEPSTOW_ISSUER", global = true)]
    issuer: Option<String>,
    /// OIDC client ID
    #[arg(
        long,
        env = "CHEPSTOW_CLIENT_ID",
        default_value = "chepstow",
        global = true
    )]
    client_id: String,
    /// LiteLLM base URL
    #[arg(long, env = "CHEPSTOW_BASE_URL", global = true)]
    base_url: Option<String>,
    /// OAuth scopes to request, space separated
    #[arg(long, env = "CHEPSTOW_SCOPE", default_value = "openid", global = true)]
    scope: String,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Log in with the device flow and cache the tokens
    Login,
    /// Print a valid access token, refreshing it if needed
    Token,
    /// Show the claims in the cached access token
    Whoami,
    /// List the models LiteLLM offers, to check it accepts the token
    Models,
    /// Revoke the refresh token and delete the cache
    Logout,
}

/// Shown when chepstow is run without a command
fn print_getting_started() {
    println!("Log in to Isambard and get a token for the inference service.\n");
    println!("To get started, run:\n");
    println!("  chepstow login\n");
    println!("This prints a link to open in your browser so you can log in.\n");
    println!("Then, to print an access token for the inference service, run:\n");
    println!("  chepstow token\n");
    println!("Available commands:");
    for sub in Args::command().get_subcommands() {
        let about = sub.get_about().map(|a| a.to_string()).unwrap_or_default();
        println!("  chepstow {:<8} {about}", sub.get_name());
    }
    println!("\nRun `chepstow help` for more details.");
}

fn now() -> i64 {
    chrono::Utc::now().timestamp()
}

/// Cache new tokens, keeping `old_refresh` if the server didn't send a new one
fn save(
    env: &str,
    issuer: &str,
    client_id: &str,
    t: oidc::Tokens,
    old_refresh: Option<String>,
) -> Result<cache::Cache> {
    let refresh_token = t.refresh_token.or(old_refresh);
    let c = cache::Cache {
        issuer: issuer.to_string(),
        client_id: client_id.to_string(),
        expires_at: jwt::exp(&t.access_token).context("Access token has no expiry.")?,
        refresh_expires_at: refresh_token.as_deref().and_then(jwt::exp),
        access_token: t.access_token,
        refresh_token,
    };
    cache::save(env, &c)?;
    Ok(c)
}

/// The cached access token, refreshed if it has less than 60s left
fn valid_token(env: &str) -> Result<String> {
    let c = cache::load(env)?;
    if c.expires_at - now() >= 60 {
        return Ok(c.access_token);
    }
    refresh(env, c).context("Your login has expired. Run `chepstow login` to log in again.")
}

/// Refresh the cached tokens and return the new access token
fn refresh(env: &str, c: cache::Cache) -> Result<String> {
    let rt = c.refresh_token.context("No refresh token cached.")?;
    if c.refresh_expires_at.is_some_and(|e| e <= now()) {
        bail!("Refresh token has expired.");
    }
    let ep = oidc::discover(&c.issuer)?;
    let t = oidc::refresh(&ep, &c.client_id, &rt)?;
    Ok(save(env, &c.issuer, &c.client_id, t, Some(rt))?.access_token)
}

fn main() -> Result<()> {
    let args = Args::parse();
    let Some(command) = args.command else {
        print_getting_started();
        return Ok(());
    };
    let (env, issuer, base_url) = match args.env {
        Env::Dev => (
            "dev",
            "https://keycloak-dev.isambard.ac.uk/realms/isambard",
            "https://apps-dev.isambard.ac.uk/inference",
        ),
        Env::Prod => (
            "prod",
            "https://keycloak.isambard.ac.uk/realms/isambard",
            "https://apps.isambard.ac.uk/inference",
        ),
    };
    let issuer = args.issuer.as_deref().unwrap_or(issuer);
    let base_url = args.base_url.as_deref().unwrap_or(base_url);

    match command {
        Command::Login => {
            let ep = oidc::discover(issuer)?;
            let t = oidc::device_login(&ep, &args.client_id, &args.scope, std::thread::sleep)?;
            save(env, issuer, &args.client_id, t, None)?;
            eprintln!("Logged in to {issuer}.");
        }
        Command::Token => match valid_token(env) {
            Ok(token) => println!("{token}"),
            Err(e) => {
                eprintln!("{e:#}");
                std::process::exit(1);
            }
        },
        Command::Whoami => {
            let c = cache::load(env)?;
            let claims = jwt::payload(&c.access_token)?;
            for name in ["iss", "aud", "short_name", "groups", "client_role"] {
                let value = match &claims[name] {
                    serde_json::Value::Null => "<missing>".to_string(),
                    serde_json::Value::String(s) => s.clone(),
                    v => v.to_string(),
                };
                println!("{name:<12} {value}");
            }
            let exp = chrono::DateTime::from_timestamp(c.expires_at, 0)
                .context("Invalid expiry time.")?
                .with_timezone(&chrono::Local);
            let left = c.expires_at - now();
            let left = if left > 0 {
                format!("{}m {}s left", left / 60, left % 60)
            } else {
                "expired".to_string()
            };
            println!(
                "{:<12} {} ({left})",
                "exp",
                exp.format("%Y-%m-%d %H:%M:%S %:z")
            );
        }
        Command::Models => {
            let url = format!("{}/v1/models", base_url.trim_end_matches('/'));
            let resp = oidc::http_client()?
                .get(&url)
                .bearer_auth(valid_token(env)?)
                .send()?;
            if !resp.status().is_success() {
                bail!("`{url}` returned {}.", resp.status());
            }
            let models: serde_json::Value = resp.json()?;
            for m in models["data"]
                .as_array()
                .context("No `data` in response.")?
            {
                println!("{}", m["id"].as_str().unwrap_or_default());
            }
        }
        Command::Logout => {
            if let Ok(c) = cache::load(env) {
                if let Some(rt) = &c.refresh_token {
                    let revoked = oidc::discover(&c.issuer)
                        .and_then(|ep| oidc::revoke(&ep, &c.client_id, rt));
                    if let Err(e) = revoked {
                        eprintln!("Warning: {e:#}");
                    }
                }
            }
            cache::delete(env)?;
            eprintln!("Logged out.");
        }
    }
    Ok(())
}
