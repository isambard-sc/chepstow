# chepstow — implementation plan

Rust CLI that logs a user in to Isambard Keycloak via the OAuth 2.0 Device
Authorization Grant (RFC 8628) and prints an access-token JWT for LiteLLM
(an OpenAI/Anthropic-compatible proxy). Users paste it into, or wire it into,
Claude Code, opencode and similar tools. 

This implementation plan is written by another claude code agent. This is an important note from me Wahab. Please make sure you use as little code as possible in this repo. Also write the RELEASE.md but without setting up for winget. After youre done write CLAUDE.md and I will delete the clifton/ repo.

## Reference code: `clifton/`

`clifton/` is a copy of Isambard's existing SSH-cert CLI. It does this device
flow against the same Keycloak. **Reference only — don't commit it or depend on it.**
Copy its approach:

- `clifton/src/auth.rs`: `oauth2` 5 `BasicClient` + `exchange_device_code()` /
  `exchange_device_access_token()`, `AuthType::RequestBody` (public client),
  browser open + QR code. It hardcodes the Keycloak paths and has a TODO to use
  discovery. Do that TODO here.
- `clifton/src/cache.rs`: cache dir under `dirs::cache_dir()`, files written `0600`.
- `clifton/Cargo.toml`: dependency set (blocking reqwest + rustls, no tokio),
  release profile, and lints (`unwrap_used`/`expect_used` deny, `unsafe_code` forbid).
  Keep the same style.

## Environments

| env  | issuer                                               | LiteLLM base URL                              |
|------|------------------------------------------------------|-----------------------------------------------|
| dev  | `https://keycloak-dev.isambard.ac.uk/realms/isambard` | `https://apps-dev.isambard.ac.uk/inference`   |
| prod | `https://keycloak.isambard.ac.uk/realms/isambard`     | `https://apps.isambard.ac.uk/inference`       |

- Default to `dev`.
- Client ID: a new **public** Keycloak client with Device Authorization Grant
  enabled. The name is TBD, so use `chepstow` as a placeholder. Don't use the
  existing `litellm` client: it's confidential (it has a secret) and is only for
  the LiteLLM admin UI's SSO.
- Every value can be overridden by a flag or env var: `--issuer`/`CHEPSTOW_ISSUER`,
  `--client-id`/`CHEPSTOW_CLIENT_ID`, `--base-url`/`CHEPSTOW_BASE_URL`,
  `--scope`/`CHEPSTOW_SCOPE` (default `openid`; `openid offline_access` if the
  realm allows offline tokens).

Get endpoints from `{issuer}/.well-known/openid-configuration`:
`device_authorization_endpoint`, `token_endpoint`, `revocation_endpoint`.

## What LiteLLM does with the token

LiteLLM validates `Authorization: Bearer <jwt>` against the realm JWKS
(`{issuer}/protocol/openid-connect/certs`) and reads these claims:

- `groups`: list of LiteLLM team IDs (`team_ids_jwt_field: "groups"`, `team_id_upsert: true`).
- `short_name`: stable user ID (the same one the LiteLLM UI SSO uses).
- `client_role`: LiteLLM role (`proxy_admin`, `internal_user`, ...).
- `aud`: checked only if the server sets `JWT_AUDIENCE`.

These claims come from Keycloak mappers, not from chepstow. `chepstow whoami`
must show them so a missing mapper is obvious.

Docs: https://docs.litellm.ai/docs/proxy/token_auth

## Commands

- `chepstow login [--env dev|prod] [--no-browser] [--qr]`
  - Runs the device flow: print `verification_uri_complete` (optionally open
    the browser and/or show a QR code), then poll.
  - Handle `authorization_pending`, `slow_down` (+5s interval), `expired_token`
    and `access_denied`. The `oauth2` crate does this; check its behaviour
    in tests.
  - Cache access token, refresh token and both expiry times.
- `chepstow token`
  - Prints a valid access token to stdout and nothing else (it's used as a
    credential helper).
  - Refreshes if fewer than 60s are left.
  - If refresh fails, exit non-zero and print `run chepstow login` on stderr.
    Never start an interactive flow from `token`.
