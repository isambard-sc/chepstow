# chepstow

Log in to Isambard and get an access token for the inference service (LiteLLM).

chepstow uses the OAuth 2.0 device flow against Isambard Keycloak and prints the access-token JWT, so you can use it as a credential helper for Claude Code, opencode and other OpenAI/Anthropic-compatible tools.

## Install

Download a binary from the [releases page](https://github.com/isambard-sc/chepstow/releases), or build from source:

```sh
cargo install --git https://github.com/isambard-sc/chepstow
```

## Use

```sh
chepstow login            # open the browser and log in; add --qr for a QR code, --no-browser to just print the URL
chepstow token            # print a valid access token, refreshing it if needed
chepstow whoami           # show the token's claims and expiry
chepstow models           # list the models LiteLLM offers, to check it accepts the token
chepstow logout           # revoke the refresh token and delete the cache
```

`token` prints only the token on stdout. If it can't get a valid token, it exits non-zero and asks you to run `chepstow login` on stderr. It never starts a login itself.

Tokens are cached per environment in your cache directory (for example `~/.cache/chepstow/dev.json` on Linux), readable only by you.

### Environments and options

| `--env`       | issuer                                                | LiteLLM base URL                            |
|---------------|-------------------------------------------------------|---------------------------------------------|
| `dev` (default) | `https://keycloak-dev.isambard.ac.uk/realms/isambard` | `https://apps-dev.isambard.ac.uk/inference` |
| `prod`        | `https://keycloak.isambard.ac.uk/realms/isambard`     | `https://apps.isambard.ac.uk/inference`     |

Every option can also be set with an environment variable:

| flag          | env var              | default                      |
|---------------|----------------------|------------------------------|
| `--env`       | `CHEPSTOW_ENV`       | `dev`                        |
| `--issuer`    | `CHEPSTOW_ISSUER`    | from `--env`                 |
| `--client-id` | `CHEPSTOW_CLIENT_ID` | `chepstow`                   |
| `--base-url`  | `CHEPSTOW_BASE_URL`  | from `--env`                 |
| `--scope`     | `CHEPSTOW_SCOPE`     | `openid`                     |

Use `--scope "openid offline_access"` for a refresh token that outlives your Keycloak session, if the client allows it.

## Claude Code

In `~/.claude/settings.json`:

```json
{
  "apiKeyHelper": "chepstow token",
  "env": {
    "ANTHROPIC_BASE_URL": "https://apps-dev.isambard.ac.uk/inference",
    "CLAUDE_CODE_API_KEY_HELPER_TTL_MS": "240000"
  }
}
```

Set `CLAUDE_CODE_API_KEY_HELPER_TTL_MS` below the access-token lifetime, so Claude Code re-runs `chepstow token` before the token expires.
Don't use `ANTHROPIC_API_KEY`: Claude Code sends it as `x-api-key`, not as a `Bearer` token.

## opencode and OpenAI-style tools

These tools read a static key, for example:

```sh
export OPENAI_BASE_URL=https://apps-dev.isambard.ac.uk/inference/v1
export OPENAI_API_KEY=$(chepstow token)
```

or opencode's `{env:OPENAI_API_KEY}` / `{file:...}` in its config.
The token isn't refreshed while the tool runs, so it expires mid-session.
The fix is a longer access-token lifespan on the Keycloak client, which is a server-side decision.

## Server-side prerequisites

chepstow works only once these are in place:

1. **LiteLLM JWT auth** (brics-inference): `enable_jwt_auth: true`, `JWT_PUBLIC_KEY_URL` set to `{issuer}/protocol/openid-connect/certs`, and in `litellm_jwtauth`:
   - `user_id_jwt_field: "short_name"`
   - `team_ids_jwt_field: "groups"` with `team_id_upsert: true`
   - `team_allowed_routes` including `anthropic_routes`, for Claude Code's `/v1/messages`
   - optionally `JWT_AUDIENCE`

   Until then, LiteLLM treats the JWT as a virtual key and returns 401.
2. **Keycloak client:** a public client with the device grant enabled, the same `groups`, `short_name` and `client_role` mappers as the `litellm` client, an audience mapper if `JWT_AUDIENCE` is set, and an access-token lifespan that suits the tools above. `chepstow whoami` shows `<missing>` for any claim a mapper doesn't provide.
3. **Nadir front door** (platforms repo): the public URL only sends `Bearer sk-...` keys to LiteLLM, and sends any other bearer to oauth2-proxy. Until it routes JWTs, test through a port-forward:

   ```sh
   kubectl -n inference-dev port-forward svc/litellm 4000:4000
   chepstow models --base-url http://localhost:4000
   ```

See the [LiteLLM JWT auth docs](https://docs.litellm.ai/docs/proxy/token_auth).

## Development

Install Rust with [rustup](https://rustup.rs).

### Build

```sh
cargo build                            # dev: fast to compile, unoptimised, target/debug/chepstow
cargo build --release                  # prod: optimised and stripped, target/release/chepstow
cargo run -- whoami                    # build and run a dev binary, passing args after --
cargo run --release -- --env prod login
```

The build type and `--env` are separate: either binary can log in to either Keycloak. Release builds use the `[profile.release]` settings in `Cargo.toml` (LTO, size-optimised, stripped), which are what CI ships.

### Check

Run these before pushing; CI runs the same:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

See [RELEASE.md](RELEASE.md) for making a release.
