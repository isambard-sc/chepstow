# Contributing

## Development

Install Rust with [rustup](https://rustup.rs).

### Build

```sh
cargo build                            # dev: fast to compile, unoptimised, target/debug/chepstow
cargo build --release                  # prod: optimised and stripped, target/release/chepstow
cargo run -- whoami                    # build and run a dev binary, passing args after --
cargo run --release -- --env dev auth
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
