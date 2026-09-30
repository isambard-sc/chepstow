# chepstow

Rust CLI that logs in to Isambard Keycloak with the OAuth 2.0 device flow (RFC 8628) and prints an access-token JWT for LiteLLM. User-facing docs are in README.md; releases in RELEASE.md.

**Keep the code minimal.** Prefer the smallest change that works. Don't add crates, abstractions or workarounds for server-side problems (for example token lifetime or routing).

## Layout

```
src/main.rs    clap args, env profiles (dev/prod), subcommands, valid_token() refresh logic
src/oidc.rs    discovery, device flow, refresh, revoke (oauth2 5 BasicClient)
src/cache.rs   per-env token cache: dirs::cache_dir()/chepstow/<env>.json, mode 0600
src/jwt.rs     unverified JWT payload decode
tests/cli.rs   runs the binary against mockito
```

## Design choices

- Blocking reqwest + rustls, no async runtime. `default-features = false` on every crate.
- Endpoints come from `{issuer}/.well-known/openid-configuration`. They are never hardcoded.
- Public client: `AuthType::RequestBody`, no secret. The `chepstow` client ID is a placeholder until the real Keycloak client exists. Don't use the confidential `litellm` client.
- The oauth2 crate handles polling (`authorization_pending`, `slow_down` +5s, `expired_token`, `access_denied`). The sleep function is injected so tests don't wait.
- `expires_at` and `refresh_expires_at` come from each token's JWT `exp` claim. Keycloak refresh tokens are JWTs, and offline tokens have no expiry, giving `None`. Times are unix seconds.
- `token` prints only the token on stdout. On failure it prints `run chepstow login` on stderr and exits 1, and never starts a login. Other human-readable output also goes to stderr.
- `token` refreshes using the issuer and client ID stored in the cache, not the current flags.
- `whoami` reads the cache without refreshing. Missing claims print as `<missing>` so a missing Keycloak mapper is obvious.
- `logout` warns but still deletes the cache if revocation fails.

## Rules

- Lints: `unsafe_code` forbidden; `unwrap_used`, `expect_used` and `dbg_macro` denied, in tests too. Tests return `anyhow::Result` and use `?`.
- Before finishing a change, run:
  ```sh
  cargo fmt --check
  cargo clippy --all-targets -- -D warnings
  cargo test
  ```
- Add user-visible changes to the `Unreleased` section of CHANGELOG.md (Keep a Changelog format; CI checks it with kacl).
- CI is in `.github/workflows/` (check, build, release), ported from Isambard's clifton. Actions are pinned by SHA.

## Not yet working end to end

These are server-side and out of scope for this repo. See README "Server-side prerequisites".
- LiteLLM JWT auth is not enabled in brics-inference.
- The Keycloak public client doesn't exist yet.
- The nadir front door doesn't route JWTs to LiteLLM. Test through `kubectl port-forward` instead.