- `chepstow whoami`
  - Decodes the JWT payload without verifying it (base64url + serde_json).
  - Prints `iss`, `aud`, `short_name`, `groups`, `client_role`, and `exp` as
    local time and time remaining.
- `chepstow models`
  - `GET {base_url}/v1/models` with the token and prints the model IDs.
    It's a smoke test that LiteLLM accepts the token.
- `chepstow logout`
  - Revokes the refresh token at `revocation_endpoint`, then deletes the cache file.

Cache: `dirs::cache_dir()/chepstow/<env>.json`, mode `0600`, with fields
`issuer`, `client_id`, `access_token`, `refresh_token`, `expires_at`,
`refresh_expires_at`. Use one file per environment so dev and prod don't overwrite
each other.

## Harness integration (document in README)

- **Claude Code:** in `settings.json`, set `"apiKeyHelper": "chepstow token"` and
  `ANTHROPIC_BASE_URL={base_url}`. Set `CLAUDE_CODE_API_KEY_HELPER_TTL_MS` below
  the access-token lifetime so Claude Code re-runs the helper before the token
  expires. Don't use `ANTHROPIC_API_KEY`: it's sent as `x-api-key`, not `Bearer`.
- **opencode / OpenAI-style tools:** they read a static key such as
  `OPENAI_API_KEY=$(chepstow token)` or opencode's `{env:...}`/`{file:...}`.
  The token expires mid-session. The fix is a longer access-token lifespan on
  the Keycloak client, which is a server-side decision (see Prerequisites).
  Don't build a workaround in chepstow.

## Layout

```
src/main.rs    clap subcommands, env profile resolution
src/oidc.rs    discovery, device flow, refresh, revoke
src/cache.rs   per-env token cache (0600)
src/jwt.rs     unverified payload decode for whoami / expiry
```

- Crates: `clap`, `anyhow`, `oauth2` 5 (`reqwest-blocking`, `rustls-tls`),
  `reqwest` (blocking, json, rustls), `serde`, `serde_json`, `url`, `dirs`,
  `webbrowser`, `qrcode`, `base64`, `chrono`.
- Dev dependency: `mockito`.
- No async runtime.

## Prerequisites outside chepstow (don't block on these; report them)

1. **LiteLLM doesn't accept JWTs yet.** The brics-inference LiteLLM config has
   `litellm_jwtauth` but no `enable_jwt_auth: true` and no
   `JWT_PUBLIC_KEY_URL`. Until those are set, LiteLLM treats the JWT as a
   virtual key and returns 401. The fix belongs in brics-inference, which also needs:
   - `user_id_jwt_field: "short_name"`
   - optionally `JWT_AUDIENCE`
   - `team_allowed_routes` including `anthropic_routes` for Claude Code's `/v1/messages`
2. **Keycloak client:**
   - public, with device grant enabled
   - the same `groups`/`short_name`/`client_role` mappers as the `litellm` client
   - an audience mapper if `JWT_AUDIENCE` is set
   - an access-token lifespan that suits the harnesses
3. **Nadir front door:** the public URL sends only `Bearer sk-<22 chars>` to
   LiteLLM. Any other bearer goes to oauth2-proxy and gets a 302 to
   `/inference/_oidc`. Until the nadir chart (platforms repo) routes JWTs,
   `chepstow models` against the public URL will fail. Test through a
   port-forward instead (below).

## Verification

- `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, `cargo test`.
- mockito tests covering:
  - discovery
  - device flow: pending → `slow_down` → success
  - `token` refreshing near expiry
  - expired refresh → non-zero exit, nothing on stdout
  - JWT payload decode
- Manual, against keycloak-dev once the client exists:
  - `chepstow login --env dev`
  - `chepstow whoami` shows `groups`, `short_name` and `client_role`.
- LiteLLM, once prerequisite 1 is done. Someone with cluster access runs
  `kubectl -n inference-dev port-forward svc/litellm 4000:4000`, then:
  ```sh
  chepstow models --base-url http://localhost:4000
  curl -s localhost:4000/v1/chat/completions -H "Authorization: Bearer $(chepstow token)" \
    -H 'Content-Type: application/json' \
    -d '{"model":"qwen3-8b","messages":[{"role":"user","content":"hi"}]}'
  ```
