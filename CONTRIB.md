

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
